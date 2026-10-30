use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::HtmlCanvasElement;
use yetlet_browser::wgpu::{Batch, browser};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = __yt076Fault)]
    fn report_fault(generation: u32, category: &str);
}
fn application_fault(generation: u32, category: &str) {
    report_fault(generation, category);
}

/// Application binding; synthetic records and the global bridge belong here.
#[wasm_bindgen]
pub struct SceneOwner {
    inner: browser::SceneOwner,
}
#[wasm_bindgen]
impl SceneOwner {
    #[wasm_bindgen(constructor)]
    pub fn new(generation: u32) -> Self {
        let data = Batch::new(crate::instances(crate::RECORDS).expect("bounded synthetic records"))
            .expect("validated default records");
        Self {
            inner: browser::SceneOwner::new(generation, data, application_fault),
        }
    }
    pub async fn initialize(&self, canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        self.inner.initialize(canvas).await
    }
    pub fn resize(
        &self,
        width: f64,
        height: f64,
        dpr: f64,
        connected: bool,
    ) -> Result<bool, JsValue> {
        self.inner.resize(width, height, dpr, connected)
    }
    pub fn configurations(&self) -> u32 {
        self.inner.configurations()
    }
    pub fn draws(&self) -> u32 {
        self.inner.draws()
    }
    pub fn device_limit(&self) -> u32 {
        self.inner.device_limit()
    }
    pub fn buffer_bytes(&self) -> u32 {
        self.inner.buffer_bytes()
    }
    pub fn replace(&self, bytes: &[u8]) -> Result<(), JsValue> {
        self.inner.replace(bytes)
    }
    pub fn update(&self, start: u32, bytes: &[u8]) -> Result<(), JsValue> {
        self.inner.update(start, bytes)
    }
    pub fn cpu_bytes(&self) -> u32 {
        self.inner.cpu_bytes()
    }
    pub fn data_version(&self) -> u32 {
        self.inner.data_version()
    }
    pub fn pick(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        dpr: f64,
    ) -> Result<Vec<u32>, JsValue> {
        self.inner.pick(x, y, width, height, dpr)
    }
    pub fn accepts_pick(&self, generation: u32, version: u32, id: u32) -> bool {
        self.inner.accepts_pick(generation, version, id)
    }
    pub fn cpu_capacity_bytes(&self) -> u32 {
        self.inner.cpu_capacity_bytes()
    }
    pub fn metrics(&self) -> Vec<f64> {
        self.inner.metrics()
    }
    pub async fn completion(&self) -> Result<(), JsValue> {
        self.inner.completion().await
    }
    pub fn draw(&self) -> Result<(), JsValue> {
        self.inner.draw()
    }
    /// True only on the first owner retirement; free the wrapper after futures settle.
    pub fn dispose(&self) -> bool {
        self.inner.dispose()
    }
}
impl Default for SceneOwner {
    fn default() -> Self {
        Self::new(0)
    }
}
