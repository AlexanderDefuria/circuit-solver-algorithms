use crate::component::Component::{Ground, Resistor, VoltageSrc};
use crate::container::Container;
use crate::controller::Controller;
use crate::elements::Element;
use crate::validation::StatusError::{Known, Multiple};
use crate::validation::{StatusError, Validation};
use std::cell::RefCell;

use crate::solvers::node_matrix_solver::NodeMatrixSolver;
use crate::solvers::node_step_solver::NodeStepSolver;
use crate::solvers::solver::{Solver, Step};
use crate::util::{create_basic_container, create_mna_container};
use js_sys::Array;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[derive(Serialize, Deserialize)]
pub struct ContainerSetup {
    pub elements: Vec<Element>,
}

/// This can be used as a test to see if the container is being loaded in properly.
#[wasm_bindgen]
pub fn load_wasm_container(js: JsValue) -> Result<String, StatusError> {
    // This JsValue is a ContainerInterface and also needs operations
    let setup: ContainerSetup = serde_wasm_bindgen::from_value(js).unwrap();

    let controller: Controller = setup.into();
    controller.container.validate()?;

    Ok(String::from("Loaded Successfully"))
}

#[wasm_bindgen]

pub fn solve(matrix: bool, nodal: bool, container_js: JsValue) -> Result<String, StatusError> {
    let setup: ContainerSetup = serde_wasm_bindgen::from_value(container_js).unwrap();
    let mut c: Container = Container::from(setup);
    c.validate()?;
    match nodal {
        true => {
            c.create_nodes();
            c.create_super_nodes();
            if matrix {
                let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
                let steps = solver.solve().unwrap();
                return Ok(serde_json::to_string(&steps).unwrap());
            } else {
                let solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
                let steps = solver.solve().unwrap();
                return Ok(serde_json::to_string(&steps).unwrap());
            }
        }
        false => {
            c.create_meshes();
            c.create_super_meshes();
            if matrix {
                return Err(Known(
                    "Matrix Solver not implemented for meshes".to_string(),
                ));
            } else {
                return Err(Known("Step Solver not implemented for meshes".to_string()));
            }
        }
    }
}

#[wasm_bindgen]
pub fn return_solved_step_example() -> String {
    let mut c: Container = create_mna_container();
    c.create_nodes();
    c.create_super_nodes();
    let solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

    let mut steps = solver.solve().unwrap();
    serde_json::to_string(&steps).unwrap()
}

#[wasm_bindgen_test]
pub fn test_serialize_steps() {
    let mut c: Container = create_mna_container();
    c.create_nodes();
    c.create_super_nodes();
    let solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

    let mut steps = solver.solve();
    match steps {
        Ok(x) => {
            let result = serde_json::to_string(&x).unwrap();
            assert!(result.len() > 1);
            // assert_eq!(result, "Some String".to_string());
        }
        Err(_) => {
            assert!(false);
        }
    }
}

#[wasm_bindgen]
pub fn return_solved_matrix_example() -> String {
    let mut c: Container = create_mna_container();
    c.create_nodes();
    c.create_super_nodes();
    let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
    let steps = solver.solve().unwrap();
    serde_json::to_string(&steps).unwrap()
}

#[wasm_bindgen]
pub fn test_wasm() -> String {
    "Hello from Rust!".to_string()
}

#[wasm_bindgen]
pub fn solve_mna_container() -> Vec<JsValue> {
    let c: Container = create_mna_container();
    let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
    let x = solver.solve().unwrap();
    x.into_iter().map(JsValue::from).collect()
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
        let status = container.validate();
        Controller {
            container: Rc::from(container),
            // operations,
            status,
        }
    }
}

impl From<ContainerSetup> for Container {
    fn from(setup: ContainerSetup) -> Container {
        let mut container = Container::new();
        for element in setup.elements {
            container.add_element_core(element);
        }
        container
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
    let c = ContainerSetup { elements: vec![] };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(load_wasm_container(x).unwrap(), "No Sources");

    let c = ContainerSetup {
        elements: vec![Element::new(Ground, 0., vec![], vec![])],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert!(load_wasm_container(x).is_err());

    let c = ContainerSetup {
        elements: vec![
            Element::new(Ground, 0., vec![1], vec![]),
            Element::new(Ground, 0., vec![0], vec![]),
        ],
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
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        Ok("Loaded Successfully".to_string()),
        load_wasm_container(x)
    );
}
