//! YT-022: reconstruct bounded data and select already compiled local code.
//! This is not closure/stack serialization, authentication or authorization.

use serde::{Deserialize, Serialize};

pub const MAX_ENCODED_BYTES: usize = 256;
const PROTOCOL_VERSION: u16 = 1;
const HANDLER_VERSION: u16 = 1;
const MAX_COUNT: i32 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    TooLarge,
    Malformed,
    ProtocolVersion,
    UnknownHandler,
    HandlerVersion,
    InvalidCapture,
    InvalidResult,
}

/// Explicit, finite code choices; no runtime registration or code loading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handler {
    Add,
    Subtract,
}

impl Handler {
    pub fn id(self) -> &'static str {
        match self {
            Self::Add => "counter.add",
            Self::Subtract => "counter.subtract",
        }
    }

    pub fn apply(self, capture: Capture) -> Result<i32, Error> {
        capture.validate()?;
        let next = match self {
            Self::Add => capture.count + i32::from(capture.step),
            Self::Subtract => capture.count - i32::from(capture.step),
        };
        if !(0..=MAX_COUNT).contains(&next) {
            return Err(Error::InvalidResult);
        }
        Ok(next)
    }
}

/// Only deliberately selected client-safe numeric fields, with no handles,
/// credentials, strings, arbitrary JSON, closures or server-owned objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    count: i32,
    step: u16,
}

impl Capture {
    pub fn new(count: i32, step: u16) -> Result<Self, Error> {
        let capture = Self { count, step };
        capture.validate()?;
        Ok(capture)
    }

    fn validate(self) -> Result<(), Error> {
        if !(0..=MAX_COUNT).contains(&self.count) || !(1..=1000).contains(&self.step) {
            return Err(Error::InvalidCapture);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    protocol_version: u16,
    handler_id: String,
    handler_version: u16,
    #[serde(deserialize_with = "deserialize_capture")]
    capture: Capture,
}

// serde structs normally also accept positional sequences; the wire capture
// deliberately permits only an object with named, allowlisted fields.
fn deserialize_capture<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Capture, D::Error> {
    struct CaptureObject;
    impl<'de> serde::de::Visitor<'de> for CaptureObject {
        type Value = Capture;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a capture object")
        }

        fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Capture, A::Error> {
            Capture::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_map(CaptureObject)
}

impl Descriptor {
    pub fn new(handler: Handler, capture: Capture) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            handler_id: handler.id().to_owned(),
            handler_version: HANDLER_VERSION,
            capture,
        }
    }

    fn resolve(&self) -> Result<Handler, Error> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(Error::ProtocolVersion);
        }
        let handler = match self.handler_id.as_str() {
            "counter.add" => Handler::Add,
            "counter.subtract" => Handler::Subtract,
            _ => return Err(Error::UnknownHandler),
        };
        if self.handler_version != HANDLER_VERSION {
            return Err(Error::HandlerVersion);
        }
        self.capture.validate()?;
        Ok(handler)
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        self.resolve()?;
        let bytes = serde_json::to_vec(self).map_err(|_| Error::Malformed)?;
        if bytes.len() > MAX_ENCODED_BYTES {
            return Err(Error::TooLarge);
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        // Bound the complete wire document before serde allocates its ID string.
        if bytes.len() > MAX_ENCODED_BYTES {
            return Err(Error::TooLarge);
        }
        if !bytes.trim_ascii_start().starts_with(b"{") {
            return Err(Error::Malformed);
        }
        let descriptor: Self = serde_json::from_slice(bytes).map_err(|_| Error::Malformed)?;
        descriptor.resolve()?;
        Ok(descriptor)
    }

    pub fn dispatch(&self) -> Result<i32, Error> {
        self.resolve()?.apply(self.capture)
    }
}

#[cfg(test)]
mod tests;
