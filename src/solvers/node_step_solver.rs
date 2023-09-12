use std::any::Any;
use crate::component::Component::{Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;
use crate::solvers::solver::{Solver, Step, SubStep};
use nalgebra::{DMatrix, DVector};
use operations::mappings::expand;
use operations::math::EquationMember;
use operations::operations::Operation;
use operations::prelude::{Divide, Equal, Negate, Sum, Text, Value, Variable};
use std::cell::RefCell;
use std::ops::Deref;
use std::panic;
use std::rc::Rc;

pub struct NodeStepSolver {
    container: Rc<RefCell<Container>>,
    sources: Vec<SourceConnection>, // Voltage sources
    node_pairs: Vec<(usize, usize, Rc<Element>)>, // Each element is attached to a pair of nodes.
    node_coefficients: Vec<Operation>, // Coefficients of the node summation for the matrix
    node_voltages: DVector<f64>, // This is the result of matrix manipulation
    connection_matrix: DMatrix<f64>, // This is the base matrix for manipulation
    node_combination_steps: Vec<Operation>,
    matrix_evaluation: Operation, // Simple operation holding the matrix multiplication display.
    kcl_operations: Vec<Operation>,
    inverse: DMatrix<f64>,
}

#[derive(Debug)]
struct SourceConnection {
    matrix: DVector<f64>,
    voltage: f64,
}

impl Solver for NodeStepSolver {

    /// Creates a new NodeStepSolver
    ///
    /// This is where all the steps are created and handled
    fn new(container: Rc<RefCell<Container>>) -> Self {
        let node_pairs = container.borrow().get_all_node_pairs();
        let out: NodeStepSolver = NodeStepSolver {
            container,
            sources: vec![],
            node_pairs,
            node_coefficients: vec![],
            node_voltages: DVector::zeros(0),
            connection_matrix: DMatrix::zeros(0, 0),
            node_combination_steps: vec![],
            matrix_evaluation: Text("".to_string()),
            kcl_operations: vec![],
            inverse: DMatrix::zeros(0, 0),
        };

        out
    }

    /// Returns a vector of strings that represent the steps to solve the circuit.
    ///
    /// This Handles the formatting of the data into what the frontend requires.
    fn solve(&mut self) -> Result<Vec<Step>, String> {

        // SETUP and CALCULATIONS
        self.setup_connections()?;
        self.setup_node_equations()?;
        self.setup_node_coefficients()?;
        self.solve_node_voltages()?;
        self.solve_current_values()?;


        // FORMATTING and OUTPUT
        let mut steps: Vec<Step> = Vec::new();
        steps.push(self.declare_variables());
        steps.push(self.voltage_src_equations());
        steps.push(self.kcl_equations());
        steps.push(self.connection_matrix());
        steps.push(self.solve_matrix());
        Ok(steps)
    }
}

impl NodeStepSolver {

    /// Node Pairs
    fn setup_connections(&mut self) -> Result<(), String> {
        let vec_size: usize = self
            .node_pairs
            .iter()
            .max_by(|(a, _, _), (b, _, _)| a.cmp(b))
            .unwrap()
            .0;

        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.class == VoltageSrc)
            .for_each(|(node1, node2, src)| {
                let mut voltage_connections: DVector<f64> = DVector::zeros(vec_size);
                match (node1, node2) {
                    (0, a) | (a, 0) => {
                        voltage_connections.get_mut(a - 1).map(|x| *x = 1.0);
                    }
                    (a, b) => {
                        voltage_connections
                            .get_mut::<usize>(a - 1)
                            .map(|x: &mut f64| *x = 1.0);
                        voltage_connections
                            .get_mut::<usize>(b - 1)
                            .map(|x: &mut f64| *x = -1.0);
                    }
                }
                self.sources.push(SourceConnection {
                    matrix: voltage_connections,
                    voltage: src.value(),
                });
            });
        Ok(())
    }

    fn solve_node_voltages(&mut self) -> Result<(), String> {
        let mut source_voltages: DVector<f64> = DVector::zeros(self.sources.len() + 1);

        self.sources.iter().enumerate().for_each(|(i, x)| {
            source_voltages.get_mut(i + 1).map(|y| *y = x.voltage);
        });

        // TODO Form matrix from coefficients
        let n: usize = self.node_coefficients.len();
        let m: usize = 1 + self.sources.len();
        self.connection_matrix = DMatrix::zeros(n, m);

        self.node_coefficients.iter().enumerate().for_each(|(i, x)| {
            self.connection_matrix.get_mut((0, i)).map(|y| *y = x.value());
        });
        self.sources.iter().enumerate().for_each(|(i, x)| {
            x.matrix.iter().enumerate().for_each(|(j, y)| {
                self.connection_matrix.get_mut((i + 1, j)).map(|z| *z = *y);
            });
        });

        let inverse_result: Result<DMatrix<f64>, Box<dyn Any + Send>> = panic::catch_unwind(|| {
            self.connection_matrix.clone().try_inverse().unwrap()
        });

        let inverse: DMatrix<f64>;
        if let Err(_) = inverse_result {
            return Err(format!("Unable to invert matrix: {}", self.connection_matrix.equation_repr()))
        } else {
            inverse = inverse_result.unwrap();
        }

        self.inverse = inverse.clone();
        let result_matrix = inverse * source_voltages.clone();
        self.node_voltages = result_matrix.clone();

        self.matrix_evaluation = Text(format!(
            "{}^{{-1}} * {} = {}",
            self.connection_matrix.equation_repr(),
            source_voltages.equation_repr(),
            self.node_voltages.equation_repr()
        ));

        // TODO Propagate the values of the nodes back into the container / solver.
        // let results: Vec<f64> = result_matrix.iter().map(|x| x.clone()).collect::<Vec<f64>>();
        // self.container.borrow_mut().nodes().iter().enumerate().for_each(|(i, x)| {
        //     x.upgrade().unwrap().set_value(results[i]);
        // });

        Ok(())
    }

    fn setup_node_equations(&mut self) -> Result<(), String> {
        // Form the basic equation for each resistor
        assert_ne!(self.node_pairs.len(), 0);
        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.class == Resistor)
            .for_each(|(node1, node2, element)| {
                let mut tools: Vec<Operation> = Vec::new();
                let mut id_1 = *node1;
                let mut id_2 = *node2;

                if *node1 != 0 {
                    id_1 -= 1;
                    tools.push(Variable(
                        self.container.borrow().get_tool_by_id(id_1).clone(),
                    ));
                }
                if *node2 != 0 {
                    id_2 -= 1;
                    tools.push(Negate(Some(Box::new(Variable(
                        self.container.borrow().get_tool_by_id(id_2).clone(),
                    )))));
                }
                self.node_combination_steps.push(Negate(Some(Box::new(Divide(
                    Some(Box::new(Sum(tools).simplify().unwrap())),
                    Some(Box::new(Variable(element.clone()))),
                )))));
            });

        assert_ne!(self.node_combination_steps.len(), 0);
        self.kcl_operations.push(Sum(self.node_combination_steps.clone()));

        // Create nicely readable equation
        self.node_combination_steps = self.node_combination_steps
            .iter()
            .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
            .collect();

        Ok(())
    }

    fn setup_node_coefficients(&mut self) -> Result<(), String> {
        // Expand equation
        assert_ne!(self.node_combination_steps.len(), 0);
        let mut combination_steps = self.node_combination_steps
            .iter()
            .map(|x| expand(x.clone()).unwrap_or_else(|_| x.clone()))
            .collect::<Vec<Operation>>();
        self.kcl_operations.push(Sum(combination_steps.clone()));
        combination_steps = combination_steps
            .iter()
            .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
            .collect::<Vec<Operation>>();
        let mut sum: Operation = Sum(combination_steps.clone());
        self.kcl_operations.push(sum.clone());
        sum = sum.simplify().unwrap_or_else(|| sum.clone());

        // Include known values to extract coefficients
        sum.apply_variables();

        // Group coefficients by variable (Tool)
        let mut collected: Vec<(Operation, f64)> = sum
            .get_variables()
            .iter()
            .map(|x| (x.clone(), 0.0))
            .collect();
        if let Sum(list) = sum.clone() {
            for i in list {
                for (var, coeff) in &mut collected {
                    if i.contains_variable(var.deref().clone()) {
                        *coeff += i.get_coefficient().unwrap_or(0.0);
                    }
                }
            }
        }
        collected
            .sort_by(|(a, _), (b, _)| a.latex_string().partial_cmp(&b.latex_string()).unwrap());
        self.node_coefficients = collected.iter().map(|(_, coeff)| Value(*coeff)).collect();

        Ok(())
    }

    fn solve_current_values(&mut self) -> Result<(), String> {

        Ok(())
    }

    fn declare_variables(&self) -> Step {
        let mut sub_steps: Vec<SubStep> = Vec::new();
        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.class == VoltageSrc)
            .for_each(|(node1, node2, _)| {
                sub_steps.push(SubStep {
                    description: Some(
                        format!("voltage and current from node {} to node {}", node1, node2)
                            .to_string(),
                    ),
                    result: Some(Text(format!("v_{{{},{}}}", node1, node2))),
                    operations: vec![],
                });
            });
        let node_labels: Vec<String> = self.container.borrow().nodes().iter().map(|x| x.upgrade().unwrap().latex_string()).collect();
        sub_steps.push(SubStep {
            description: Some("Voltage at each node".to_string()),
            result: Some(Text(node_labels.join(", "))),
            operations: vec![],
        });
        Step::new_with_steps("Declare Variables.", sub_steps)
    }

    fn kcl_equations(&self) -> Step {
        Step {
            description: Some("KCL equations".to_string()),
            result: Some(Equal(Some(Box::new(Value(0.0))), Some(Box::new(self.kcl_operations.last().unwrap().clone())))),
            sub_steps: vec![SubStep {
                description: None,
                result: None,
                operations: self.kcl_operations.clone(),
            }],
        }
    }

    fn voltage_src_equations(&self) -> Step {
        let mut eq_steps: Vec<SubStep> = Vec::new();
        // Step 2.1.2 Find all voltage sources going between nodes including ground

        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.class == VoltageSrc)
            .for_each(|(node1, node2, element)| {
                let mut tool2: Operation = Value(0.0);
                let mut tool1: Operation = Value(0.0);
                let mut id_1 = *node1;
                let mut id_2 = *node2;
                if *node1 != 0 {
                    id_1 -= 1;
                    tool1 = Variable(self.container.borrow().get_tool_by_id(id_1).clone());
                }
                if *node2 != 0 {
                    id_2 -= 1;
                    tool2 = Variable(self.container.borrow().get_tool_by_id(id_2).clone());
                }

                tool2 = Negate(Some(Box::new(tool2)));

                eq_steps.push(SubStep {
                    description: None,
                    result: None,
                    operations: vec![Equal(
                        Some(Box::new(Variable(element.clone()))),
                        Some(Box::new(Sum(vec![tool1, tool2]))),
                    )],
                })
            });


        Step::new_with_steps("Find voltage across each voltage source", eq_steps)
    }

    fn connection_matrix(&self) -> Step {
        Step {
            description: Some("Connection Matrix".to_string()),
            result: Some(Text(format!("{}", self.connection_matrix.equation_repr()))),
            sub_steps: vec![
                SubStep {
                    description: Some("Coefficients".to_string()),
                    result: Some(Text(format!("{}", DVector::from(self.node_coefficients.clone()).equation_repr()))),
                    operations: vec![],
                },
                SubStep {
                    description: Some("Connections".to_string()),
                    result: Some(Text(format!("{}", self.connection_matrix.clone().remove_rows(0, 1).equation_repr()))),
                    operations: vec![],
                },
                SubStep {
                    description: Some("TODO explain this super step".to_string()),
                    result: None,
                    operations: vec![],
                }
            ],
        }
    }

    fn solve_matrix(&self) -> Step {
        Step {
            description: Some("Matrix equations".to_string()),
            result: Some(Text(self.node_voltages.clone().equation_repr())),
            sub_steps: vec![
                SubStep {
                    description: Some("Invert the matrix".to_string()),
                    result: Some(Text(self.inverse.clone().equation_repr())),
                    operations: vec![
                        Text(format!("M^{{-1}}")),
                        Text(format!("{}^{{-1}}", self.connection_matrix.equation_repr())),
                    ],
                },
                SubStep {
                    description: Some("Multiply the inverted matrix by the source voltages".to_string()),
                    result: Some(Text(self.node_voltages.equation_repr())),
                    operations: vec![
                        self.matrix_evaluation.clone(),
                    ],
                },
            ],
        }
    }
}


#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use nalgebra::DVector;
    use operations::math::EquationMember;
    use crate::container::Container;
    use crate::solvers::node_step_solver::NodeStepSolver;
    use crate::solvers::solver::Solver;
    use crate::util::create_mna_container;

    #[test]
    fn test_node_pairs() {
        let solver = setup_mna_solver();
        assert_eq!(solver.node_pairs.len(), 5);
    }

    #[test]
    fn test_coefficients() {
        let solver = setup_mna_solver();
        assert_eq!(solver.node_coefficients.len(), 3);
        assert_eq!(solver.node_coefficients.into_iter().map(|x| x.value()).collect::<Vec<f64>>(), vec![-0.25, 0.375, 0.5]);
    }

    #[test]
    fn test_combination_steps() {
        let solver = setup_mna_solver();
        assert_eq!(solver.node_combination_steps.len(), 3);
    }

    #[test]
    fn test_matrix() {
        let solver = setup_mna_solver();
        assert_eq!(solver.node_voltages.len(), 3);
        assert_eq!(solver.node_voltages, DVector::from_vec(vec![20.0, 24.0, -8.0]));
    }

    fn setup_mna_solver() -> NodeStepSolver {
        let mut c: Container = create_mna_container();
        c.create_nodes();
        c.create_super_nodes();
        let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
        solver.solve().expect("Unable to solve");
        solver
    }


}