use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Console;
    pub type ConsoleLogLimit;

    #[wasm_bindgen(js_namespace = cons)]
    pub fn new(lang: String, log_limit: ConsoleLogLimit) -> Console;
}
