use crate::container::Container;
use crate::solvers::solver::{Solver, Step};
use std::cell::RefCell;
use std::rc::Rc;

pub struct MeshMatrixSolver {
    container: Rc<RefCell<Container>>,
}

impl Solver for MeshMatrixSolver {
    fn new(container: Rc<RefCell<Container>>) -> Self {
        MeshMatrixSolver { container }
    }

    fn solve(&self) -> Result<Vec<Step>, String> {
        todo!()
    }
}
