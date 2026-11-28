use std::{convert::Infallible, io, path::PathBuf, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    response::sse::{Event, Sse},
};
use futures_util::{Stream, StreamExt, stream};
use http_body_util::BodyExt;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use serde::Serialize;
use tokio::{
    fs::File,
    io::AsyncReadExt,
    sync::watch,
    time::{sleep, timeout},
};
use tokio_util::io::ReaderStream;

use super::{CHUNK, Failure, MAX_BYTES, PACE, Receipt, TIMEOUT};

#[derive(Clone, Debug, Serialize)]
struct Snapshot {
    phase: &'static str,
    sent: u64,
    total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt: Option<Receipt>,
}

impl Snapshot {
    fn terminal(&self) -> bool {
        matches!(self.phase, "done" | "failed")
    }
}

#[derive(Clone)]
pub struct Sender {
    path: Arc<PathBuf>,
    receiver_port: u16,
    progress: watch::Sender<Snapshot>,
}

impl Sender {
    pub fn new(receiver_port: u16, path: PathBuf) -> Self {
        let progress = watch::Sender::new(Snapshot {
            phase: "ready",
            sent: 0,
            total: 0,
            receipt: None,
        });
        Self {
            path: Arc::new(path),
            receiver_port,
            progress,
        }
    }

    async fn transfer(&self) -> Result<Receipt, Failure> {
        let file = File::open(self.path.as_ref()).await?;
        let metadata = file.metadata().await?;
        let total = metadata.len();
        if !metadata.is_file() || !(1..=MAX_BYTES).contains(&total) {
            return Err("expected a regular file of 1 byte to 64 MiB".into());
        }
        self.progress.send_modify(|s| {
            s.total = total;
            s.phase = "upload";
        });
        let progress = self.progress.clone();
        let chunks = ReaderStream::with_capacity(file.take(total), CHUNK).then(move |chunk| {
            let progress = progress.clone();
            async move {
                sleep(PACE).await;
                if let Ok(bytes) = &chunk {
                    progress.send_modify(|s| s.sent += bytes.len() as u64);
                }
                chunk
            }
        });
        let request = Request::post(format!("http://127.0.0.1:{}/receive", self.receiver_port))
            .header(header::CONTENT_LENGTH, total)
            .body(Body::from_stream(chunks))?;
        let client = Client::builder(TokioExecutor::new()).build_http();
        let response = client.request(request).await?;
        if !response.status().is_success() {
            return Err(format!("receiver returned {}", response.status()).into());
        }
        self.progress.send_modify(|s| s.phase = "scan");
        let bytes = http_body_util::Limited::new(response.into_body(), 1024)
            .collect()
            .await?
            .to_bytes();
        let receipt: Receipt = serde_json::from_slice(&bytes)?;
        if receipt.bytes != total {
            return Err("receiver scan length differs from source length".into());
        }
        Ok(receipt)
    }
}

pub async fn start(State(sender): State<Sender>, headers: HeaderMap) -> StatusCode {
    if headers.contains_key(header::ORIGIN) {
        return StatusCode::FORBIDDEN;
    }
    let started = sender.progress.send_if_modified(|s| {
        if s.phase != "ready" {
            return false;
        }
        s.phase = "starting";
        true
    });
    if !started {
        return StatusCode::CONFLICT;
    }
    tokio::spawn(async move {
        let result = match timeout(TIMEOUT, sender.transfer()).await {
            Ok(result) => result,
            Err(_) => Err(io::Error::new(io::ErrorKind::TimedOut, "job timed out").into()),
        };
        sender.progress.send_modify(|s| match result {
            Ok(receipt) => {
                s.receipt = Some(receipt);
                s.phase = "done";
            }
            Err(error) => {
                eprintln!("job failed: {error}");
                s.phase = "failed";
            }
        });
    });
    StatusCode::ACCEPTED
}

pub async fn events(
    State(sender): State<Sender>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // borrow_and_update couples the first snapshot to the watch cursor. A change
    // that races the next poll stays visible even if it is the terminal change.
    let receiver = sender.progress.subscribe();
    let events = stream::unfold(
        (receiver, true, false),
        |(mut receiver, first, ended)| async move {
            if ended || (!first && receiver.changed().await.is_err()) {
                return None;
            }
            let current = receiver.borrow_and_update().clone();
            let ended = current.terminal();
            let event = Event::default()
                .json_data(&current)
                .expect("snapshot contains only JSON values");
            Some((Ok(event), (receiver, false, ended)))
        },
    );
    Sse::new(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn failed_job_survives_absent_subscribers_and_cannot_restart() {
        let directory = tempfile::tempdir().unwrap();
        let sender = Sender::new(0, directory.path().join("missing"));
        let mut observer = sender.progress.subscribe();
        assert_eq!(
            start(State(sender.clone()), HeaderMap::new()).await,
            StatusCode::ACCEPTED
        );
        timeout(TIMEOUT, observer.wait_for(|s| s.terminal()))
            .await
            .unwrap()
            .unwrap();
        drop(observer);
        assert_eq!(sender.progress.subscribe().borrow().phase, "failed");
        assert_eq!(
            start(State(sender), HeaderMap::new()).await,
            StatusCode::CONFLICT
        );
    }
}
