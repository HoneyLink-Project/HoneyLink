//! WASM bindings for web integration

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmSdk {
    inner: crate::sdk::HoneyLinkSdk,
}

#[wasm_bindgen]
impl WasmSdk {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::sdk::HoneyLinkSdk::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), JsValue> {
        self.inner
            .initialize()
            .await
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    #[wasm_bindgen(js_name = isInitialized)]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_initialized()
    }
}
