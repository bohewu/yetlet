//! The CDP/BiDi wire operations used by the original suites. No external runtime.
use super::require;
use crate::tooling::{Result, fail};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha1::{Digest, Sha1};
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
const MAX_FRAME: usize = 32 * 1024 * 1024;
pub struct Socket {
    stream: TcpStream,
}
impl Socket {
    pub fn connect(url: &str) -> Result<Self> {
        Self::connect_origin(url, true)
    }
    pub fn connect_bidi(url: &str) -> Result<Self> {
        Self::connect_origin(url, false)
    }
    fn connect_origin(url: &str, origin: bool) -> Result<Self> {
        let rest = url
            .strip_prefix("ws://127.0.0.1:")
            .or_else(|| url.strip_prefix("ws://localhost:"))
            .ok_or_else(|| fail("WebSocket endpoint must be owned loopback"))?;
        let (port, path) = rest
            .split_once('/')
            .ok_or_else(|| fail("WebSocket URL lacks path"))?;
        let port = port.parse::<u16>().map_err(|_| fail("WebSocket port"))?;
        let mut stream =
            TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_secs(8))?;
        stream.set_read_timeout(Some(Duration::from_secs(12)))?;
        stream.set_write_timeout(Some(Duration::from_secs(12)))?;
        let key = STANDARD.encode(random::<16>()?);
        let origin = if origin {
            "Origin: http://127.0.0.1\r\n"
        } else {
            ""
        };
        stream.write_all(format!("GET /{path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n{origin}\r\n").as_bytes())?;
        let mut header = Vec::new();
        while !header.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte)?;
            header.push(byte[0]);
            require(header.len() < 65536, "WebSocket header too large")?;
        }
        verify_handshake(&header, &key)?;
        Ok(Self { stream })
    }
    pub fn send(&mut self, value: &serde_json::Value) -> Result<()> {
        send_frame(
            &mut self.stream,
            1,
            &serde_json::to_vec(value).map_err(|_| fail("WebSocket JSON"))?,
            random::<4>()?,
        )
    }
    pub fn receive(&mut self, deadline: Instant) -> Result<serde_json::Value> {
        let mut fragments = Vec::new();
        let mut started = false;
        loop {
            crate::tooling::check_cancel()?;
            require(Instant::now() < deadline, "WebSocket command deadline")?;
            self.stream.set_read_timeout(Some(
                deadline
                    .saturating_duration_since(Instant::now())
                    .max(Duration::from_millis(1)),
            ))?;
            let (fin, opcode, content) = read_frame(&mut self.stream)?;
            match opcode {
                8 => return Err(fail("WebSocket closed")),
                9 => {
                    require(fin && content.len() <= 125, "invalid WebSocket ping")?;
                    send_frame(&mut self.stream, 10, &content, random::<4>()?)?;
                    continue;
                }
                10 => continue,
                1 => {
                    require(!started, "unexpected nested text frame")?;
                    fragments = content;
                    started = true;
                }
                0 => {
                    require(started, "unexpected continuation")?;
                    require(
                        fragments.len() + content.len() <= MAX_FRAME,
                        "fragment limit",
                    )?;
                    fragments.extend(content);
                }
                _ => return Err(fail("unexpected WebSocket opcode")),
            }
            if fin {
                return serde_json::from_slice(&fragments)
                    .map_err(|_| fail("invalid WebSocket JSON"));
            }
        }
    }
}
fn random<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|_| fail("WebSocket random source unavailable"))?;
    Ok(bytes)
}
pub fn verify_handshake(header: &[u8], key: &str) -> Result<()> {
    let header = std::str::from_utf8(header).map_err(|_| fail("WebSocket invalid handshake"))?;
    require(
        header.lines().next().is_some_and(|s| s.contains(" 101 ")),
        "WebSocket handshake status",
    )?;
    let expected = STANDARD.encode(Sha1::digest(format!(
        "{key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
    )));
    require(
        header
            .lines()
            .filter_map(|l| l.split_once(':'))
            .any(|(name, value)| {
                name.eq_ignore_ascii_case("sec-websocket-accept") && value.trim() == expected
            }),
        "WebSocket accept mismatch",
    )
}
pub fn send_frame(writer: &mut impl Write, opcode: u8, bytes: &[u8], mask: [u8; 4]) -> Result<()> {
    require(bytes.len() <= MAX_FRAME, "WebSocket send limit")?;
    let mut header = vec![0x80 | opcode];
    let len = bytes.len();
    if len < 126 {
        header.push(0x80 | len as u8);
    } else if len < 65536 {
        header.push(0xfe);
        header.extend((len as u16).to_be_bytes());
    } else {
        header.push(0xff);
        header.extend((len as u64).to_be_bytes());
    }
    header.extend(mask);
    writer.write_all(&header)?;
    let masked = bytes
        .iter()
        .enumerate()
        .map(|(i, b)| b ^ mask[i % 4])
        .collect::<Vec<_>>();
    writer.write_all(&masked)?;
    Ok(())
}
pub fn read_frame(reader: &mut impl Read) -> Result<(bool, u8, Vec<u8>)> {
    let mut prefix = [0; 2];
    reader.read_exact(&mut prefix)?;
    require(prefix[0] & 0x70 == 0, "WebSocket reserved bits")?;
    let mut size = (prefix[1] & 0x7f) as u64;
    if size == 126 {
        let mut b = [0; 2];
        reader.read_exact(&mut b)?;
        size = u16::from_be_bytes(b) as u64;
    } else if size == 127 {
        let mut b = [0; 8];
        reader.read_exact(&mut b)?;
        size = u64::from_be_bytes(b);
    }
    require(size <= MAX_FRAME as u64, "WebSocket frame limit")?;
    let mut mask = [0; 4];
    let masked = prefix[1] & 0x80 != 0;
    if masked {
        reader.read_exact(&mut mask)?;
    }
    let mut content = vec![0; size as usize];
    reader.read_exact(&mut content)?;
    if masked {
        for (i, b) in content.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
    }
    Ok((prefix[0] & 0x80 != 0, prefix[0] & 15, content))
}
