use crate::component::Component::{Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;
use crate::solvers::solver::{Solver, Step, SubStep};
use crate::tools::Tool;
use crate::tools::ToolType::{Node, SuperNode};
use crate::validation::Validation;
use nalgebra::{DMatrix, DVector};
use operations::mappings::expand;
use operations::math::EquationMember;
use operations::operations::Operation;
use operations::prelude::{
    Display, Divide, Equal, Multiply, Negate, Power, Sum, Text, Value, Variable,
};
use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::panic;
use std::rc::{Rc, Weak};

pub struct NodeStepSolver {
    pub(crate) container: Rc<RefCell<Container>>,
    sources: Vec<SourceConnection>,          // Voltage sources
    current_values: Vec<(usize, Operation)>, // (Element ID, Equation for current form nodes)
    node_pairs: Vec<(usize, usize, Rc<RefCell<Element>>)>, // Each element is attached to a pair of nodes.
    node_coefficients: Vec<Operation>, // Coefficients of the node summation for the matrix
    node_voltages: DVector<f64>,       // This is the result of matrix manipulation
    connection_matrix: DMatrix<f64>,   // This is the base matrix for manipulation
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
            current_values: vec![],
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

        // FORMATTING and OUTPUT
        let mut steps: Vec<Step> = Vec::new();
        steps.push(self.display_base_kcl_equations()?);
        steps.push(self.display_connection_matrix()?);
        steps.push(self.display_solved_matrix()?);
        steps.push(self.display_currents()?);
        steps.push(self.current_steps()?);
        Ok(steps)
    }
}

impl NodeStepSolver {
    /// Node Pairs
    fn setup_connections(&mut self) -> Result<(), String> {
        let vec_size: usize = match self
            .node_pairs
            .iter()
            .max_by(|(a, _, _), (b, _, _)| a.cmp(b))
        {
            None => return Err("Node pairs are invalid, cannot do analysis".to_string()),
            Some(a) => a.0,
        };

        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.borrow().class == VoltageSrc)
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
                    voltage: src.borrow().value(),
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

        self.node_coefficients
            .iter()
            .enumerate()
            .for_each(|(i, x)| {
                self.connection_matrix
                    .get_mut((0, i))
                    .map(|y| *y = x.value());
            });
        self.sources.iter().enumerate().for_each(|(i, x)| {
            x.matrix.iter().enumerate().for_each(|(j, y)| {
                self.connection_matrix.get_mut((i + 1, j)).map(|z| *z = *y);
            });
        });

        let inverse_result: Result<DMatrix<f64>, Box<dyn Any + Send>> =
            panic::catch_unwind(|| self.connection_matrix.clone().try_inverse().unwrap());

        let inverse: DMatrix<f64>;
        if let Err(_) = inverse_result {
            return Err(format!(
                "Unable to invert matrix: {}",
                self.connection_matrix.equation_repr()
            ));
        } else {
            inverse = inverse_result.unwrap();
        }

        self.inverse = inverse.clone();
        let result_matrix = inverse * source_voltages.clone();
        self.node_voltages = result_matrix.clone();

        self.matrix_evaluation = Display(Rc::new(Equal(
            Some(Box::new(Multiply(vec![
                Power(
                    Some(Box::new(Display(Rc::new(self.connection_matrix.clone())))),
                    Some(Box::new(Value(-1.0))),
                ),
                Display(Rc::new(source_voltages.clone())),
            ]))),
            Some(Box::new(Display(Rc::new(result_matrix.clone())))),
        )));

        // Propagate the values of the nodes back into the container / solver.
        let results: Vec<f64> = result_matrix
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<f64>>();
        self.container
            .borrow_mut()
            .nodes()
            .iter()
            .enumerate()
            .for_each(|(i, x)| {
                x.upgrade().unwrap().borrow_mut().set_value(results[i]);
            });

        Ok(())
    }

    fn setup_node_equations(&mut self) -> Result<(), String> {
        // Form the basic equation for each resistor
        assert_ne!(self.node_pairs.len(), 0);
        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.borrow().class == Resistor)
            .for_each(|(node1, node2, element)| {
                let mut tools: Vec<Operation> = Vec::new();
                let mut id_1 = *node1;
                let mut id_2 = *node2;

                if *node1 != 0 {
                    id_1 -= 1;
                    tools.push(Variable(Rc::new(
                        self.container
                            .borrow()
                            .get_tool_by_id(id_1)
                            .borrow()
                            .clone(),
                    )));
                }
                if *node2 != 0 {
                    id_2 -= 1;
                    tools.push(Negate(Some(Box::new(Variable(Rc::new(
                        self.container
                            .borrow()
                            .get_tool_by_id(id_2)
                            .borrow()
                            .clone(),
                    ))))));
                }

                let value: f64 = element.borrow().value().clone();
                element.borrow_mut().set_current(Divide(
                    Some(Box::new(Sum(tools.clone()).simplify().unwrap())),
                    Some(Box::new(Value(value))),
                ));

                self.current_values.push((
                    element.id(),
                    Divide(
                        Some(Box::new(Sum(tools).simplify().unwrap())),
                        Some(Box::new(Variable(Rc::new(element.borrow().clone())))),
                    ),
                ));

                self.node_combination_steps.push(Negate(Some(Box::new(
                    self.current_values.last().unwrap().1.clone(),
                ))));
            });

        assert_ne!(self.node_combination_steps.len(), 0);

        self.kcl_operations
            .push(Sum(self.node_combination_steps.clone()));

        // Create nicely readable equation
        self.node_combination_steps = self
            .node_combination_steps
            .iter()
            .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
            .collect();

        Ok(())
    }

    fn setup_node_coefficients(&mut self) -> Result<(), String> {
        // Expand equation
        assert_ne!(self.node_combination_steps.len(), 0);
        let mut combination_steps = self
            .node_combination_steps
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

    fn declare_variables(&self) -> Vec<SubStep> {
        let mut sub_steps: Vec<SubStep> = Vec::new();
        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.borrow().class == VoltageSrc)
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
        let node_labels: Vec<String> = self
            .container
            .borrow()
            .nodes()
            .iter()
            .map(|x| x.upgrade().unwrap().borrow().latex_string())
            .collect();
        sub_steps.push(SubStep {
            description: Some("Voltage at each node".to_string()),
            result: Some(Text(node_labels.join(", "))),
            operations: vec![],
        });
        sub_steps
    }

    fn display_base_kcl_equations(&self) -> Result<Step, String> {
        let mut steps: Vec<SubStep> = Vec::new();
        let nodes: Vec<Rc<RefCell<Tool>>> = self.container.borrow().get_calculation_nodes();
        let super_nodes: Vec<Weak<RefCell<Tool>>> =
            self.container.borrow().get_tools_by_type(SuperNode);
        let base_nodes: Vec<Weak<RefCell<Tool>>> = self.container.borrow().get_tools_by_type(Node);

        let mut kcl_equations: Vec<Operation> = Vec::new();
        let mut node_count = 0;
        let mut supernode_count = 0;
        for node in nodes.iter() {
            let members: Vec<Rc<RefCell<Element>>> = node.borrow().clone().into_iter().collect();

            let cleaned_i: Vec<Operation> = members
                .iter()
                .filter(|x| x.borrow().class != VoltageSrc)
                .map(|x| {
                    let mut new: Element = (**x).borrow().clone();
                    new.set_name("i".to_string());
                    Variable(Rc::new(new))
                })
                .collect();

            let (node_type, count): (&str, usize) = if node.borrow().class == SuperNode {
                supernode_count += 1;
                ("Super Node", supernode_count)
            } else {
                node_count += 1;
                ("Node", node_count)
            };

            kcl_equations.push(Equal(
                Some(Box::new(Text(format!("{node_type} ({count}): ")))),
                Some(Box::new(Sum(cleaned_i))),
            ));
        }

        steps.push(SubStep {
            description: Some("Mark Nodes".to_string()),
            result: None,
            operations: base_nodes
                .iter()
                .map(|x| Variable(Rc::new(x.upgrade().unwrap().borrow().clone())))
                .collect(),
        });

        steps.push(SubStep {
            description: Some("Mark Supernodes".to_string()),
            result: None,
            operations: super_nodes
                .iter()
                .map(|x| Variable(Rc::new(x.upgrade().unwrap().borrow().clone())))
                .collect(),
        });

        steps.push(SubStep {
            description: Some("Current entering and exiting each node.".to_string()),
            result: None,
            operations: kcl_equations,
        });

        let mut i_values: Vec<Operation> = Vec::new();
        self.current_values.iter().for_each(|(id, equation)| {
            let i_element = (**self.container.borrow().get_element_by_id(*id)).clone();
            let v_element = (**self.container.borrow().get_element_by_id(*id)).clone();
            i_element.borrow_mut().name = "i".to_string();
            v_element.borrow_mut().name = "V".to_string();
            let potential_expansion: Operation = match expand(equation.clone()) {
                Ok(mut x) => {
                    x.apply_variables();
                    Equal(
                        Some(Box::new(Variable(Rc::new(equation.clone())))),
                        Some(Box::new(Variable(Rc::new(x.clone())))),
                    )
                }
                Err(_) => Variable(Rc::new(equation.clone())),
            };

            i_values.push(Equal(
                Some(Box::new(Variable(Rc::new(i_element.borrow().clone())))),
                Some(Box::new(Equal(
                    Some(Box::new(Divide(
                        Some(Box::new(Variable(Rc::new(v_element.borrow().clone())))),
                        Some(Box::new(Variable(Rc::new(
                            self.container
                                .borrow()
                                .get_element_by_id(*id)
                                .borrow()
                                .clone(),
                        )))),
                    ))),
                    Some(Box::new(potential_expansion)),
                ))),
            ));
        });

        steps.push(SubStep{
            description: Some("Use potential difference between nodes ($ N_{j, k} $) and Ohm's law to solve for current. Where $j, k$ are the two nodes that the element is connected to. We can treat GND as 0.".to_string()),
            result: None,
            operations: i_values,
        });

        Ok(Step {
            title: Some("KCL Equations".to_string()),
            description: Some("Outline the basis of the circuit using KCL equations".to_string()),
            result: None,
            sub_steps: steps,
        })
    }

    fn voltage_src_equations(&self) -> Result<Step, String> {
        let mut eq_steps: Vec<SubStep> = Vec::new();
        // Step 2.1.2 Find all voltage sources going between nodes including ground

        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.borrow().class == VoltageSrc)
            .for_each(|(node1, node2, element)| {
                let mut tool2: Operation = Value(0.0);
                let mut tool1: Operation = Value(0.0);
                let mut id_1 = *node1;
                let mut id_2 = *node2;
                if *node1 != 0 {
                    id_1 -= 1;
                    tool1 = Variable(Rc::new(
                        self.container
                            .borrow()
                            .get_tool_by_id(id_1)
                            .borrow()
                            .clone(),
                    ));
                }
                if *node2 != 0 {
                    id_2 -= 1;
                    tool2 = Variable(Rc::new(
                        self.container
                            .borrow()
                            .get_tool_by_id(id_2)
                            .borrow()
                            .clone(),
                    ));
                }

                tool2 = Negate(Some(Box::new(tool2)));

                eq_steps.push(SubStep {
                    description: None,
                    result: None,
                    operations: vec![Equal(
                        Some(Box::new(Variable(Rc::new(element.borrow().clone())))),
                        Some(Box::new(Sum(vec![tool1, tool2]))),
                    )],
                })
            });

        Ok(Step::new_with_steps(
            "Find voltage across each voltage source",
            eq_steps,
        ))
    }

    fn current_steps(&self) -> Result<Step, String> {
        let mut current_equations: Vec<Operation> = Vec::new();
        let mut element_vector: Vec<Operation> = Vec::new();
        self.node_pairs
            .iter()
            .filter(|(_, _, element)| element.borrow().class == Resistor)
            .for_each(|(node1, node2, element)| {
                let mut i = element.borrow().clone();
                i.set_name("i".to_string());
                element_vector.push(Variable(Rc::new(i)));
                let mut tools: Vec<Operation> = Vec::new();
                if *node1 != 0 {
                    tools.push(Value(self.node_voltages[*node1 - 1]));
                }
                if *node2 != 0 {
                    tools.push(Negate(Some(Box::new(Value(
                        self.node_voltages[*node2 - 1],
                    )))));
                }

                current_equations.push(
                    Divide(
                        Some(Box::new(Sum(tools).simplify().unwrap())),
                        Some(Box::new(Value(element.borrow().value()))),
                    )
                    .simplify()
                    .unwrap(),
                );
            });

        Ok(Step {
            title: Some("Current Results".to_string()),
            description: None,
            result: Some(Equal(
                Some(Box::new(Display(Rc::new(DVector::from_vec(
                    element_vector.clone(),
                ))))),
                Some(Box::new(Display(Rc::new(DVector::from_vec(
                    current_equations.clone(),
                ))))),
            )),
            sub_steps: vec![],
        })
    }

    fn display_connection_matrix(&self) -> Result<Step, String> {
        Ok(Step {
            title: Some("Connection Matrix".to_string()),
            description: None,
            result: Some(Display(Rc::new(self.connection_matrix.clone()))),
            sub_steps: vec![
                SubStep {
                    description: Some("Coefficients from the expanded KCL equations".to_string()),
                    result: Some(Equal(
                        Some(Box::new(Display(Rc::new(DVector::from(
                            self.node_coefficients.clone(),
                        ))))),
                        Some(Box::new(Display(Rc::new(DVector::from_vec(
                            self.container
                                .borrow()
                                .nodes()
                                .iter()
                                .map(|x| {
                                    Variable(Rc::new(x.upgrade().unwrap().borrow().deref().clone()))
                                })
                                .collect::<Vec<Operation>>(),
                        ))))),
                    )),
                    operations: vec![],
                },
                SubStep {
                    description: Some("Element connections between nodes.".to_string()),
                    result: Some(Display(Rc::new(
                        self.connection_matrix.clone().remove_rows(0, 1),
                    ))),
                    operations: self
                        .node_pairs
                        .iter()
                        .filter_map(|x| {
                            if x.0 != 0 && x.1 != 0 || x.2.borrow().class == VoltageSrc {
                                return Some(Equal(
                                    Some(Box::new(Display(Rc::new(DVector::from_vec(vec![
                                        x.0 as f64, x.1 as f64,
                                    ]))))),
                                    Some(Box::new(Text(format!(
                                        " The current flows from Node {} to Node {}",
                                        x.0, x.1
                                    )))),
                                ));
                            }
                            None
                        })
                        .collect::<Vec<Operation>>(),
                },
                SubStep {
                    description: Some("TODO explain this super step".to_string()),
                    result: None,
                    operations: vec![],
                },
            ],
        })
    }

    fn display_solved_matrix(&self) -> Result<Step, String> {
        let i_values: DVector<Operation> = DVector::from_vec(
            self.container
                .borrow()
                .nodes()
                .iter()
                .map(|x| Variable(Rc::new(x.upgrade().unwrap().borrow().deref().clone())))
                .collect::<Vec<Operation>>(),
        );
        let result: Operation = Equal(
            Some(Box::new(Display(Rc::new(i_values.clone())))),
            Some(Box::new(Display(Rc::new(self.node_voltages.clone())))),
        );

        Ok(Step {
            title: Some("Solve For Node Voltages".to_string()),
            description: None,
            result: Some(result),
            sub_steps: vec![
                SubStep {
                    description: Some("Invert the matrix".to_string()),
                    result: None,
                    operations: vec![
                        Power(
                            Some(Box::new(Display(Rc::new(self.connection_matrix.clone())))),
                            Some(Box::new(Value(-1.0))),
                        ),
                        Display(Rc::new(self.inverse.clone())),
                    ],
                },
                SubStep {
                    description: Some(
                        "Multiply the inverted matrix by the source voltages".to_string(),
                    ),
                    result: Some(Display(Rc::new(self.node_voltages.clone()))),
                    operations: vec![Display(Rc::new(self.matrix_evaluation.clone()))],
                },
            ],
        })
    }

    fn display_currents(&self) -> Result<Step, String> {
        let mut steps: Vec<SubStep> = Vec::new();
        let mut i_values: Vec<Operation> = Vec::new();
        self.current_values.iter().for_each(|(id, equation)| {
            let i_element = (**self.container.borrow().get_element_by_id(*id)).clone();
            i_element.borrow_mut().name = "i".to_string();

            i_values.push(Equal(
                Some(Box::new(Variable(Rc::new(i_element.borrow().clone())))),
                Some(Box::new(equation.clone())),
            ));
        });

        steps.push(SubStep{
            description: Some("Use potential difference between nodes ($ N_j $) and Ohm's law to solve for current.".to_string()),
            result: None,
            operations: i_values,
        });

        Ok(Step {
            title: Some("Currents".to_string()),
            description: Some(
                "Evaluate the currents using the KCL equations and node voltages shown previously."
                    .to_string(),
            ),
            result: None,
            sub_steps: steps,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::container::Container;
    use crate::solvers::node_step_solver::NodeStepSolver;
    use crate::solvers::solver::Solver;
    use crate::util::create_mna_container;
    use nalgebra::DVector;
    use operations::math::EquationMember;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_node_pairs() {
        let solver = setup_mna_solver();
        println!("{:?}", solver.node_pairs);
        assert_eq!(solver.node_pairs.len(), 5);
    }

    #[test]
    fn test_coefficients() {
        let solver = setup_mna_solver();
        assert_eq!(solver.node_coefficients.len(), 3);
        assert_eq!(
            solver
                .node_coefficients
                .into_iter()
                .map(|x| x.value())
                .collect::<Vec<f64>>(),
            vec![-0.25, 0.375, 0.5]
        );
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
        assert_eq!(
            solver.node_voltages,
            DVector::from_vec(vec![20.0, 24.0, -8.0])
        );
    }

    fn setup_mna_solver() -> NodeStepSolver {
        let mut c: Container = create_mna_container();
        c.create_nodes().unwrap();
        c.create_super_nodes();
        let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
        solver.solve().expect("Unable to solve");
        solver
    }
}
