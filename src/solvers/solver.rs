use crate::container::Container;
use operations::prelude::*;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use wasm_bindgen::JsValue;

/// This will take a container and solve it using the given method.
/// KCL and KVL will be used to solve the circuit.

pub trait Solver {
    fn new(container: Rc<RefCell<Container>>) -> Self;
    fn solve(&mut self) -> Result<Vec<Step>, String>;
}

pub struct Step {
    pub description: Option<String>,
    pub result: Option<Operation>,
    pub sub_steps: Vec<SubStep>,
}

#[derive(Clone)]
pub struct SubStep {
    pub description: Option<String>,
    pub result: Option<Operation>,
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

    pub fn new_with_steps(label: &str, steps: Vec<SubStep>) -> Self {
        Step {
            description: Some(label.to_string()),
            result: None,
            sub_steps: steps,
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
            result: None,
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
        let mut state: <S>::SerializeStruct;
        if &self.result == &None {
            state = serializer.serialize_struct("Step", 2)?;
        } else {
            state = serializer.serialize_struct("Step", 3)?;
            state.serialize_field("result", &latex_serialize(self.result.clone().unwrap()))?;
        }
        state.serialize_field("description", &self.description())?;
        state.serialize_field("sub_steps", &self.get_steps())?;
        state.end()
    }
}

impl Serialize for SubStep {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state: <S>::SerializeStruct;
        if &self.result == &None {
            state = serializer.serialize_struct("SubStep", 2)?;
        } else {
            state = serializer.serialize_struct("SubStep", 3)?;
            state.serialize_field("result", &latex_serialize(self.result.clone().unwrap()))?;
        }
        state.serialize_field("stepInstruction", &self.description())?;
        state.serialize_field(
            "operations",
            &self
                .get_steps()
                .into_iter()
                .map(|x| latex_serialize(x))
                .collect::<Vec<String>>(),
        )?;
        state.end()
    }
}

fn latex_serialize(op: Operation) -> String {
    let content = &*op.latex_string();
    format!("${}$", content.replace("$$", "$ $"))
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output: String = self.description.clone().unwrap_or_else(|| "".to_string());

        if let Some(result) = &self.result {
            output.push_str(&format!("\nResult: {:?}", result));
        }
        if self.sub_steps.len() > 0 {
            output.push_str("\nSub Steps:");
        }
        for i in self.sub_steps.clone() {
            output.push_str(&format!("\n\t{}\n", i));
        }

        write!(f, "{}", output)
    }
}

impl Display for SubStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output: String = String::from("Step: ");
        output.push_str(&self.description.clone().unwrap_or_else(|| "".to_string()));
        if let Some(result) = &self.result {
            output.push_str(&format!("\n\tResult: {:?}", result));
        }
        if self.operations.len() > 0 {
            output.push_str("\n");
            for i in self.operations.clone() {
                output.push_str(&format!("\t\t{:?}\n", i));
            }
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
        let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));

        for i in solver.solve().unwrap() {
            println!("---- Step ---- \n{}", i);
        }
    }
    #[test]
    fn test_solve_matrix() {
        let mut c = create_mna_container();
        c.create_nodes();
        let mut solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));

        for i in solver.solve().unwrap() {
            println!("---- Step ---- \n{}", i);
        }
    }
}
