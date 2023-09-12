use crate::component::Component::{Ground, Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;
use crate::validation::StatusError::{Known, Multiple};
use crate::validation::{StatusError, Validation};
use std::cell::RefCell;
use crate::solvers::node_matrix_solver::NodeMatrixSolver;
use crate::solvers::node_step_solver::NodeStepSolver;
use crate::solvers::solver::Solver;
use crate::util::{create_basic_container, create_basic_supermesh_container, create_basic_supernode_container, create_mna_container, create_mna_container_2};
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
    let container = Container::from(setup);
    container.validate()?;
    Ok(String::from("Loaded Successfully"))
}

#[wasm_bindgen]
pub fn get_tools(container_js: JsValue) -> Result<String, StatusError> {
    let setup: ContainerSetup = serde_wasm_bindgen::from_value(container_js).unwrap();
    let mut c: Container = Container::from(setup);
    c.validate()?;
    c.create_nodes();
    c.create_super_nodes();
    let nodes: Vec<Vec<usize>> = c
        .nodes()
        .iter()
        .map(|x| x.upgrade().unwrap().members())
        .collect();

    Ok(serde_json::to_string(&nodes).unwrap())
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
                let mut solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
                let steps = solver.solve().unwrap();
                return Ok(serde_json::to_string(&steps).unwrap());
            } else {
                let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
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
pub fn return_solved_step_example() -> Result<String, JsValue> {
    let mut c: Container = create_mna_container();
    c.create_nodes();
    c.create_super_nodes();
    let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

    let steps = solver.solve()?;
    if let Ok(x) = serde_json::to_string(&steps) {
        return Ok(x);
    }
    Err(JsValue::from_str("Steps Errored out."))}

#[wasm_bindgen]
pub fn return_solved_matrix_example() -> Result<String, JsValue> {
    let mut c: Container = create_mna_container();
    c.create_nodes();
    c.create_super_nodes();
    let mut solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
    let steps = solver.solve()?;
    if let Ok(x) = serde_json::to_string(&steps) {
        return Ok(x);
    }
    Err(JsValue::from_str("Steps Errored out."))
}

#[wasm_bindgen]
pub fn test_wasm() -> String {
    "Hello from Rust!".to_string()
}

#[wasm_bindgen]
pub fn solve_mna_container() -> Result<Vec<JsValue>, JsValue> {
    let c: Container = create_mna_container();
    let mut solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
    let x = solver.solve()?;
    let js_steps: Vec<JsValue> = x.into_iter().map(JsValue::from).collect();
    Ok(js_steps)
}

#[wasm_bindgen]
pub fn solve_test_container(container_id: i32) -> Result<String, JsValue> {
    let c: Container = match container_id {
        0 => create_basic_container(),
        1 => create_basic_supernode_container(),
        2 => create_basic_supermesh_container(),
        3 => create_mna_container(),
        4 => create_mna_container_2(),
        _ => create_basic_container(),
    };
    let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
    let steps = solver.solve()?;
    serde_json::to_string(&steps).unwrap();
    if let Ok(x) = serde_json::to_string(&steps) {
        return Ok(x);
    }
    Err(JsValue::from_str("Steps Errored out."))
}

#[wasm_bindgen]
pub fn return_result(x: bool) -> Result<String, JsValue> {
    if x {
        Ok("Success".to_string())
    } else {
        Err(JsValue::from("Failure AHAHAHAHHA 🦞🦞🦞🦞".to_string()))
    }
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

impl From<ContainerSetup> for Container {
    fn from(setup: ContainerSetup) -> Container {
        let mut container = Container::new();
        for element in setup.elements {
            container.add_element_core(element);
        }
        container
    }
}
