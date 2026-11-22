use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use axum::{
    extract::Request,
    http::{Method, StatusCode, uri::PathAndQuery},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use super::WebError;

const MAX_STATUS_LINE: usize = 1024;
const MAX_PROBE_PATH: usize = 1024;

/// An explicitly selected startup HTTP probe.
///
/// Before ordinary group admission, only this exact method and path-and-query
/// may reach the application. The exception ends when shutdown fences admission.
/// The probe uses one retained deadline, expects the selected status exactly,
/// and reads at most 1024 bytes of the response status line.
#[derive(Clone, Debug)]
pub struct HttpProbe {
    method: Method,
    path: String,
    expected: StatusCode,
    timeout: Duration,
}

impl HttpProbe {
    pub fn new(method: Method, path: impl Into<String>, expected_status: StatusCode) -> Self {
        Self {
            method,
            path: path.into(),
            expected: expected_status,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub(super) fn validate(&self) -> Result<(), WebError> {
        let valid = self.path.starts_with('/')
            && self.path.len() <= MAX_PROBE_PATH
            && self.path.is_ascii()
            && !self.path.contains('#')
            && !self
                .path
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
            && self.path.parse::<PathAndQuery>().is_ok();
        if valid {
            Ok(())
        } else {
            Err(WebError::message(
                io::ErrorKind::InvalidInput,
                "HTTP probe requires an ASCII path-and-query of at most 1024 bytes",
            ))
        }
    }

    pub(super) fn matches(&self, request: &Request) -> bool {
        request.method() == self.method
            && request
                .uri()
                .path_and_query()
                .is_some_and(|path| path.as_str() == self.path)
    }
}

pub(super) async fn run(endpoint: SocketAddr, probe: HttpProbe) -> Result<(), WebError> {
    let deadline = tokio::time::Instant::now()
        .checked_add(probe.timeout)
        .ok_or_else(|| {
            WebError::message(
                io::ErrorKind::InvalidInput,
                "HTTP probe deadline is out of range",
            )
        })?;
    // timeout_at polls the exchange before its timer. A retained exchange can
    // become ready after this deadline while the borrowing drive is suspended.
    // Check the same absolute deadline before publishing successful readiness.
    match tokio::time::timeout_at(deadline, exchange(endpoint, &probe)).await {
        Ok(Ok(())) if tokio::time::Instant::now() < deadline => Ok(()),
        Ok(Err(error)) => Err(error),
        _ => Err(WebError::message(
            io::ErrorKind::TimedOut,
            "HTTP startup probe timed out",
        )),
    }
}

async fn exchange(mut endpoint: SocketAddr, probe: &HttpProbe) -> Result<(), WebError> {
    // Wildcard listeners are probed through the matching loopback family.
    if endpoint.ip().is_unspecified() {
        endpoint.set_ip(match endpoint.ip() {
            IpAddr::V4(_) => IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(_) => IpAddr::V6(Ipv6Addr::LOCALHOST),
        });
    }
    let mut stream = TcpStream::connect(endpoint).await?;
    let request = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
        probe.method, probe.path, endpoint,
    );
    stream.write_all(request.as_bytes()).await?;
    let mut line = Vec::with_capacity(128);
    loop {
        let byte = stream.read_u8().await?;
        line.push(byte);
        if byte == b'\n' {
            break;
        }
        if line.len() >= MAX_STATUS_LINE {
            return Err(WebError::message(
                io::ErrorKind::InvalidData,
                "HTTP startup probe status line exceeded 1024 bytes",
            ));
        }
    }
    let actual = parse_status(&line)?;
    if actual == probe.expected {
        Ok(())
    } else {
        Err(WebError::message(
            io::ErrorKind::Other,
            format!(
                "HTTP startup probe expected {}, received {actual}",
                probe.expected
            ),
        ))
    }
}

fn parse_status(line: &[u8]) -> Result<StatusCode, WebError> {
    let text = std::str::from_utf8(line).map_err(|_| {
        WebError::message(io::ErrorKind::InvalidData, "invalid HTTP probe status line")
    })?;
    let Some(text) = text.strip_suffix("\r\n") else {
        return Err(WebError::message(
            io::ErrorKind::InvalidData,
            "invalid HTTP probe status line",
        ));
    };
    let mut fields = text.splitn(3, ' ');
    let version = fields.next();
    let status = fields.next();
    if !matches!(version, Some("HTTP/1.0" | "HTTP/1.1")) {
        return Err(WebError::message(
            io::ErrorKind::InvalidData,
            "invalid HTTP probe version",
        ));
    }
    match status
        .filter(|status| status.len() == 3 && status.bytes().all(|byte| byte.is_ascii_digit()))
    {
        Some(status) => StatusCode::from_bytes(status.as_bytes()).map_err(|_| {
            WebError::message(io::ErrorKind::InvalidData, "invalid HTTP probe status")
        }),
        None => Err(WebError::message(
            io::ErrorKind::InvalidData,
            "invalid HTTP probe status",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_configuration_rejects_request_line_injection_and_preserves_query() {
        assert!(
            HttpProbe::new(Method::GET, "/health?ready=1", StatusCode::NO_CONTENT)
                .validate()
                .is_ok()
        );
        for path in [
            "/health HTTP/1.1",
            "/health\r\nInjected: yes",
            "https://example.test/health",
            "/health#fragment",
        ] {
            assert!(
                HttpProbe::new(Method::GET, path, StatusCode::OK)
                    .validate()
                    .is_err()
            );
        }
    }

    #[test]
    fn status_line_requires_an_exact_http_status() {
        assert_eq!(
            parse_status(b"HTTP/1.1 204 No Content\r\n").unwrap(),
            StatusCode::NO_CONTENT
        );
        for line in [
            &b"HTTP/1.1 2000 Wrong\r\n"[..],
            b"HTTP/2 200 OK\r\n",
            b"HTTP/1.1 200 OK\n",
            b"garbage",
        ] {
            assert!(parse_status(line).is_err());
        }
    }
}
