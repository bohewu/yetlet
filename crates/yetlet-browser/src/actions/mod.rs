//! Browser HTTP executor using shared Yetlet action contracts.
#[cfg(target_arch = "wasm32")]
use serde::Serialize;
pub use yetlet::actions::{
    ActionError, ActionExecutor, ApiErrorBody, ApiErrorEnvelope, Command, LocalExecutor,
};
/// A remote executor is always explicit and only accepts same-origin paths.
#[derive(Debug, Clone)]
pub struct HttpExecutor {
    endpoint: String,
}

impl HttpExecutor {
    pub fn new(endpoint: &str) -> Result<Self, ActionError> {
        if !endpoint.starts_with('/')
            || endpoint.starts_with("//")
            || endpoint.contains(['?', '#', '\\'])
            // URL parsing strips ASCII TAB/CR/LF and can turn /<TAB>/host
            // into an off-origin //host URL unless controls are rejected.
            || endpoint.bytes().any(|byte| byte.is_ascii_control())
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
        assert_eq!(
            HttpExecutor::new("/api/todos").unwrap().endpoint(),
            "/api/todos"
        );
        assert!(HttpExecutor::new("/api/items/v2").is_ok());
        for endpoint in [
            "https://example.org/api",
            "//example.org/api",
            "/api/todos?token=secret",
            "/api/todos#fragment",
            "/api\\commands",
            "/\t/attacker.example/api",
            "/\r/attacker.example/api",
            "/\n/attacker.example/api",
            "/\0/attacker.example/api",
            "/\u{7f}/attacker.example/api",
            "/api/\tcommands",
        ] {
            assert_eq!(
                HttpExecutor::new(endpoint).unwrap_err(),
                ActionError::InvalidEndpoint,
                "unexpectedly accepted {endpoint:?}",
            );
        }
    }
}
