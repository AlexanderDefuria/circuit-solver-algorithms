use std::cell::RefCell;
use std::rc::Rc;
use circuit_solver_algorithms::container::Container;
use circuit_solver_algorithms::interfaces::ContainerSetup;
use wasm_bindgen_test::wasm_bindgen_test;
use circuit_solver_algorithms::solvers::node_step_solver::NodeStepSolver;
use circuit_solver_algorithms::solvers::solver::{Solver, Step};
use circuit_solver_algorithms::validation::Status::Valid;
use circuit_solver_algorithms::validation::Validation;

#[wasm_bindgen_test]
fn test_validateable_containers() {
    let raw_json: &str = include_str!("./data/case_1/input.json");
    let setup: ContainerSetup = serde_json::from_str(raw_json).unwrap();
    let mut container: Container = setup.into();
    assert_eq!(container.validate().unwrap(), Valid);

    container.create_nodes();
    container.create_super_nodes();
    let solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(container)));
    let steps: Vec<Step> = solver.solve().unwrap();
    let steps_string: String = serde_json::to_string(&steps).unwrap();
    let mut expected: &str = include_str!("./data/case_1/result.json");
    assert_eq!(expected.replace("\n", ""), steps_string, "Steps are not matching")

}

