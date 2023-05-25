use crate::components::Component;
use crate::elements::Element;
use crate::validation::Validation;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn load(x: JsValue) -> String {
    let mut y: ContainerWasm = serde_wasm_bindgen::from_value(x).unwrap();
    if y.elements.len() == 0 {
        return String::from("No elements");
    }
    y.elements[0].clean().pretty_string().to_string()
}

#[derive(Serialize, Deserialize)]
pub struct ContainerWasm {
    elements: Vec<Element>,
    ground: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ElementWasm {
    id: usize,
    value: f64,
    class: Component,
    positive: Vec<usize>, // Link to other elements
    negative: Vec<usize>,
}

pub fn simplify() {}
pub fn serialize() {}
pub fn deserialize() {}
