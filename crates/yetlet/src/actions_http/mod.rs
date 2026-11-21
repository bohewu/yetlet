//! Target-neutral HTTP action errors and wire envelopes.
//!
//! These types do not execute commands or install an HTTP server. Application
//! adapters explicitly map their domain failures to the HTTP contract; local
//! executors can retain their own error types without this capability.

use serde::Serialize;

/// Error vocabulary used by HTTP action adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionError {
    Validation(&'static str),
    NotFound,
    CapacityReached,
    RevisionOverflow,
    InvalidEndpoint,
    Network,
    InvalidResponse,
    Internal,
    Remote { status: u16, code: String },
}

impl ActionError {
    pub fn code(&self) -> &str {
        match self {
            Self::Validation(code) => code,
            Self::NotFound => "not_found",
            Self::CapacityReached => "capacity_reached",
            Self::RevisionOverflow => "revision_overflow",
            Self::InvalidEndpoint => "invalid_endpoint",
            Self::Network => "network_error",
            Self::InvalidResponse => "invalid_response",
            Self::Internal => "internal_error",
            Self::Remote { code, .. } => code,
        }
    }
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "action failed: {}", self.code())
    }
}
impl std::error::Error for ActionError {}

/// A small, explicit envelope for non-successful HTTP actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::{ApiErrorBody, ApiErrorEnvelope};

    #[test]
    fn error_envelope_preserves_the_existing_http_wire_shape() {
        let envelope = ApiErrorEnvelope {
            error: ApiErrorBody {
                code: "invalid_title".to_owned(),
                message: "Invalid \"title\"".to_owned(),
            },
        };
        let wire = r#"{"error":{"code":"invalid_title","message":"Invalid \"title\""}}"#;
        assert_eq!(serde_json::to_string(&envelope).unwrap(), wire);
        assert_eq!(
            serde_json::from_str::<ApiErrorEnvelope>(wire).unwrap(),
            envelope
        );
    }
}
