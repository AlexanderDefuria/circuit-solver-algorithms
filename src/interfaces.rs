use crate::components::Component;
use crate::components::Component::Ground;
use crate::container::Container;
use crate::elements::Element;
use crate::util::PrettyString;
use crate::validation::{StatusError, Validation};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn load_wasm_container(x: JsValue) -> Result<String, StatusError> {
    let y: ContainerWasm = serde_wasm_bindgen::from_value(x).unwrap();
    if y.elements.len() == 0 {
        return Ok(String::from("No elements"));
    }

    let container: Container = y.into();
    container.validate()?;

    Ok(String::from("Loaded"))
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ContainerWasm {
    pub elements: Vec<Element>,
    pub ground: usize,
}

impl From<ContainerWasm> for Container {
    fn from(wasm: ContainerWasm) -> Container {
        let mut container = Container::new();
        for element in wasm.elements {
            container.add_element_core(element);
        }
        container
    }
}

pub fn simplify() {}
pub fn serialize() {}
pub fn deserialize() {}
