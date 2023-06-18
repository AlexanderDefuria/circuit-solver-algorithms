use crate::container::Container;
use crate::elements::Element;
use crate::validation::{StatusError, Validation};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn load_wasm_container(x: JsValue) -> Result<String, StatusError> {
    /// This JsValue is a ContainerInterface and also needs operations
    let y: Vec<Element> = serde_wasm_bindgen::from_value(x).unwrap();
    if y.len() == 0 {
        return Ok(String::from("No elements"));
    }

    let container: Container = y.into();
    container.validate()?;

    Ok(String::from("Loaded Successfully"))
}

impl From<Vec<Element>> for Container {
    fn from(wasm: Vec<Element>) -> Container {
        let mut container = Container::new();
        for element in wasm {
            container.add_element_core(element);
        }
        container
    }
}

pub fn simplify() {}
pub fn serialize() {}
pub fn deserialize() {}
