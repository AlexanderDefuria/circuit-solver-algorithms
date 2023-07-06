use crate::component::Component::{CurrentSrc, VoltageSrc};
use crate::container::Container;
use std::cell::RefCell;
use std::rc::Rc;

/// This will take a container and solve it using the given method.
/// KCL and KVL will be used to solve the circuit.

pub trait Solver {
    fn new(container: Rc<RefCell<Container>>) -> Self;
    fn solve(&mut self) -> Result<(), String>;
    fn latex(&self) -> String;
}

struct NodeSolver {
    container: Rc<RefCell<Container>>,
    a_matrix: ndarray::Array2<f64>,
    x_matrix: ndarray::Array2<f64>,
    z_matrix: ndarray::Array2<f64>,
}

impl Solver for NodeSolver {
    fn new(mut container: Rc<RefCell<Container>>) -> NodeSolver {
        container.borrow_mut().create_nodes();
        let n = container.borrow().nodes().len();
        let m = container
            .borrow()
            .get_elements()
            .iter()
            .fold(0, |acc: usize, x| match x.class {
                VoltageSrc => acc + 1,
                CurrentSrc => acc + 1,
                _ => acc,
            });

        // https://lpsa.swarthmore.edu/Systems/Electrical/mna/MNA3.html#B_matrix

        NodeSolver {
            container,
            a_matrix: form_a_matrix(n, m),
            x_matrix: form_x_matrix(n, m),
            z_matrix: form_z_matrix(n, m),
        }
    }

    fn solve(&mut self) -> Result<(), String> {
        todo!()
    }

    fn latex(&self) -> String {
        todo!()
    }
}

fn solve(container: Rc<RefCell<Container>>) -> Result<Rc<RefCell<Container>>, String> {
    let mut solver = NodeSolver::new(container);

    Ok(solver.container)
}

fn form_a_matrix(n: usize, m: usize) -> ndarray::Array2<f64> {
    let matrix = ndarray::Array2::<f64>::zeros((n + m, n + m));

    matrix
}

fn form_g_matrix(n: usize, m: usize) -> ndarray::Array2<f64> {
    let matrix = ndarray::Array2::<f64>::zeros((n, n));
    matrix
}

fn form_b_matrix(n: usize, m: usize) -> ndarray::Array2<f64> {
    let matrix = ndarray::Array2::<f64>::zeros((n, m));
    matrix
}

fn form_c_matrix(n: usize, m: usize) -> ndarray::Array2<f64> {
    let matrix = ndarray::Array2::<f64>::zeros((m, n));
    matrix
}

fn form_d_matrix(n: usize, m: usize) -> ndarray::Array2<f64> {
    let matrix = ndarray::Array2::<f64>::zeros((m, m));
    matrix
}

fn form_z_matrix(n: usize, m: usize) -> ndarray::Array2<f64> {
    todo!()
}

fn form_x_matrix(n: usize, m: usize) -> ndarray::Array2<f64> {
    todo!()
}
