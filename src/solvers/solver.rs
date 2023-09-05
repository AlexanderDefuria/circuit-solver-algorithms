use crate::container::Container;
use operations::prelude::*;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::cell::RefCell;
use std::error::Error;
use std::fmt::Display;
use std::rc::Rc;
use wasm_bindgen::JsValue;

/// This will take a container and solve it using the given method.
/// KCL and KVL will be used to solve the circuit.

pub trait Solver {
    fn new(container: Rc<RefCell<Container>>) -> Self;
    fn solve(&self) -> Result<Vec<Step>, String>;
}

pub struct Step {
    pub description: Option<String>,
    pub sub_steps: Vec<SubStep>,
    pub result: Option<Operation>,
}

#[derive(Clone)]
pub struct SubStep {
    pub description: Option<String>,
    pub operations: Vec<Operation>,
}

impl Step {
    pub fn new(label: &str) -> Self {
        Step {
            description: Some(label.to_string()),
            sub_steps: vec![],
            result: None,
        }
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn get_steps(&self) -> Vec<SubStep> {
        self.sub_steps.clone()
    }
}

impl SubStep {
    pub fn new(label: &str) -> Self {
        SubStep {
            description: Some(label.to_string()),
            operations: vec![],
        }
    }

    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn get_steps(&self) -> Vec<Operation> {
        self.operations.clone()
    }
}

impl Serialize for Step {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Step", 2)?;
        state.serialize_field("description", &self.description())?;
        state.serialize_field(
            "sub_steps",
            &self.get_steps(),
        )?;
        state.end()
    }
}

impl Serialize for SubStep {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("SubStep", 2)?;
        state.serialize_field("stepInstruction", &self.description())?;
        state.serialize_field(
            "operations",
            &self.get_steps()
                .into_iter()
                .map(|x| x.latex_string())
                .collect::<Vec<String>>(),

        )?;
        state.end()
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output: String = self.description.clone().unwrap_or_else(|| "".to_string());
        for i in self.sub_steps.clone() {
            output.push_str(&format!("{}\n", serde_json::to_string(&i).unwrap()));
        }

        write!(f, "{}", output)
    }
}

impl From<Step> for JsValue {
    fn from(step: Step) -> Self {
        let mut output: String = step.description.unwrap_or_else(|| "".to_string());
        output.push_str("\n");
        for i in step.sub_steps {
            output.push_str(&format!("{}\n", serde_json::to_string(&i).unwrap()));
        }

        JsValue::from_str(&output)
    }
}

#[cfg(test)]
mod tests {
    use crate::container::Container;
    use crate::solvers::node_matrix_solver::NodeMatrixSolver;
    use crate::solvers::node_step_solver::NodeStepSolver;
    use crate::solvers::solver::{Solver, Step};
    use crate::util::create_mna_container;
    use operations::math::EquationMember;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// This should mirror the Step struct in src/solvers/solver.rs
    struct StepResult {
        label: String,
        sub_steps: Option<Vec<String>>,
    }

    #[test]
    fn test_solve_steps() {
        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

        for i in solver.solve().unwrap() {
            println!("{}", i);
        }
    }
}
