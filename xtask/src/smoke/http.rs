//! Loopback HTTP, explicit cookies and no redirects/proxies. Never logs secrets.
use crate::tooling::{Result, fail};
use serde_json::Value;
use std::{collections::BTreeMap, time::Duration};
pub struct Response {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}
impl Response {
    pub fn text(&self) -> Result<&str> {
        std::str::from_utf8(&self.body).map_err(|_| fail("response is not UTF-8"))
    }
    pub fn json(&self) -> Result<Value> {
        serde_json::from_slice(&self.body).map_err(|_| fail("invalid response JSON"))
    }
    pub fn header(&self, k: &str) -> Result<&str> {
        self.headers
            .get(k)
            .map(String::as_str)
            .ok_or_else(|| fail(format!("missing header {k}")))
    }
    pub fn cookie(&self) -> Result<String> {
        Ok(self.header("set-cookie")?.split(';').next().unwrap().into())
    }
    pub fn hidden(&self, name: &str) -> Result<String> {
        hidden(self.text()?, name)
    }
}
pub fn hidden(html: &str, name: &str) -> Result<String> {
    let pattern = format!("name=\"{}\" value=\"([^\"]+)\"", regex::escape(name));
    let re = regex::Regex::new(&pattern).map_err(|_| fail("invalid hidden-field pattern"))?;
    let value = re
        .captures(html)
        .and_then(|m| m.get(1))
        .ok_or_else(|| fail(format!("missing hidden field {name}")))?
        .as_str();
    Ok(value
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&"))
}
pub fn agent(timeout: u64) -> ureq::Agent {
    ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(timeout)))
        .build()
        .into()
}
pub fn call(
    base: &str,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
    headers: &[(&str, &str)],
) -> Result<Response> {
    call_timeout(base, method, path, body, headers, 5)
}
pub fn call_timeout(
    base: &str,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
    headers: &[(&str, &str)],
    timeout: u64,
) -> Result<Response> {
    let agent = agent(timeout);
    call_agent(&agent, base, method, path, body, headers)
}
/// Board's original per-client HTTP connection reuse matters to its raw timings.
pub fn call_agent(
    agent: &ureq::Agent,
    base: &str,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
    headers: &[(&str, &str)],
) -> Result<Response> {
    crate::tooling::check_cancel()?;
    let mut req = ureq::http::Request::builder()
        .method(method)
        .uri(format!("{base}{path}"));
    for (key, value) in headers {
        req = req.header(*key, *value);
    }
    let req = req
        .body(body.unwrap_or_default())
        .map_err(|_| fail("invalid HTTP request"))?;
    let mut response = agent.run(req).map_err(|_| fail("HTTP transport failed"))?;
    let status = response.status().as_u16();
    let mut fields = BTreeMap::new();
    for (key, value) in response.headers() {
        fields.insert(
            key.as_str().to_ascii_lowercase(),
            value.to_str().map_err(|_| fail("invalid header"))?.into(),
        );
    }
    let body = response
        .body_mut()
        .with_config()
        .limit(16 * 1024 * 1024)
        .read_to_vec()
        .map_err(|_| fail("HTTP body failed"))?;
    Ok(Response {
        status,
        headers: fields,
        body,
    })
}
pub fn get(base: &str, path: &str) -> Result<Response> {
    call(base, "GET", path, None, &[])
}
pub fn json(
    base: &str,
    method: &str,
    path: &str,
    value: &Value,
    extra: &[(&str, &str)],
) -> Result<Response> {
    let mut headers = vec![("content-type", "application/json")];
    headers.extend_from_slice(extra);
    call(
        base,
        method,
        path,
        Some(&serde_json::to_vec(value).map_err(|_| fail("JSON encode"))?),
        &headers,
    )
}
pub fn form(base: &str, path: &str, cookie: &str, fields: &[(&str, &str)]) -> Result<Response> {
    let body = fields
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    call(
        base,
        "POST",
        path,
        Some(body.as_bytes()),
        &[
            ("Cookie", cookie),
            ("Content-Type", "application/x-www-form-urlencoded"),
        ],
    )
}
fn encode(text: &str) -> String {
    percent_encoding::utf8_percent_encode(text, percent_encoding::NON_ALPHANUMERIC).to_string()
}
pub fn authenticated_get(base: &str, path: &str, cookie: &str) -> Result<Response> {
    call(base, "GET", path, None, &[("Cookie", cookie)])
}
#[derive(Default)]
pub struct Cookies {
    cookie: String,
}
impl Cookies {
    pub fn get(&mut self, base: &str, path: &str) -> Result<Response> {
        let r = authenticated_get(base, path, &self.cookie)?;
        self.update(&r);
        Ok(r)
    }
    pub fn form(&mut self, base: &str, path: &str, fields: &[(&str, &str)]) -> Result<Response> {
        let r = form(base, path, &self.cookie, fields)?;
        self.update(&r);
        Ok(r)
    }
    fn update(&mut self, r: &Response) {
        if let Ok(c) = r.cookie() {
            self.cookie = c;
        }
    }
}
