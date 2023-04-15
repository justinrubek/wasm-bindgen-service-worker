use wasm_bindgen::JsValue;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not in a service worker")]
    NotInServiceWorker,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for JsValue {
    fn from(e: Error) -> Self {
        JsValue::from_str(&e.to_string())
    }
}
