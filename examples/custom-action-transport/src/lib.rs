//! YT-024: an opt-in, native-only application adapter, not a core transport API.

use std::{net::SocketAddr, time::Duration};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, time::timeout};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use yetlet_actions::{ActionError, ActionExecutor, Command};

pub const MAX_FRAME_BYTES: usize = 1024;
pub const SERVER_DEADLINE: Duration = Duration::from_secs(1);

/// Synthetic, read-only arithmetic; no identity, secrets or DOM events.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Add {
    pub left: u32,
    pub right: u32,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sum {
    pub value: u32,
}

impl Command for Add {
    type Response = Sum;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
enum Reply {
    Ok { value: u32 },
    Error { code: String },
}

fn framed(socket: TcpStream) -> Framed<TcpStream, LengthDelimitedCodec> {
    LengthDelimitedCodec::builder()
        .big_endian()
        .length_field_length(4)
        .max_frame_length(MAX_FRAME_BYTES)
        .new_framed(socket)
}

/// Application code explicitly selects this executor instead of LocalExecutor.
/// Loopback-only construction prevents accidentally exposing this unauthenticated spike.
pub struct TcpExecutor {
    endpoint: SocketAddr,
    deadline: Duration,
}

impl TcpExecutor {
    pub fn new(endpoint: SocketAddr, deadline: Duration) -> Result<Self, ActionError> {
        if !endpoint.ip().is_loopback() || endpoint.port() == 0 || deadline.is_zero() {
            return Err(ActionError::InvalidEndpoint);
        }
        Ok(Self { endpoint, deadline })
    }
}

impl ActionExecutor<Add> for TcpExecutor {
    async fn execute(&self, command: Add) -> Result<Sum, ActionError> {
        timeout(self.deadline, async {
            let request = serde_json::to_vec(&command).map_err(|_| ActionError::InvalidResponse)?;
            let socket = TcpStream::connect(self.endpoint)
                .await
                .map_err(|_| ActionError::Network)?;
            let mut wire = framed(socket);
            wire.send(request.into())
                .await
                .map_err(|_| ActionError::Network)?;
            let response = wire
                .next()
                .await
                .ok_or(ActionError::InvalidResponse)?
                .map_err(|_| ActionError::InvalidResponse)?;
            let reply: Reply =
                serde_json::from_slice(&response).map_err(|_| ActionError::InvalidResponse)?;
            match reply {
                Reply::Ok { value } => Ok(Sum { value }),
                // Status is an app-owned compatibility mapping, not a TCP/HTTP status.
                Reply::Error { code } => Err(ActionError::Remote { status: 422, code }),
            }
            // Dropping `wire` closes the connection, also on error/cancellation.
        })
        .await
        .map_err(|_| ActionError::Network)?
    }
}

/// One accepted connection, one command, one reply; no session or event circuit.
/// The application owns listener setup and must bind it only to loopback.
pub async fn serve_connection(socket: TcpStream) -> std::io::Result<()> {
    if !socket.local_addr()?.ip().is_loopback() || !socket.peer_addr()?.ip().is_loopback() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "research transport requires loopback",
        ));
    }
    timeout(SERVER_DEADLINE, async {
        let mut wire = framed(socket);
        let Some(request) = wire.next().await else {
            return Ok(());
        };
        let request = request?;
        let reply = match serde_json::from_slice::<Add>(&request) {
            Ok(command) => match command.left.checked_add(command.right) {
                Some(value) => Reply::Ok { value },
                None => Reply::Error {
                    code: "overflow".into(),
                },
            },
            Err(_) => Reply::Error {
                code: "invalid_command".into(),
            },
        };
        wire.send(serde_json::to_vec(&reply)?.into()).await?;
        wire.close().await
    })
    .await
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "command deadline"))?
}

#[cfg(test)]
mod tests;
