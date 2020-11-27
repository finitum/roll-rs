use roll_lib::{roll_inline, Parser};
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;

// to build:  wasm-pack build --target web

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn hello_world() -> String {
    "Hello World".to_string()
}

#[wasm_bindgen(start)]
pub fn init_console() {
    console_error_panic_hook::set_once();
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ObjType {
    JsRoll,
    JsRolls,
}

#[derive(Serialize, Deserialize)]
pub struct JsRoll {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub vals: Vec<u64>,
    pub total: i64,
    pub sides: u64,
    pub dpos: u64,
}

#[derive(Serialize, Deserialize)]
pub struct JsRolls {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub rolls: Vec<JsRoll>,
    pub total: f64,
}

#[wasm_bindgen]
pub fn roll_dice_short(s: &str, advanced: bool) -> Result<String, JsValue> {
    roll_inline(s, advanced).map_err(|s| JsValue::from(String::from("\n".to_string() + &s)))
}

#[wasm_bindgen]
pub fn roll_dice(s: &str, advanced: bool) -> Result<JsValue, JsValue> {
    let mut p = Parser::new(s);
    p.advanced = advanced;

    let ast = p.parse().map_err(|e| JsValue::from(e.to_string()))?;

    let mut rolls = Vec::new();
    let res = ast.interp(&mut rolls).unwrap();

    let rolls: Vec<JsRoll> = rolls
        .into_iter()
        .map(|(dpos, r)| JsRoll {
            obj_type: ObjType::JsRoll,
            vals: r.vals,
            total: r.total,
            sides: r.sides.get(),
            dpos,
        })
        .collect();

    let res = JsRolls {
        obj_type: ObjType::JsRolls,
        total: res.into(),
        rolls,
    };

    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_hello_world() {
        assert_eq!(hello_world(), "Hello World")
    }

    #[wasm_bindgen_test]
    fn smoke_roll_dice() {
        let res = roll_dice("(2d8 + 5) * 12 // 3 + 2d%kh", true).unwrap();
        let de: JsRolls = serde_wasm_bindgen::from_value(res).unwrap();
        assert_eq!(ObjType::JsRolls, de.obj_type);
        assert_eq!(2, de.rolls.len());

        for roll in &de.rolls {
            assert_eq!(ObjType::JsRoll, roll.obj_type);
        }

        assert_eq!(8, de.rolls[0].sides);
        assert_eq!(2, de.rolls[0].vals.len());

        assert_eq!(100, de.rolls[1].sides);
        assert_eq!(1, de.rolls[1].vals.len());
    }

    #[wasm_bindgen_test]
    fn smoke_roll_short() {
        let res = roll_dice_short("4d8", false).unwrap();
        assert!(res.contains("4d8"))
    }
}
