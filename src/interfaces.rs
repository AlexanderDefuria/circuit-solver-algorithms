use crate::component::Component::{Ground, Resistor, VoltageSrc};
use crate::container::Container;
use crate::controller::Controller;
use crate::elements::Element;
use crate::operations::{OpMethod, Operation};
use crate::validation::StatusError::{Known, Multiple};
use crate::validation::{StatusError, Validation};
use std::cell::RefCell;

use crate::solvers::SolverType::Matrix;
use crate::solvers::{NodeSolver, Solver};
use crate::util::{create_basic_container, create_mna_container};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[derive(Serialize, Deserialize)]
pub struct ContainerSetup {
    pub elements: Vec<Element>,
    pub operations: Vec<OpMethod>,
    // pub method: OpMethod,
    // pub target: Option<usize>,
}

#[wasm_bindgen]
pub fn load_wasm_container(js: JsValue) -> Result<String, StatusError> {
    // This JsValue is a ContainerInterface and also needs operations
    let setup: ContainerSetup = serde_wasm_bindgen::from_value(js).unwrap();
    if setup.elements.len() == 0 {
        return Ok(String::from("No elements"));
    }

    let controller: Controller = setup.into();
    controller.container.validate()?;

    Ok(String::from("Loaded Successfully"))
}

#[wasm_bindgen]
pub fn return_create_basic_container() -> String {
    let c: Container = create_basic_container();
    let x = ContainerSetup {
        elements: c
            .get_elements()
            .iter()
            .map(|y| {
                let x = y.clone();
                Element::new(
                    x.class.clone(),
                    x.value,
                    x.positive.clone(),
                    x.negative.clone(),
                )
            })
            .collect(),
        operations: vec![],
    };

    serde_json::to_string(&x).unwrap()
}

#[wasm_bindgen]
pub fn return_create_mna_container() -> String {
    let c: Container = create_mna_container();
    let x = ContainerSetup {
        elements: c
            .get_elements()
            .iter()
            .map(|y| {
                let x = y.clone();
                Element::new(
                    x.class.clone(),
                    x.value,
                    x.positive.clone(),
                    x.negative.clone(),
                )
            })
            .collect(),
        operations: vec![],
    };

    serde_json::to_string(&x).unwrap()
}

#[wasm_bindgen]
pub fn test_wasm() -> String {
    "Hello from Rust!".to_string()
}

#[wasm_bindgen]
pub fn solve_mna_container() -> String {
    let c: Container = create_mna_container();
    let solver: NodeSolver = Solver::new(Rc::new(RefCell::new(c)), Matrix);
    solver.solve_matrix().unwrap()
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

impl From<ContainerSetup> for Controller {
    fn from(setup: ContainerSetup) -> Controller {
        let container: Container = setup.elements.into();
        let mut operations: Vec<Operation> = vec![];
        for op in setup.operations {
            operations.push(op.into());
        }
        let status = container.validate();
        Controller {
            container: Rc::from(container),
            operations,
            status,
        }
    }
}

pub fn simplify() {}

#[wasm_bindgen_test]
fn test_container_wasm() {
    let c: Vec<Element> = vec![];
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    let y: Vec<Element> = serde_wasm_bindgen::from_value(x).unwrap();
    assert_eq!(c, y);
}

#[wasm_bindgen_test]
fn test_load() {
    let c = ContainerSetup {
        elements: vec![],
        operations: vec![],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(load_wasm_container(x).unwrap(), "No elements");

    let c = ContainerSetup {
        elements: vec![Element::new(Ground, 0., vec![], vec![])],
        operations: vec![],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert!(load_wasm_container(x).is_err());

    let c = ContainerSetup {
        elements: vec![
            Element::new(Ground, 0., vec![1], vec![]),
            Element::new(Ground, 0., vec![0], vec![]),
        ],
        operations: vec![],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        load_wasm_container(x),
        Err(Multiple(vec![
            Known("No Sources".to_string()),
            Known("Multiple Grounds".to_string())
        ]))
    );

    let c = ContainerSetup {
        elements: vec![
            Element::new(VoltageSrc, 1.0, vec![2, 3], vec![1]),
            Element::new(Resistor, 1.0, vec![0], vec![2]),
            Element::new(Resistor, 1.0, vec![1], vec![0, 3]),
            Element::new(Ground, 0., vec![0, 2], vec![]),
        ],
        operations: vec![],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        Ok("Loaded Successfully".to_string()),
        load_wasm_container(x)
    );
}

#[cfg(test)]
mod tests {
    use crate::container::Container;
    use crate::interfaces::{load_wasm_container, return_create_basic_container};
    use crate::util::create_basic_container;
    use wasm_bindgen::JsValue;

    #[test]
    fn test() {
        println!("{:?}", return_create_basic_container());
        assert!(true);
    }
}
