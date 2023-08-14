use crate::container::Container;
use operations::prelude::*;
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
}
