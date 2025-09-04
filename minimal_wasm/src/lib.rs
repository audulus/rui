use wasm_bindgen::prelude::*;

// Import the `console.log` function from the `console` module
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();
    
    console_log!("Hello from Rust and WebAssembly!");
    
    // Get the document
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    // Create a new div element
    let div = document.create_element("div").unwrap();
    div.set_inner_html("<h2>Hello from WASM!</h2><p>This is a minimal Rust WebAssembly example.</p>");
    
    // Add some styling
    let div_html = div.dyn_ref::<web_sys::HtmlElement>().unwrap();
    div_html.style().set_property("background-color", "#e8f5e8").unwrap();
    div_html.style().set_property("padding", "20px").unwrap();
    div_html.style().set_property("margin", "20px").unwrap();
    div_html.style().set_property("border-radius", "8px").unwrap();
    div_html.style().set_property("border", "2px solid #4caf50").unwrap();
    
    // Add it to the body
    document.body().unwrap().append_child(&div).unwrap();
    
    console_log!("Successfully added content to the page!");
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}!", name);
}