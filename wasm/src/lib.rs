use wasm_bindgen::prelude::*;
use roll_lib::{Parser};
use serde::{Serialize};

// to build:  wasm-pack build --target web

#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello World".to_string()
}

#[wasm_bindgen(start)]
pub fn init_console() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize)]
pub enum ObjType {
    JsRoll,
    JsRolls
}

#[derive(Serialize)]
pub struct JsRoll {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub vals: Vec<u64>,
    pub total: i64,
    pub sides: u64,
    pub dpos: u64,
}

#[derive(Serialize)]
pub struct JsRolls {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub rolls: Vec<JsRoll>,
    pub total: f64
}

#[wasm_bindgen]
pub fn roll_dice(s: &str) -> Result<JsValue, JsValue> {
    let mut p = Parser::new(s);

    let ast = match p.parse() {
        Ok(i) => i,
        Err(e) => {
            return Err(JsValue::from(e.to_string()))
        }
    };

    let mut rolls = Vec::new();
    let res = ast.interp(&mut rolls).unwrap();

    let rolls: Vec<JsRoll> = rolls.into_iter().map(|(dpos, r)| JsRoll{
        obj_type: ObjType::JsRoll,
        vals: r.vals,
        total: r.total,
        sides: r.sides.get(),
        dpos
    }).collect();

    let res = JsRolls{
        obj_type: ObjType::JsRolls,
        total: res.into(),
        rolls,
    };

    Ok(serde_wasm_bindgen::to_value(&res)?)
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
