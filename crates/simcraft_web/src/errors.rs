use std::fmt;

use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::JsValue;

#[derive(Serialize)]
pub struct CustomJsError {
    pub error: String,
    pub message: String,
}

pub(crate) fn wasm_error<E: fmt::Debug + fmt::Display>(err: E) -> JsValue {
    let js_error = CustomJsError {
        error: format!("{:?}", err),
        message: err.to_string(),
    };

    to_value(&js_error).unwrap_or_else(|_| {
        JsValue::from_str("{\"error\": \"Unknown\", \"message\": \"Failed to convert error\"}")
    })
}
