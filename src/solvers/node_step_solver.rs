use crate::component::Component::{ Resistor, VoltageSrc};
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
use std::rc::Rc;

pub struct NodeStepSolver {
    container: Rc<RefCell<Container>>,
    sources: Vec<SourceConnection>,
    node_pairs: Vec<(usize, usize, Rc<Element>)>,
}

#[derive(Debug)]
struct SourceConnection {
    matrix: DVector<f64>,
    voltage: f64,
}

impl Solver for NodeStepSolver {
    fn new(container: Rc<RefCell<Container>>) -> Self {
        let node_pairs = container.borrow().get_all_node_pairs();
        let mut out: NodeStepSolver = NodeStepSolver {
            container,
            sources: vec![],
            node_pairs,
        };
        out.setup_connections();

        out
    }

    /// Returns a vector of strings that represent the steps to solve the circuit.
    fn solve(&self) -> Result<Vec<Step>, String> {
        let mut steps: Vec<Step> = Vec::new();

        // Step 1 Declare
        steps.push(Step::new("Steps to solve the circuit:"));
        steps.push(self.declare_variables());

        // Step 2 Setup Voltages
        steps.push(self.breakdown_resistor_equations());
        steps.push(self.breakdown_voltage_src_equations());

        // Step 3 Solve Voltages
        steps.push(Step::new("Solve for voltages:"));

        Ok(steps)
    }
}

impl NodeStepSolver {
    fn setup_connections(&mut self) {
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
                println!("Voltage Connections: {:?}", voltage_connections);
                self.sources.push(SourceConnection {
                    matrix: voltage_connections,
                    voltage: src.value(),
                });
            });
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
                    operations: vec![],
                });
            });

        Step {
            description: Some("Find voltage and current between nodes.".to_string()),
            sub_steps,
            result: None,
        }
    }

    fn breakdown_resistor_equations(&self) -> Step {
        let mut summation_steps: Vec<Operation> = Vec::new();
        // Step 2.1.1 Find all resistors going between nodes including ground

        // Form the basic equation for each resistor
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
                summation_steps.push(Negate(Some(Box::new(Divide(
                    Some(Box::new(Sum(tools).simplify().unwrap())),
                    Some(Box::new(Variable(element.clone()))),
                )))));
            });

        // Create nicely readable equation
        summation_steps = summation_steps
            .iter()
            .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
            .collect();
        let init_eq = Equal(
            Some(Box::new(Value(0.0))),
            Some(Box::new(Sum(summation_steps.clone()))),
        );

        // Expand equation
        summation_steps = summation_steps
            .iter()
            .map(|x| expand(x.clone()).unwrap_or_else(|_| x.clone()))
            .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
            .collect();
        let mut sum: Operation = Sum(summation_steps.clone());
        sum = sum.simplify().unwrap_or_else(|| sum.clone());
        let expanded = Equal(Some(Box::new(Value(0.0))), Some(Box::new(sum.clone())));

        // Include known values to exract coefficients
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
        let coefficients: Vec<Operation> =
            collected.iter().map(|(_, coeff)| Value(*coeff)).collect();

        let mut source_voltages: DVector<f64> = DVector::zeros(self.sources.len() + 1);
        self.sources.iter().enumerate().for_each(|(i, x)| {
            source_voltages.get_mut(i + 1).map(|y| *y = x.voltage);
        });

        // TODO Form matrix from coefficients
        let n: usize = coefficients.len();
        let m: usize = 1 + self.sources.len();
        let mut matrix: DMatrix<f64> = DMatrix::zeros(n, m);

        coefficients.iter().enumerate().for_each(|(i, x)| {
            matrix.get_mut((0, i)).map(|y| *y = x.value());
        });
        self.sources.iter().enumerate().for_each(|(i, x)| {
            x.matrix.iter().enumerate().for_each(|(j, y)| {
                matrix.get_mut((i + 1, j)).map(|z| *z = *y);
            });
        });

        let inverse: DMatrix<f64> = matrix.clone().try_inverse().unwrap();

        let result_matrix = inverse.clone() * source_voltages.clone();


        let resistor_values: Operation =
            Equal(Some(Box::new(Value(0.0))), Some(Box::new(sum.clone())));

        Step {
            description: Some("Find current through each resistor:".to_string()),
            sub_steps: vec![
                SubStep {
                    description: Some("Initial equation:".to_string()),
                    operations: vec![init_eq, expanded],
                },
                SubStep {
                    description: Some("Find our coefficients:".to_string()),
                    operations: vec![resistor_values, Sum(coefficients)],
                },
                SubStep {
                    description: Some("Form matrix from coefficients:".to_string()),
                    operations: vec![Text(format!("{:?}", matrix.equation_repr()))],
                },
                SubStep {
                    description: Some(
                        "Take the inverse and multiply by node voltages:".to_string(),
                    ),
                    operations: vec![
                        Text(format!(
                            "{}^-1 = {}",
                            matrix.equation_repr(),
                            inverse.equation_repr()
                        )),
                        Text(format!("{}^-1 * {} = {}", matrix.equation_repr(), source_voltages.equation_repr(), result_matrix.equation_repr())),
                    ],
                },
            ],
            result: None,
        }
    }

    fn breakdown_voltage_src_equations(&self) -> Step {
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
                    operations: vec![Equal(
                        Some(Box::new(Variable(element.clone()))),
                        Some(Box::new(Sum(vec![tool1, tool2]))),
                    )],
                })
            });

        for i in 0..eq_steps.len() {
            if let Equal(a, _) = eq_steps[i].operations[0].clone() {
                println!("{:?}", a.unwrap().value());
            }
        }

        Step {
            description: Some("Find voltage across each voltage source".to_string()),
            sub_steps: eq_steps,
            result: None,
        }
    }
}
