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

    #[test]
    fn test_solve_matrix() {
        let mut c: Container = create_mna_container();
        let expected_steps: Vec<StepResult> = vec![
            StepResult {
                label: "A Matrix".to_string(),
                sub_steps: Some(vec![String::from("\\begin{bmatrix}1/R1 &  &  & -1 & 0 & \\\\ & 1/R2 + 1/R3 & -1/R2 & 1 & 0 & \\\\ & -1/R2 & 1/R2 & 0 & 1 & \\\\-1 & 1 & 0 & 0 & 0 & \\\\0 & 0 & 1 & 0 & 0 & \\\\\\end{bmatrix}")]),
            },
            StepResult {
                label: "Z Matrix".to_string(),
                sub_steps: Some(vec![String::from("\\begin{bmatrix}0 \\\\0 \\\\0 \\\\32 \\\\20 \\\\\\end{bmatrix}")]),
            },
            StepResult {
                label: "X Matrix".to_string(),
                sub_steps: Some(vec![String::from("\\begin{bmatrix}Node: 1 \\\\Node: 2 \\\\Node: 3 \\\\SRC(V)4: 32 V \\\\SRC(V)5: 20 V \\\\\\end{bmatrix}")]),
            },
            StepResult {
                label: "Inverse A Matrix".to_string(),
                sub_steps: None,
            },
            StepResult {
                label: "Final Equation".to_string(),
                sub_steps: Some(vec![String::from("\\begin{bmatrix}Node: 1\\\\Node: 2\\\\Node: 3\\\\SRC(V)4: 32 V\\\\SRC(V)5: 20 V\\\\\\end{bmatrix} = \\begin{bmatrix}1/R1 &  &  & -1 & 0 & \\\\ & 1/R2 + 1/R3 & -1/R2 & 1 & 0 & \\\\ & -1/R2 & 1/R2 & 0 & 1 & \\\\-1 & 1 & 0 & 0 & 0 & \\\\0 & 0 & 1 & 0 & 0 & \\\\\\end{bmatrix}^{-1} * \\begin{bmatrix}0\\\\0\\\\0\\\\32\\\\20\\\\\\end{bmatrix}")]),
            },
            StepResult {
                label: "In theory we are solved.".to_string(),
                sub_steps: Some(vec![String::from("\\begin{bmatrix}Node: 1\\\\Node: 2\\\\Node: 3\\\\SRC(V)4: 32 V\\\\SRC(V)5: 20 V\\\\\\end{bmatrix} = \\begin{bmatrix}-8\\\\24\\\\20\\\\-4\\\\1\\\\\\end{bmatrix}")]),
            },
        ];

        c.create_nodes();
        c.create_super_nodes();

        let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
        let steps = solver.solve().unwrap();
        assert_eq!(steps.len(), expected_steps.len());

        for (i, (step, expected)) in steps.iter().zip(expected_steps.iter()).enumerate() {
            assert_eq!(step.get_label(), expected.label);
            if let Some(sub_steps) = &expected.sub_steps {
                println!("Step #{}:", i);
                for (j, (sub_step, expected_sub_step)) in step
                    .get_steps()
                    .unwrap()
                    .iter()
                    .zip(sub_steps.iter())
                    .enumerate()
                {
                    println!(
                        "   {} \n = {}\n",
                        sub_step.equation_repr(),
                        *expected_sub_step
                    );
                    assert_eq!(
                        sub_step.equation_repr().replace(" ", ""),
                        *expected_sub_step.replace(" ", "")
                    );
                }
            }
        }
    }
}
