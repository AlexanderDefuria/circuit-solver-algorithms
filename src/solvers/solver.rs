use crate::container::Container;
use operations::prelude::*;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use wasm_bindgen::JsValue;

/// This will take a container and solve it using the given method.
/// KCL and KVL will be used to solve the circuit.

pub trait Solver {
    fn new(container: Rc<RefCell<Container>>) -> Self;
    fn solve(&self) -> Result<Vec<Step>, String>;
}

pub enum SolverTool {
    Node,
    Mesh,
}

#[derive(Serialize, Deserialize)]
pub enum SolverMethod {
    Step,
    Matrix,
}

pub struct Step {
    pub label: String,
    pub sub_steps: Option<Vec<Operation>>,
}

impl Step {
    pub fn new(label: &str) -> Self {
        Step {
            label: label.to_string(),
            sub_steps: None,
        }
    }

    pub fn get_label(&self) -> String {
        self.label.clone()
    }

    pub fn get_steps(&self) -> Option<Vec<Operation>> {
        self.sub_steps.clone()
    }
}

impl Serialize for Step {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Step", 2)?;
        state.serialize_field("label", &self.get_label())?;
        let steps = self.get_steps().unwrap_or_else(|| vec![]);
        state.serialize_field(
            "sub_steps",
            &steps
                .into_iter()
                .map(|x| x.latex_string())
                .collect::<Vec<String>>(),
        )?;
        state.end()
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!("{}\n", self.label);
        if let Some(sub_steps) = &self.sub_steps {
            for i in sub_steps {
                s.push_str(&format!("{}\n", i.latex_string()));
            }
        }

        write!(f, "{}", s)
    }
}

impl From<Step> for JsValue {
    fn from(step: Step) -> Self {
        let mut output: String = step.label;
        output.push_str("\n");
        for i in step.sub_steps.unwrap_or_else(|| vec![]) {
            output.push_str(&format!("{}\n", i.latex_string()));
        }

        JsValue::from_str(&output)
    }
}

#[cfg(test)]
mod tests {
    use crate::solvers::node_matrix_solver::NodeMatrixSolver;
    use crate::solvers::node_step_solver::NodeStepSolver;
    use crate::solvers::solver::Solver;
    use crate::util::create_mna_container;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_solve_steps() {
        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

        for i in solver.solve().unwrap() {
            println!("{}", i);
        }
    }

    #[test]
    fn test_solve_matrix() {
        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
        for i in solver.solve().unwrap() {
            println!("{}", i);
        }
    }
}
