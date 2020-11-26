use wasm_bindgen::prelude::*;
use roll_lib::Parser;

// to build:  wasm-pack build --target web

#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello World".to_string()
}

#[wasm_bindgen]
pub fn roll_dice(s: &str) -> String {
    let mut p = Parser::new(s);

    let ast = match p.parse() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("{}", e);
            return "error".to_string();
        }
    };

    let mut rolls = Vec::new();
    let res = ast.interp(&mut rolls).unwrap();

    format!("{}, {:?}", res, rolls)
}

#[cfg(test)]
mod test {
    use crate::hello_world;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello World")
    }
}
