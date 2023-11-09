use crate::container::Container;
use crate::solvers::solver::{Solver, Step};
use crate::validation::StatusError;
use std::cell::RefCell;
use std::rc::Rc;

// TODO MeshStepSolver
#[allow(dead_code)]
pub struct MeshStepSolver {
    container: Rc<RefCell<Container>>,
}

impl Solver for MeshStepSolver {
    fn new(container: Rc<RefCell<Container>>) -> Self {
        MeshStepSolver { container }
    }

    fn solve(&mut self) -> Result<Vec<Step>, StatusError> {
        todo!()
    }
}
