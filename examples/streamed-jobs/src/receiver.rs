use std::{io, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, header},
};
use futures_util::{StreamExt, stream};
use sha2::{Digest, Sha256};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Semaphore,
    time::{sleep, timeout},
};

use super::{CHUNK, MAX_BYTES, PACE, Receipt, TIMEOUT};

pub async fn receive(
    State(slot): State<Arc<Semaphore>>,
    headers: HeaderMap,
    body: Body,
) -> Result<Body, StatusCode> {
    if headers.contains_key(header::ORIGIN) {
        return Err(StatusCode::FORBIDDEN);
    }
    let _permit = slot.try_acquire_owned().map_err(|_| StatusCode::CONFLICT)?;
    let length: u64 = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok()?.parse().ok())
        .ok_or(StatusCode::LENGTH_REQUIRED)?;
    if !(1..=MAX_BYTES).contains(&length) {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let mut file =
        File::from_std(tempfile::tempfile().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    let upload = async {
        let mut received = 0;
        let mut chunks = body.into_data_stream();
        while let Some(chunk) = chunks.next().await {
            let chunk = chunk.map_err(|_| StatusCode::BAD_REQUEST)?;
            let previous = received;
            received += chunk.len() as u64;
            if received > length {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
            file.write_all(&chunk)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if previous == 0 && received > 0 {
                println!("receiver first write {received}/{length}");
            }
        }
        if received != length {
            return Err(StatusCode::BAD_REQUEST);
        }
        file.rewind()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    };
    timeout(TIMEOUT, upload)
        .await
        .map_err(|_| StatusCode::REQUEST_TIMEOUT)??;
    println!("receiver upload complete {length}, starting scan");

    // Return the headers now. The response body owns the file and the receiver
    // slot until the scan finishes or the client drops the connection.
    Ok(Body::from_stream(stream::once(async move {
        let _permit = _permit;
        let mut digest = Sha256::new();
        let mut bytes = 0;
        let mut buffer = vec![0; CHUNK];
        loop {
            let count = file.read(&mut buffer).await?;
            if count == 0 {
                break;
            }
            digest.update(&buffer[..count]);
            bytes += count as u64;
            sleep(PACE).await;
        }
        let receipt = Receipt {
            bytes,
            sha256: format!("{:x}", digest.finalize()),
        };
        println!(
            "receiver scan complete {} {}",
            receipt.bytes, receipt.sha256
        );
        serde_json::to_vec(&receipt).map_err(io::Error::from)
    })))
}
