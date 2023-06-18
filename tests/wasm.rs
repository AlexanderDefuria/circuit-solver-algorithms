use circuit_solver_algorithms::components::Component::{Ground, Resistor, VoltageSrc};
use circuit_solver_algorithms::elements::Element;
use circuit_solver_algorithms::interfaces::load_wasm_container;
use circuit_solver_algorithms::validation::StatusError::{Known, Multiple};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

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
