use crate::component::Component::{CurrentSrc, Ground, Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;
use crate::math::{matrix_to_latex, EquationRepr, MathOp, EquationMember};
use crate::tools::Tool;
use crate::util::PrettyPrint;
use ndarray::{s, ArrayBase, Ix2, OwnedRepr};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// This will take a container and solve it using the given method.
/// KCL and KVL will be used to solve the circuit.

pub trait Solver {
    fn new(container: Rc<RefCell<Container>>, solve_for: SolverType) -> Self;
    fn solve_matrix(&self) -> Result<String, String>;
    fn solve_steps(&self) -> Result<Vec<String>, String>;
}

pub enum SolverType {
    Matrix,
    Step,
}

pub struct NodeSolver {
    solve_for: SolverType,
    container: Rc<RefCell<Container>>,
    a_matrix: ndarray::Array2<MathOp>,
    x_matrix: ndarray::Array2<MathOp>,
    z_matrix: ndarray::Array2<MathOp>,
}

impl Solver for NodeSolver {
    fn new(container: Rc<RefCell<Container>>, solve_for: SolverType) -> NodeSolver {
        container.borrow_mut().create_nodes();
        let n = container.borrow().nodes().len(); // Node Count
        let m = container // Source Count
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
            solve_for: SolverType::Matrix,
            container: container.clone(),
            a_matrix: form_a_matrix(container.clone(), n, m),
            x_matrix: form_x_matrix(container.clone(), n, m),
            z_matrix: form_z_matrix(container.clone(), n, m),
        }
    }

    /// Returns a string that represents the matrix equation to solve the circuit.
    fn solve_matrix(&self) -> Result<String, String> {
        let inverse_a_matrix: ndarray::Array2<MathOp> = self.a_matrix.clone();
        // solve::inverse(&mut inverse_a_matrix).unwrap();

        // Wrap in matrix
        // [x] = [A]^-1 * [z]
        Ok(format!(
            "{} = {}^{{-1}} * {}",
            matrix_to_latex(self.x_matrix.clone()),
            matrix_to_latex(inverse_a_matrix),
            matrix_to_latex(self.z_matrix.clone())
        ))
    }

    /// Returns a vector of strings that represent the steps to solve the circuit.
    fn solve_steps(&self) -> Result<Vec<String>, String> {
        let node_pairs: Vec<(usize, usize, Rc<Element>)> = self.container.borrow().get_all_node_pairs();

        let mut steps: Vec<String> = Vec::new();
        steps.push("Steps to solve the circuit:".to_string());

        // Step 1 Declare
        let mut step: String = String::new();
        step.push_str("Voltage Sources have 0 resistance.\n");
        node_pairs.iter().for_each(|(node1, node2, element)| {
            if element.class == VoltageSrc {
                step.push_str(
                    format!(
                        "V_{{{node1}, {node2}}} = current from node {node1} to node {node2}\n",
                        node1 = node1,
                        node2 = node2
                    )
                        .as_str(),
                );
            }
        });
        steps.append(&mut vec![step]);

        // Step 2 Find Voltages
        // Step 2.1 Declare intent
        let mut step: String = String::new();
        node_pairs.iter().for_each(|(node1, node2, element)| {
            if element.class == VoltageSrc {
                step.push_str(
                    format!(
                        "V_{{{node1}, {node2}}} = voltage from node {node1} to node {node2}\n",
                        node1 = node1,
                        node2 = node2
                    )
                        .as_str(),
                );
            }
        });
        steps.append(&mut vec![step]);

        // Step 2.2 Find voltages
        let mut sub_steps: Vec<MathOp> = Vec::new();
        // Step 2.2.1 Find all resistors going between nodes including ground
        let resistor_node_pairs: Vec<&(usize, usize, Rc<Element>)> = node_pairs
            .iter()
            .filter(|(node1, node2, element)| element.class == Resistor)
            .collect::<Vec<&(usize, usize, Rc<Element>)>>();
        resistor_node_pairs.iter().for_each(|(node1, node2, element)| {
            let mut tool2: MathOp = MathOp::None(Rc::new(0.0));
            let mut tool1: MathOp = MathOp::None(Rc::new(0.0));
            let mut id_1 = *node1;
            if *node1 != 0 {
                id_1 -= 1;
            }

            let mut id_2 = *node2;
            if *node2 != 0 {
                id_2 -= 1;
            }

            if *node1 == 0 {
                tool1 = MathOp::None(Rc::new(0.0));
            } else {
                tool1 = MathOp::None(self.container.borrow().get_tool_by_id(id_1).clone());
            }

            if *node2 == 0 {
                tool2 = MathOp::None(Rc::new(0.0));
            } else {
                tool2 = MathOp::None(self.container.borrow().get_tool_by_id(id_2).clone());
            }

            tool2 = MathOp::Negate(Rc::new(tool2));

            sub_steps.push(MathOp::Negate(Rc::new(MathOp::Divide(
                Rc::new(MathOp::Collect(Rc::new(MathOp::Sum(vec![tool1, tool2])))),
                Rc::new(MathOp::None(Rc::new(element.value)))
            ))));
        });

        steps.append(
            &mut vec![
                format!("Find current through each resistor:\n").to_string(),
                format!("{:?}\n", MathOp::Sum(sub_steps).latex_string()).to_string(),
            ]
        );

        sub_steps = Vec::new();
        // Step 2.2.2 Find all voltage sources going between nodes including ground
        let voltage_src_node_pairs: Vec<&(usize, usize, Rc<Element>)> = node_pairs
            .iter()
            .filter(|(node1, node2, element)| element.class == VoltageSrc)
            .collect::<Vec<&(usize, usize, Rc<Element>)>>();
        voltage_src_node_pairs.iter().for_each(|(node1, node2, element)| {
        let mut tool2: MathOp = MathOp::None(Rc::new(0.0));
            let mut tool1: MathOp = MathOp::None(Rc::new(0.0));
            let mut id_1 = *node1;
            if *node1 != 0 {
                id_1 -= 1;
            }

            let mut id_2 = *node2;
            if *node2 != 0 {
                id_2 -= 1;
            }

            if *node1 == 0 {
                tool1 = MathOp::None(Rc::new(0.0));
            } else {
                tool1 = MathOp::None(self.container.borrow().get_tool_by_id(id_1).clone());
            }

            if *node2 == 0 {
                tool2 = MathOp::None(Rc::new(0.0));
            } else {
                tool2 = MathOp::None(self.container.borrow().get_tool_by_id(id_2).clone());
            }

            tool2 = MathOp::Negate(Rc::new(tool2));

            sub_steps.push(
                MathOp::Equal(
                    Rc::new(MathOp::Collect(Rc::new(MathOp::Sum(vec![tool1, tool2])))),
                    Rc::new(MathOp::None(Rc::new(element.value)))
            ));
        });


        steps.append(&mut vec![format!("Deal with voltage sources").to_string()]);
        steps.append(sub_steps.iter().map(|x| format!("{:?}\n", x.latex_string())).collect::<Vec<String>>().as_mut());

        println!("resistor_node_pairs: {:?}", resistor_node_pairs);

        Ok(steps)
    }
}

fn form_a_matrix(container: Rc<RefCell<Container>>, n: usize, m: usize) -> ndarray::Array2<MathOp> {
    let mut matrix: ArrayBase<OwnedRepr<MathOp>, Ix2> =
        ndarray::Array2::<MathOp>::zeros((n + m, n + m));

    let g: ndarray::Array2<MathOp> = form_g_matrix(container.clone(), n);
    let b: ndarray::Array2<MathOp> = form_b_matrix(container.clone(), n, m);
    let c: ndarray::Array2<MathOp> = form_c_matrix(container.clone(), n, m);
    let d: ndarray::Array2<MathOp> = form_d_matrix(container.clone(), m);

    matrix.slice_mut(s![0..n, 0..n]).assign(&g);
    matrix.slice_mut(s![0..n, n..n + m]).assign(&b);
    matrix.slice_mut(s![n..n + m, 0..n]).assign(&c);
    matrix.slice_mut(s![n..n + m, n..n + m]).assign(&d);

    matrix
}

fn form_g_matrix(container: Rc<RefCell<Container>>, n: usize) -> ndarray::Array2<MathOp> {
    let mut matrix: ArrayBase<OwnedRepr<MathOp>, Ix2> = ndarray::Array2::<MathOp>::zeros((n, n));
    let mut nodes = container.borrow().nodes().clone();
    let _elements = container.borrow().get_elements().clone();

    nodes.sort_by(|a, b| a.upgrade().unwrap().id.cmp(&b.upgrade().unwrap().id));

    assert_eq!(nodes.len(), n);

    // Form the diagonal
    for (i, tool) in nodes.iter().enumerate() {
        let equation_members: Vec<EquationRepr> = tool
            .upgrade()
            .unwrap()
            .members
            .iter()
            .filter(|x| x.upgrade().unwrap().class == Resistor)
            .map(|x| EquationRepr::from(x.upgrade().unwrap().into()))
            .collect();
        let set: Vec<MathOp> = equation_members
            .into_iter()
            .map(|x| MathOp::Inverse(Rc::new(MathOp::None(Rc::new(x)))))
            .collect();

        matrix[[n - i - 1, n - i - 1]] = MathOp::Sum(set);
    }

    // Form the off-diagonal
    // Find all resistors between two nodes
    for (i, tool) in nodes.iter().enumerate() {
        for (j, tool2) in nodes.iter().enumerate() {
            if i == j {
                continue;
            }
            let mut set: Vec<MathOp> = Vec::new();
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
                        set.push(MathOp::Negate(Rc::new(MathOp::Inverse(element.clone()))));
                    }
                }
            }
            matrix[[n - i - 1, n - j - 1]] = MathOp::Sum(set);
        }
    }
    matrix
}

fn form_b_matrix(container: Rc<RefCell<Container>>, n: usize, m: usize) -> ndarray::Array2<MathOp> {
    let mut matrix: ArrayBase<OwnedRepr<MathOp>, Ix2> = ndarray::Array2::<MathOp>::zeros((n, m));

    for (i, tool) in container.borrow().nodes().iter().enumerate() {
        for (j, element) in container.borrow().get_voltage_sources().iter().enumerate() {
            if tool.upgrade().unwrap().contains(element) {
                if element
                    .upgrade()
                    .unwrap()
                    .positive
                    .contains(&tool.upgrade().unwrap().members[0].upgrade().unwrap().id)
                {
                    matrix[[n - i - 1, j]] = MathOp::None(Rc::new(-1.0));
                } else {
                    matrix[[n - i - 1, j]] = MathOp::None(Rc::new(1.0));
                }
            }
        }
    }

    matrix
}

fn form_c_matrix(container: Rc<RefCell<Container>>, n: usize, m: usize) -> ndarray::Array2<MathOp> {
    let mut matrix = form_b_matrix(container.clone(), n, m);
    matrix.swap_axes(0, 1);
    matrix
}

fn form_d_matrix(_container: Rc<RefCell<Container>>, m: usize) -> ndarray::Array2<MathOp> {
    let matrix: ArrayBase<OwnedRepr<MathOp>, Ix2> = ndarray::Array2::<MathOp>::zeros((m, m));
    matrix
}

fn form_z_matrix(container: Rc<RefCell<Container>>, n: usize, m: usize) -> ndarray::Array2<MathOp> {
    let mut matrix: ArrayBase<OwnedRepr<MathOp>, Ix2> =
        ndarray::Array2::<MathOp>::zeros((n + m, 1));

    // I Matrix
    // The balance of current flowing in the node.
    for (i, tool) in container.borrow().nodes().iter().enumerate() {
        let mut set: Vec<MathOp> = Vec::new();
        for element in &tool.upgrade().unwrap().members {
            let element = element.upgrade().unwrap();
            if element.class != CurrentSrc {
                continue;
            }
            set.push(MathOp::None(Rc::new(element.value)));
        }
        if set.len() == 0 {
            continue;
        }
        matrix[[i, 0]] = MathOp::Sum(set);
    }

    // E Matrix
    // The value of the voltage source.
    for (i, source) in container.borrow().get_voltage_sources().iter().enumerate() {
        matrix[[n + i, 0]] = MathOp::None(Rc::new(source.upgrade().unwrap().value));
    }

    matrix
}

fn form_x_matrix(container: Rc<RefCell<Container>>, n: usize, m: usize) -> ndarray::Array2<MathOp> {
    let mut matrix: ArrayBase<OwnedRepr<MathOp>, Ix2> =
        ndarray::Array2::<MathOp>::zeros((n + m, 1));

    // V Matrix
    for (i, tool) in container.borrow().nodes().iter().enumerate() {
        matrix[[i, 0]] = MathOp::Unknown(EquationRepr::new(
            format!("{}", tool.upgrade().unwrap().pretty_string()),
            0.0,
        ));
    }

    // J Matrix
    for (i, source) in container.borrow().get_voltage_sources().iter().enumerate() {
        matrix[[n + i, 0]] = MathOp::Unknown(EquationRepr::new(
            format!("{}", source.upgrade().unwrap().pretty_string()),
            0.0,
        ));
    }

    matrix
}

#[cfg(test)]
mod tests {
    use crate::math::EquationMember;
    use crate::solvers::SolverType::Matrix;
    use crate::solvers::{
        form_b_matrix, form_c_matrix, form_d_matrix, form_g_matrix, NodeSolver, Solver,
    };
    use crate::util::create_mna_container;
    use ndarray::array;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_node_solver() {
        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeSolver = Solver::new(Rc::new(RefCell::new(c)), Matrix);
        // println!("{:?}", solver.solve_matrix());
    }

    #[test]
    fn test_a_matrix() {
        let expected = array![
            ["1/R1", "", "", "-1", "0"],
            ["", "1/R2 + 1/R3", "-1/R2", "1", "0"],
            ["", "-1/R2", "1/R2", "0", "1"],
            ["-1", "1", "0", "0", "0"],
            ["0", "0", "1", "0", "0"]
        ];

        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeSolver = Solver::new(Rc::new(RefCell::new(c)), Matrix);

        assert_eq!(solver.a_matrix.map(|x| x.equation_string()), expected);
    }

    #[test]
    fn test_g_matrix() {
        let expected = array![
            ["1/R1", "", ""],
            ["", "1/R2 + 1/R3", "-1/R2"],
            ["", "-1/R2", "1/R2"]
        ];

        let mut c = create_mna_container();
        c.create_nodes();
        let n = c.nodes().len();

        assert_eq!(
            form_g_matrix(Rc::new(RefCell::new(c)), n).map(|x| x.equation_string()),
            expected
        );
    }

    #[test]
    fn test_b_matrix() {
        let expected = array![["-1", "0"], ["1", "0"], ["0", "1"]];

        let mut c = create_mna_container();
        c.create_nodes();
        let n = c.nodes().len();
        let m = c.get_voltage_sources().len();

        assert_eq!(
            form_b_matrix(Rc::new(RefCell::new(c)), n, m).map(|x| x.equation_string()),
            expected
        );
    }

    #[test]
    fn test_c_matrix() {
        let expected = array![["-1", "1", "0"], ["0", "0", "1"]];

        let mut c = create_mna_container();
        c.create_nodes();
        let n = c.nodes().len();
        let m = c.get_voltage_sources().len();

        assert_eq!(
            form_c_matrix(Rc::new(RefCell::new(c)), n, m).map(|x| x.equation_string()),
            expected
        );
    }

    #[test]
    fn test_d_matrix() {
        let expected = array![["0", "0"], ["0", "0"]];

        let mut c = create_mna_container();
        c.create_nodes();
        let _n = c.nodes().len();
        let m = c.get_voltage_sources().len();

        assert_eq!(
            form_d_matrix(Rc::new(RefCell::new(c)), m).map(|x| x.equation_string()),
            expected
        );
    }

    #[test]
    fn test_x_matrix() {
        let expected = array![
            ["Node: 1"],
            ["Node: 2"],
            ["Node: 3"],
            ["SRC(V)4: 32 V"],
            ["SRC(V)5: 20 V"]
        ];

        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeSolver = Solver::new(Rc::new(RefCell::new(c)), Matrix);

        assert_eq!(solver.x_matrix.map(|x| x.equation_string()), expected);
    }

    #[test]
    fn test_z_matrix() {
        let expected = array![["0"], ["0"], ["0"], ["32"], ["20"]];

        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeSolver = Solver::new(Rc::new(RefCell::new(c)), Matrix);

        assert_eq!(solver.z_matrix.map(|x| x.equation_string()), expected);
    }

    #[test]
    fn test_solve_steps() {
        let mut c = create_mna_container();
        c.create_nodes();
        let solver: NodeSolver = Solver::new(Rc::new(RefCell::new(c)), Matrix);
        println!("{:?}", solver.solve_steps());
    }
}
