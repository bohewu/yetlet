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

/// A remote executor is always explicit and only accepts same-origin paths.
#[derive(Debug, Clone)]
pub struct HttpExecutor {
    endpoint: String,
}

impl HttpExecutor {
    pub fn new(endpoint: &str) -> Result<Self, ActionError> {
        if !endpoint.starts_with('/')
            || endpoint.starts_with("//")
            || endpoint.contains(['?', '#', '\\', '\r', '\n'])
        {
            return Err(ActionError::InvalidEndpoint);
        }
        Ok(Self {
            endpoint: endpoint.to_owned(),
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[cfg(target_arch = "wasm32")]
impl<C> ActionExecutor<C> for HttpExecutor
where
    C: Command + Serialize,
    C::Response: serde::de::DeserializeOwned,
{
    async fn execute(&self, command: C) -> Result<C::Response, ActionError> {
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, Response};

        let body = serde_json::to_string(&command).map_err(|_| ActionError::InvalidResponse)?;
        let init = RequestInit::new();
        init.set_method("POST");
        init.set_body(&JsValue::from_str(&body));
        let request = Request::new_with_str_and_init(&self.endpoint, &init)
            .map_err(|_| ActionError::InvalidEndpoint)?;
        request
            .headers()
            .set("Content-Type", "application/json")
            .map_err(|_| ActionError::Network)?;
        request
            .headers()
            .set("Accept", "application/json")
            .map_err(|_| ActionError::Network)?;
        let window = web_sys::window().ok_or(ActionError::Network)?;
        let response_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_| ActionError::Network)?;
        let response: Response = response_value
            .dyn_into()
            .map_err(|_| ActionError::InvalidResponse)?;
        let status = response.status();
        let response_text = JsFuture::from(response.text().map_err(|_| ActionError::Network)?)
            .await
            .map_err(|_| ActionError::Network)?
            .as_string()
            .ok_or(ActionError::InvalidResponse)?;
        if !(200..300).contains(&status) {
            let code = serde_json::from_str::<ApiErrorEnvelope>(&response_text)
                .map(|envelope| envelope.error.code)
                .unwrap_or_else(|_| "remote_error".to_owned());
            return Err(ActionError::Remote { status, code });
        }
        serde_json::from_str(&response_text).map_err(|_| ActionError::InvalidResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionError, ActionExecutor, Command, HttpExecutor, LocalExecutor};
    // Neither type implements serde traits: local actions require no transport.
    #[derive(Debug)]
    struct Increment(u64);

    #[derive(Debug, PartialEq, Eq)]
    struct Updated(u64);

    impl Command for Increment {
        type Response = Updated;
    }

    #[tokio::test]
    async fn local_executor_maps_typed_input_and_output() {
        let executor = LocalExecutor::new(|increment: Increment| Ok(Updated(increment.0 + 1)));
        assert_eq!(executor.execute(Increment(41)).await, Ok(Updated(42)));
    }

    #[tokio::test]
    async fn local_executor_propagates_typed_error() {
        let executor = LocalExecutor::new(|_: Increment| {
            Err::<Updated, _>(ActionError::Validation("invalid_number"))
        });
        assert_eq!(
            executor.execute(Increment(0)).await,
            Err(ActionError::Validation("invalid_number"))
        );
    }

    #[test]
    fn http_executor_requires_explicit_same_origin_path() {
        assert!(HttpExecutor::new("/api/todos").is_ok());
        assert!(HttpExecutor::new("https://example.org/api").is_err());
        assert!(HttpExecutor::new("//example.org/api").is_err());
        assert!(HttpExecutor::new("/api/todos?token=secret").is_err());
    }
}
