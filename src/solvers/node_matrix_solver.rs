use crate::component::Component::{CurrentSrc, Resistor, VoltageSrc};
use crate::container::Container;
use crate::solvers::solver::{Solver, Step, SubStep};
use crate::util::PrettyPrint;
use nalgebra::{DMatrix, DVector};
use operations::math::{EquationMember, EquationRepr};
use operations::prelude::{Divide, Negate, Operation, Sum, Text, Value, Variable};
use std::cell::RefCell;
use std::rc::Rc;
use crate::elements::Element;
use crate::tools::{Tool, ToolType};

pub struct NodeMatrixSolver {
    a_matrix: DMatrix<Operation>,
    x_matrix: DVector<Operation>,
    z_matrix: DVector<Operation>,
}

impl Solver for NodeMatrixSolver {
    fn new(container: Rc<RefCell<Container>>) -> NodeMatrixSolver {
        container.borrow_mut().create_nodes();
        let n = container.borrow().nodes().len();

        // Node Count SuperNodes Must Replace their nodes with a single node
        let proper_nodes = get_calculation_nodes(container.clone());

        // let n = proper_nodes.len();

        let m = container // Source Count
            .borrow()
            .get_elements()
            .iter()
            .fold(0, |acc: usize, x: &Rc<Element>| match x.class {
                VoltageSrc => acc + 1,
                _ => acc,
            });

        // https://lpsa.swarthmore.edu/Systems/Electrical/mna/MNA3.html#B_matrix
        NodeMatrixSolver {
            a_matrix: form_a_matrix(container.clone(), n, m),
            x_matrix: form_x_vector(container.clone()),
            z_matrix: form_z_vector(container.clone()),
        }
    }

    /// Returns a string that represents the matrix equation to solve the circuit.
    fn solve(&mut self) -> Result<Vec<Step>, String> {
        let mut steps: Vec<Step> = Vec::new();

        let inverse_result: Option<DMatrix<f64>> = DMatrix::from_iterator(
            self.a_matrix.nrows(),
            self.a_matrix.ncols(),
            self.a_matrix.iter().map(|x| x.value()),
        )
        .try_inverse();

        let inverse: DMatrix<f64>;
        match inverse_result {
            Some(a) => {
                inverse = a;
            }
            None => {
                return Err(format!("Matrix is not invertible!\nThis might have something to do with sizing.\n{}\n", self.a_matrix.latex_string()));
            }
        }

        let z_vector: DVector<f64> = self
            .z_matrix
            .iter()
            .map(|x| x.value())
            .collect::<Vec<f64>>()
            .into();

        let mut result = inverse.clone() * z_vector;

        result
            .iter_mut()
            .for_each(|x| *x = (*x * 100.).round() / 100.);

        steps.push(Step {
            description: Some("Form matrices".to_string()),
            sub_steps: vec![
                SubStep {
                    description: Some("A Matrix".to_string()),
                    result: None,
                    operations: vec![Variable(Rc::new(self.a_matrix.clone()))],
                },
                SubStep {
                    description: Some("Z Matrix".to_string()),
                    result: None,
                    operations: vec![Variable(Rc::new(self.z_matrix.clone()))],
                },
                SubStep {
                    result: None,
                    description: Some("X Matrix".to_string()),
                    operations: vec![Variable(Rc::new(self.x_matrix.clone()))],
                },
                SubStep {
                    description: Some("Inverse A Matrix".to_string()),
                    result: None,
                    operations: vec![Variable(Rc::new(inverse.clone()))],
                },
                SubStep {
                    description: Some("Final Equation".to_string()),
                    result: None,
                    operations: vec![Text(format!(
                        "{} = {}^{{-1}} * {}",
                        self.x_matrix.equation_repr(),
                        self.a_matrix.equation_repr(),
                        self.z_matrix.equation_repr()
                    ))],
                },
            ],
            result: Some(Text(format!(
                "${} = {}$",
                self.x_matrix.equation_repr(),
                result.equation_repr()
            ))),
        });

        Ok(steps)
    }
}

fn form_a_matrix(container: Rc<RefCell<Container>>, n: usize, m: usize) -> DMatrix<Operation> {
    let mut a_matrix: DMatrix<Operation> = DMatrix::<Operation>::zeros(n + m, n + m);

    let g: DMatrix<Operation> = form_g_matrix(container.clone(), n);
    let b: DMatrix<Operation> = form_b_matrix(container.clone(), n, m);
    let c: DMatrix<Operation> = form_c_matrix(container.clone(), n, m);
    let d: DMatrix<Operation> = form_d_matrix(container.clone(), m);

    a_matrix.view_mut((0, 0), (n, n)).copy_from(&g);
    a_matrix.view_mut((0, n), (n, m)).copy_from(&b);
    a_matrix.view_mut((n, 0), (m, n)).copy_from(&c);
    a_matrix.view_mut((n, n), (m, m)).copy_from(&d);

    a_matrix
}

fn form_g_matrix(container: Rc<RefCell<Container>>, n: usize) -> DMatrix<Operation> {
    let mut matrix: DMatrix<Operation> = DMatrix::zeros(n, n);
    let mut nodes = container.borrow().nodes().clone();
    let _elements = container.borrow().get_elements().clone();

    nodes.sort_by(|a, b| a.upgrade().unwrap().id.cmp(&b.upgrade().unwrap().id));

    // assert_eq!(nodes.len(), n);



    // Form the diagonal
    for (i, tool) in nodes.iter().enumerate() {
        let equation_members: Vec<EquationRepr> = tool
            .upgrade()
            .unwrap()
            .members
            .iter()
            .filter(|x| x.upgrade().unwrap().class == Resistor)
            .map(|x| EquationRepr::from(x.upgrade().unwrap()))
            .collect();
        let set: Vec<Operation> = equation_members
            .into_iter()
            .map(|x| {
                Divide(
                    Some(Box::new(Value(1.0))),
                    Some(Box::new(Variable(Rc::new(x)))),
                )
            })
            .collect();

        matrix[(n - i - 1, n - i - 1)] = Sum(set);
    }

    // Form the off-diagonal
    // Find all resistors between two nodes
    for (i, tool) in nodes.iter().enumerate() {
        for (j, tool2) in nodes.iter().enumerate() {
            if i == j {
                continue;
            }
            let mut set: Vec<Operation> = Vec::new();
            for element in &tool.upgrade().unwrap().members {
                let element = element.upgrade().unwrap();
                if element.class != Resistor {
                    continue;
                }
                for element2 in tool2.upgrade().unwrap().members.clone() {
                    let element2 = element2.upgrade().unwrap();
                    if element2.class != Resistor {
                        continue;
                    }
                    if element.id == element2.id {
                        set.push(Negate(Some(Box::new(Divide(
                            Some(Box::new(Value(1.0))),
                            Some(Box::from(Variable(element.clone()))),
                        )))));
                    }
                }
            }
            matrix[(n - i - 1, n - j - 1)] = Sum(set);
        }
    }
    matrix
}

pub fn form_b_matrix(container: Rc<RefCell<Container>>, n: usize, m: usize) -> DMatrix<Operation> {
    let mut matrix: DMatrix<Operation> = DMatrix::zeros(n, m);

    for (i, tool) in container.borrow().nodes().iter().enumerate() {
        for (j, element) in container.borrow().get_voltage_sources().iter().enumerate() {
            if tool.upgrade().unwrap().contains(element) {
                if element
                    .upgrade()
                    .unwrap()
                    .positive
                    .contains(&tool.upgrade().unwrap().members[0].upgrade().unwrap().id)
                {
                    matrix[(n - i - 1, j)] = Value(-1.0);
                } else {
                    matrix[(n - i - 1, j)] = Value(1.0);
                }
            }
        }
    }

    matrix
}

pub(crate) fn form_c_matrix(
    container: Rc<RefCell<Container>>,
    n: usize,
    m: usize,
) -> DMatrix<Operation> {
    let matrix: DMatrix<Operation> = form_b_matrix(container.clone(), n, m);
    matrix.transpose()
}

fn form_d_matrix(_container: Rc<RefCell<Container>>, m: usize) -> DMatrix<Operation> {
    DMatrix::zeros(m, m)
}

fn get_calculation_nodes(container: Rc<RefCell<Container>>) -> Vec<Rc<Tool>> {
    let nodes: Vec<Rc<Tool>> = container.borrow().nodes().iter().map(|x| x.upgrade().unwrap()).collect();
    let binding = container.borrow();
    let super_nodes: Vec<Rc<Tool>> = binding.supernodes().iter().map(|x| x.upgrade().unwrap()).collect();
    let mut cleaned: Vec<Rc<Tool>> = nodes.into_iter().filter(|node| {
        for super_node in &super_nodes {
            let super_node_member_ids: Vec<usize> = super_node.members.iter().map(|x| x.upgrade().unwrap().id).collect();
            if node.members.iter().all(|y| super_node_member_ids.contains(&y.upgrade().unwrap().id)) {
                return false;
            }
        }
        true
    }).collect();
    cleaned.extend(super_nodes);
    cleaned
}

fn form_z_vector(container: Rc<RefCell<Container>>) -> DVector<Operation> {
    let mut z_vec: Vec<Operation> = Vec::new();

    // I Matrix
    // The balance of current flowing in the node.
    container.borrow().nodes().iter().for_each(|tool| {
        let mut set: Vec<Operation> = Vec::new();
        for element in &tool.upgrade().unwrap().members {
            let element = element.upgrade().unwrap();
            if element.class != CurrentSrc {
                continue;
            }
            set.push(Value(element.value));
        }
        if set.len() == 0 {
            z_vec.push(Value(0.0));
        } else {
            z_vec.push(Sum(set));
        }
    });

    // E Matrix
    // The value of the voltage source.
    container
        .borrow()
        .get_voltage_sources()
        .iter()
        .for_each(|source| {
            z_vec.push(Value(source.upgrade().unwrap().value));
        });

    DVector::from(z_vec)
}

fn form_x_vector(container: Rc<RefCell<Container>>) -> DVector<Operation> {
    let mut x_vec: Vec<Operation> = Vec::new();

    // V Matrix
    for tool in container.borrow().nodes() {
        x_vec.push(Variable(Rc::new(EquationRepr::new(
            format!("{}", tool.upgrade().unwrap().pretty_string()),
            0.0,
        ))));
    }

    // J Matrix
    for source in container.borrow().get_voltage_sources() {
        x_vec.push(Variable(Rc::new(EquationRepr::new(
            format!("{}", source.upgrade().unwrap().pretty_string()),
            0.0,
        ))));
    }

    DVector::from(x_vec)
}

#[cfg(test)]
mod tests {
    use crate::solvers::node_matrix_solver::{form_b_matrix, form_c_matrix, form_d_matrix, form_g_matrix, get_calculation_nodes, NodeMatrixSolver};
    use crate::solvers::solver::Solver;
    use crate::util::{create_mna_container, create_mna_container_2, PrettyPrint};
    use operations::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_node_solver() {
        let mut c = create_mna_container();
        c.create_nodes();
        let _solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));

        let mut c = create_mna_container_2();
        c.create_nodes();
        c.create_super_nodes();
        let mut solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));
        let steps = solver.solve();
    }

    #[test]
    fn test_a_matrix() {
        let expected = vec![
            vec!["1/R1", "", "", "-1", "0"],
            vec!["", "1/R2 + 1/R3", "-1/R2", "1", "0"],
            vec!["", "-1/R2", "1/R2", "0", "1"],
            vec!["-1", "1", "0", "0", "0"],
            vec!["0", "0", "1", "0", "0"],
        ];

        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c.clone())));

        assert_eq!(2., c.get_element_by_id(1).clone().value);
        assert_eq!(1. / 2., solver.a_matrix[(0, 0)].value());
        assert_eq!(
            solver.a_matrix[(0, 0)].value(),
            Divide(
                Some(Box::new(Value(1.0))),
                Some(Box::new(Variable(c.get_element_by_id(1).clone())))
            )
            .value()
        );
        for i in 0..5 {
            for j in 0..5 {
                assert_eq!(expected[i][j], solver.a_matrix[(i, j)].equation_repr());
            }
        }

        let expected = vec![
            vec!["1/R2 + 1/R3", "-1/R2", "-1"],
            vec!["-1/R2", "1/R1 + 1/R2", "1" ],
            vec!["-1", "1", "0"],
        ];


        let mut c = create_mna_container_2();
        c.create_nodes();
        let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c.clone())));

        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(expected[i][j], solver.a_matrix[(i, j)].equation_repr());
            }
        }
        assert_eq!(expected.len(), solver.a_matrix.nrows());

    }

    #[test]
    fn test_g_matrix() {
        let expected = vec![
            vec!["1/R1", "", ""],
            vec!["", "1/R2 + 1/R3", "-1/R2"],
            vec!["", "-1/R2", "1/R2"],
        ];

        let mut c = create_mna_container();
        c.create_nodes();
        let n = c.nodes().len();
        let matrix = form_g_matrix(Rc::new(RefCell::new(c)), n);

        for i in 0..n {
            for j in 0..n {
                assert_eq!(expected[i][j], matrix[(i, j)].equation_repr());
            }
        }
    }

    #[test]
    fn test_b_matrix() {
        let expected = vec![vec!["-1", "0"], vec!["1", "0"], vec!["0", "1"]];

        let mut c = create_mna_container();
        c.create_nodes();
        let n = c.nodes().len();
        let m = c.get_voltage_sources().len();
        let matrix = form_b_matrix(Rc::new(RefCell::new(c)), n, m);

        for i in 0..n {
            for j in 0..m {
                assert_eq!(expected[i][j], matrix[(i, j)].equation_repr());
            }
        }
    }

    #[test]
    fn test_c_matrix() {
        let expected = vec![vec!["-1", "1", "0"], vec!["0", "0", "1"]];

        let mut c = create_mna_container();
        c.create_nodes();
        let n = c.nodes().len();
        let m = c.get_voltage_sources().len();

        let c_matrix = form_c_matrix(Rc::new(RefCell::new(c)), n, m);

        for i in 0..m {
            for j in 0..n {
                assert_eq!(expected[i][j], c_matrix[(i, j)].equation_repr());
            }
        }
    }

    #[test]
    fn test_d_matrix() {
        let expected = vec![vec!["0", "0"], vec!["0", "0"]];

        let mut c = create_mna_container();
        c.create_nodes();
        let m = c.get_voltage_sources().len();

        let d_matrix = form_d_matrix(Rc::new(RefCell::new(c)), m);

        for i in 0..m {
            for j in 0..m {
                assert_eq!(expected[i][j], d_matrix[(i, j)].equation_repr());
            }
        }
    }

    #[test]
    fn test_x_matrix() {
        let expected = "\\begin{bmatrix}Node: 1\\\\Node: 2\\\\Node: 3\\\\SRC(V)4: 32 V\\\\SRC(V)5: 20 V\\\\\\end{bmatrix}";

        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));

        assert_eq!(solver.x_matrix.equation_repr(), expected);
    }

    #[test]
    fn test_z_matrix() {
        let expected = "\\begin{bmatrix}0\\\\0\\\\0\\\\32\\\\20\\\\\\end{bmatrix}";

        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeMatrixSolver = Solver::new(Rc::new(RefCell::new(c)));

        assert_eq!(solver.z_matrix.equation_repr(), expected);
    }
}
