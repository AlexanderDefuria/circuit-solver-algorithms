use crate::component::Component;
use crate::component::Component::{CurrentSrc, Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;
use crate::solvers::solver::{Solver, Step, SubStep};
use crate::tools::{Tool, ToolType};
use operations::mappings::expand;
use operations::math::EquationMember;
use operations::operations::Operation;
use operations::prelude::{Divide, Equal, Negate, Sum, Text, Value, Variable};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use crate::solvers::node_matrix_solver::form_b_matrix;

pub struct NodeStepSolver {
    container: Rc<RefCell<Container>>,
}

impl Solver for NodeStepSolver {
    fn new(container: Rc<RefCell<Container>>) -> Self {
        NodeStepSolver { container }
    }

    /// Returns a vector of strings that represent the steps to solve the circuit.
    fn solve(&self) -> Result<Vec<Step>, String> {
        let node_pairs: Vec<(usize, usize, Rc<Element>)> =
            self.container.borrow().get_all_node_pairs();

        let mut steps: Vec<Step> = Vec::new();

        let supernodes: Vec<Weak<Tool>> = self
            .container
            .borrow()
            .get_tools_by_type(ToolType::SuperNode);
        // if supernodes.len() > 0 {
        //     steps.push(Step::new("Solve for supernodes:"));
        //     supernodes.iter().for_each(|_| {
        //         // TODO: Add supernode solver
        //         todo!();
        //     });
        // }

        // Step 1 Declare
        steps.push(Step::new("Steps to solve the circuit:"));
        steps.push(declare_variables(&node_pairs));

        // Step 2 Setup Voltages
        steps.push(breakdown_resistor_equations(
            &node_pairs,
            &self.container.borrow(),
        ));
        steps.push(breakdown_voltage_src_equations(
            &node_pairs,
            &self.container.borrow(),
        ));

        // Step 3 Solve Voltages
        steps.push(Step::new("Solve for voltages:"));

        Ok(steps)
    }
}


fn declare_variables(node_pairs: &Vec<(usize, usize, Rc<Element>)>) -> Step {
    let mut sub_steps: Vec<SubStep> = Vec::new();
    let voltage_pairs: Vec<&(usize, usize, Rc<Element>)> = get_node_pairs(node_pairs, VoltageSrc);
    voltage_pairs.iter().for_each(|(node1, node2, _)| {
        let ls: Operation = Text(format!("{{V_{{{}, {}}}}}", node1, node2));

        sub_steps.push(SubStep {
            description: Some(
                format!("voltage and current from node {} to node {}", node1, node2).to_string(),
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

fn breakdown_voltage_src_equations(
    node_pairs: &Vec<(usize, usize, Rc<Element>)>,
    container: &Container,
) -> Step {
    let mut eq_steps: Vec<SubStep> = Vec::new();
    // Step 2.1.2 Find all voltage sources going between nodes including ground
    let voltage_src_node_pairs: Vec<&(usize, usize, Rc<Element>)> =
        get_node_pairs(node_pairs, VoltageSrc);
    voltage_src_node_pairs
        .iter()
        .for_each(|(node1, node2, element)| {
            let mut tool2: Operation = Value(0.0);
            let mut tool1: Operation = Value(0.0);
            let mut id_1 = *node1;
            let mut id_2 = *node2;
            if *node1 != 0 {
                id_1 -= 1;
                tool1 = Variable(container.get_tool_by_id(id_1).clone());
            }
            if *node2 != 0 {
                id_2 -= 1;
                tool2 = Variable(container.get_tool_by_id(id_2).clone());
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

    Step {
        description: Some("Find voltage across each voltage source".to_string()),
        sub_steps: eq_steps,
        result: None,
    }
}

fn breakdown_resistor_equations(
    node_pairs: &Vec<(usize, usize, Rc<Element>)>,
    container: &Container,
) -> Step {
    let mut summation_steps: Vec<Operation> = Vec::new();
    // Step 2.1.1 Find all resistors going between nodes including ground
    let resistor_node_pairs: Vec<&(usize, usize, Rc<Element>)> =
        get_node_pairs(node_pairs, Resistor);

    // Form the basic equation for each resistor
    resistor_node_pairs
        .iter()
        .for_each(|(node1, node2, element)| {
            let mut tools: Vec<Operation> = Vec::new();
            let mut id_1 = *node1;
            let mut id_2 = *node2;

            if *node1 != 0 {
                id_1 -= 1;
                tools.push(Variable(container.get_tool_by_id(id_1).clone()));
            }
            if *node2 != 0 {
                id_2 -= 1;
                tools.push(Negate(Some(Box::new(Variable(
                    container.get_tool_by_id(id_2).clone(),
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
    let mut collected: Vec<(Operation, f64)> = sum.get_variables().iter().map(|x| (x.clone(), 0.0)).collect();
    if let Sum(list) = sum.clone() {
        for i in list {
            for (var, coeff) in &mut collected {
                if i.contains_variable(var.deref().clone()) {
                    *coeff += i.get_coefficient().unwrap_or(0.0);
                }
            }
        }
    }
    let coefficients: Vec<Operation> = collected.iter().map(|(_, coeff)| Value(*coeff)).collect();

    // TODO Form matrix from coefficients
    let n = container.nodes().len(); // Node Count
    let m = container // Source Count
        .get_elements()
        .iter()
        .fold(0, |acc: usize, x| match x.class {
            VoltageSrc | CurrentSrc => acc + 1,
            _ => acc,
        });
    let b_matrix = form_b_matrix(Rc::new(RefCell::new(container.clone())), n, m);


    let resistor_values: Operation = Equal(Some(Box::new(Value(0.0))), Some(Box::new(sum.clone())));

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
        ],
        result: None,
    }
}

fn get_node_pairs(
    node_pairs: &Vec<(usize, usize, Rc<Element>)>,
    filter: Component,
) -> Vec<&(usize, usize, Rc<Element>)> {
    node_pairs
        .iter()
        .filter(|(_, _, element)| element.class == filter)
        .collect::<Vec<&(usize, usize, Rc<Element>)>>()
}
