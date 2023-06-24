use crate::container::Container;
use crate::elements::Element;
use crate::validation::{StatusError, Validation};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::component::Component::{Ground, Resistor, VoltageSrc};
use crate::validation::StatusError::{Known, Multiple};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;



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



#[wasm_bindgen_test]
fn test_container_wasm() {
    let c: Vec<Element> = vec![];
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    let y: Vec<Element> = serde_wasm_bindgen::from_value(x).unwrap();
    assert_eq!(c, y);
}

#[wasm_bindgen_test]
fn test_load() {
    let c: Vec<Element> = vec![];
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(load_wasm_container(x).unwrap(), "No elements");

    let c: Vec<Element> = vec![Element::new(Ground, 0., vec![], vec![])];
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert!(load_wasm_container(x).is_err());

    let c: Vec<Element> = vec![
        Element::new(Ground, 0., vec![1], vec![]),
        Element::new(Ground, 0., vec![0], vec![]),
    ];
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        load_wasm_container(x),
        Err(Multiple(vec![
            Known("No Sources".to_string()),
            Known("Multiple Grounds".to_string())
        ]))
    );

    let c: Vec<Element> = vec![
        Element::new(VoltageSrc, 1.0, vec![2, 3], vec![1]),
        Element::new(Resistor, 1.0, vec![0], vec![2]),
        Element::new(Resistor, 1.0, vec![1], vec![0, 3]),
        Element::new(Ground, 0., vec![0, 2], vec![]),
    ];
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        Ok("Loaded Successfully".to_string()),
        load_wasm_container(x)
    );
}
