//! Semantic commands and explicitly injectable action executors.
//!
//! UI events never cross this abstraction. Browser code turns an event into a
//! typed command and chooses a local or HTTP executor; no persistent circuit.

use serde::Serialize;

/// Local-only commands do not need serialization. HTTP adds that bound
/// explicitly at its transport boundary.
pub trait Command {
    type Response;
}

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

/// A static-dispatch command boundary, independent of DOM event types.
/// Browser-local closures/futures are intentionally not required to be Send.
#[allow(async_fn_in_trait)]
pub trait ActionExecutor<C: Command> {
    async fn execute(&self, command: C) -> Result<C::Response, ActionError>;
}

/// Local actions can use any injected pure or stateful command handler.
/// The caller controls its state lifetime; no framework session is created.
#[derive(Clone)]
pub struct LocalExecutor<F> {
    handler: F,
}

impl<F> LocalExecutor<F> {
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<C: Command, F: Fn(C) -> Result<C::Response, ActionError>> ActionExecutor<C>
    for LocalExecutor<F>
{
    async fn execute(&self, command: C) -> Result<C::Response, ActionError> {
        (self.handler)(command)
    }
}

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
