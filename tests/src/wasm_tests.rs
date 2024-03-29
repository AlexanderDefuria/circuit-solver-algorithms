use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

use circuit_solver_algorithms::component::Component::{Ground, Resistor, VoltageSrc};
use circuit_solver_algorithms::container::Container;
use circuit_solver_algorithms::elements::Element;
use circuit_solver_algorithms::interfaces::{get_tools, load_wasm_container, ContainerSetup, solve};
use circuit_solver_algorithms::solvers::node_step_solver::NodeStepSolver;
use circuit_solver_algorithms::solvers::solver::{Solver, Step};
use circuit_solver_algorithms::util::create_mna_container;
use circuit_solver_algorithms::validation::Status::Valid;
use circuit_solver_algorithms::validation::StatusError::{Known, Multiple};
use circuit_solver_algorithms::validation::{StatusError, Validation};

use crate::compare_test_case::InputCaseSerde;



pub fn test_solver_select() {
    let mut c: Container = create_mna_container();
    c.create_nodes().unwrap();
    c.create_super_nodes();
    let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

    let steps = solver.solve();
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

#[wasm_bindgen_test]
pub fn test_get_tools() {
    let raw_json: &str = include_str!("../data/mna_container/input.json");
    let setup: InputCaseSerde = serde_json::from_str(raw_json).unwrap();
    let mut container: Container = setup.container.into();
    assert_eq!(container.validate().unwrap(), Valid);

    container.create_nodes().unwrap();
    let nodes = container.nodes();
    assert_eq!(nodes.len(), 3);

    let nodes: Result<String, StatusError> =
        get_tools(serde_wasm_bindgen::to_value(&container).unwrap());
    assert_eq!(nodes.unwrap(), "[[5,2],[2,4,3],[1,4]]")
}

#[wasm_bindgen_test]
pub fn test_serialize_steps() {
    let mut c: Container = create_mna_container();
    c.create_nodes().unwrap();
    c.create_super_nodes();
    let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

    let steps = solver.solve();
    match steps {
        Ok(x) => {
            let result = serde_json::to_string(&x).unwrap();
            assert!(result.len() > 1);
        }
        Err(_) => {
            assert!(false);
        }
    }
}

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
    assert_eq!(
        load_wasm_container(x),
        Err(Multiple(vec![
            Known("No Sources".parse().unwrap()),
            Known("Multiple Grounds".parse().unwrap()),
        ]))
    );

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
            Known("Multiple Grounds".to_string()),
        ]))
    );

    let c = ContainerSetup {
        elements: vec![
            Element::new(Ground, 0., vec![1, 3], vec![]),
            Element::new(VoltageSrc, 1.0, vec![3, 0], vec![2]),
            Element::new(Resistor, 1.0, vec![1], vec![3]),
            Element::new(Resistor, 1.0, vec![2], vec![1, 0]),
        ],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        load_wasm_container(x),
        Ok("Loaded Successfully".to_string())
    );
}

#[wasm_bindgen_test]
fn test_error() {
    let c = ContainerSetup {
        elements: vec![
            Element::new(VoltageSrc, 1.0, vec![3, 0], vec![2]),
            Element::new(Resistor, 1.0, vec![1], vec![3]),
            Element::new(Resistor, 1.0, vec![2], vec![1, 0]),
        ],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        solve(false, true, x),
        // Err("No Sources".to_string())
        Err("{\"errors\": [\"Known Issue: Element cannot be connected to itself SRC(V)0: 1 V\", \"Known Issue: Element cannot be connected to itself R1: 1 Ω\", \"Known Issue: Element cannot be connected to itself R2: 1 Ω\", \"Known Issue: Multiple Grounds\"]}".to_string())
    );
}

pub fn cleanup_include_str(input: String) -> String {
    let mut output: String = input.replace("\n", "");
    output = output.replace(" ", "");
    output
}

#[wasm_bindgen_test]
fn test_matrix_invert_error() {
    let c: ContainerSetup = ContainerSetup {
        elements: vec![
            Element::new(Ground, 0.0, vec![], vec![1,3]),
            Element::new(Resistor, 10.0, vec![2], vec![0, 3]),
            Element::new(Resistor, 10.0, vec![4], vec![1]),
            Element::new(Resistor, 10.0, vec![0, 1], vec![4]),
            Element::new(VoltageSrc, 10.0, vec![3], vec![2]),
        ],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        solve(false, true, x),
        Err("{\"errors\": [\"Known Issue... Matrix is not square: \\begin{bmatrix}0 & 0.1\\\\0 & 1\\\\0 & 0\\\\\\end{bmatrix}\"]}".to_string())
    );

    let c: ContainerSetup = ContainerSetup {
        elements: vec![
            Element::new(Ground, 0.0, vec![], vec![1,4]),
            Element::new(Resistor, 10.0, vec![2], vec![0, 4]),
            Element::new(VoltageSrc, 10.0, vec![4], vec![1]),
            Element::new(Resistor, 10.0, vec![2], vec![0, 1]),
        ],
    };
    let x: JsValue = serde_wasm_bindgen::to_value(&c).unwrap();
    assert_eq!(
        solve(false, true, x),
        Err("{\"errors\": [\"Known Issue... Matrix is not square: \\begin{bmatrix}0 & 0.1\\\\0 & 1\\\\0 & 0\\\\\\end{bmatrix}\"]}".to_string())
    );
}

#[wasm_bindgen_test]
pub fn test_malformed_ids_from_netlist() {
    let json: JsValue = JsValue::from_str(r#"
    {
 "elements": [
  {
   "value": 10,
   "id": 1,
   "positive": [],
   "negative": [
    2
   ],
   "class": "Resistor"
  },
  {
   "value": 10,
   "id": 2,
   "positive": [
    4
   ],
   "negative": [
    0
   ],
   "class": "Resistor"
  },
  {
   "value": 10,
   "id": 3,
   "positive": [
    4
   ],
   "negative": [
    0
   ],
   "class": "VoltageSrc"
  },
  {
   "value": 0,
   "id": 0,
   "positive": [],
   "negative": [
    2,
    4
   ],
   "class": "Ground"
  },
  {
   "value": 10,
   "id": 5,
   "positive": [
    1
   ],
   "negative": [
    2,
    3
   ],
   "class": "Resistor"
  }
 ]
}"#);
    assert_eq!(
        solve(false, true, json),
        Err("{\"errors\": [\"Known Issue: Element cannot be connected to itself R1: 10 Ω\", \"Known Issue: Element cannot be connected to itself R2: 10 Ω\", \"Known Issue: Element cannot be connected to itself SRC(V)3: 10 V\", \"Known Issue: Multiple Grounds\"]}".to_string())
    );
}