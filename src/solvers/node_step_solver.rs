use crate::component::Component;
use crate::component::Component::{Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;
use crate::solvers::solver::{Solver, Step};
use crate::tools::{Tool, ToolType};
use operations::mappings::expand;
use operations::math::EquationMember;
use operations::operations::Operation;
use operations::prelude::{Divide, Equal, Negate, Sum, Text, Value, Variable};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

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

fn extract_coefficients(operation: Operation) -> Vec<f64> {
    todo!()
}

fn declare_variables(node_pairs: &Vec<(usize, usize, Rc<Element>)>) -> Step {
    let mut sub_steps: Vec<Operation> = Vec::new();
    let voltage_pairs: Vec<&(usize, usize, Rc<Element>)> = get_node_pairs(node_pairs, VoltageSrc);
    voltage_pairs.iter().for_each(|(node1, node2, _)| {
        let ls: Operation = Text(format!("{{V_{{{}, {}}}}}", node1, node2));
        let rs: Operation = Text(format!(
            "voltage and current from node {} to node {}",
            node1, node2
        ));
        sub_steps.push(Equal(Some(Box::new(ls)), Some(Box::new(rs))));
    });
    Step {
        label: "Voltage Sources have 0 resistance.".to_string(),
        sub_steps: Some(sub_steps),
    }
}

fn breakdown_voltage_src_equations(
    node_pairs: &Vec<(usize, usize, Rc<Element>)>,
    container: &Container,
) -> Step {
    let mut eq_steps: Vec<Operation> = Vec::new();
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

            eq_steps.push(Equal(
                Some(Box::new(Variable(element.clone()))),
                Some(Box::new(Sum(vec![tool1, tool2]))),
            ))
        });

    Step {
        label: "Find voltage across each voltage source:".to_string(),
        sub_steps: Some(eq_steps),
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

    summation_steps = summation_steps
        .iter()
        .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
        .collect();

    let init_eq = Equal(
        Some(Box::new(Value(0.0))),
        Some(Box::new(Sum(summation_steps.clone()))),
    );
    summation_steps = summation_steps
        .iter()
        .map(|x| expand(x.clone()).unwrap_or_else(|_| x.clone()))
        .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
        .collect();
    let mut sum: Operation = Sum(summation_steps.clone());
    sum = sum.simplify().unwrap_or_else(|| sum.clone());
    let expanded = Equal(Some(Box::new(Value(0.0))), Some(Box::new(sum.clone())));

    sum.apply_variables();
    let mut coefficients: Vec<Operation> = Vec::new();
    if let Sum(mut list) = sum.clone() {
        coefficients = list
            .iter_mut()
            .map(|x| x.get_coefficient())
            .map(|x| {
                if let Some(coeff) = x {
                    Value(coeff)
                } else {
                    Value(0.0)
                }
            })
            .collect();
    }

    let resistor_values: Operation = Equal(Some(Box::new(Value(0.0))), Some(Box::new(sum.clone())));

    Step {
        label: "Find current through each resistor:".to_string(),
        sub_steps: Some(vec![init_eq, expanded, resistor_values, Sum(coefficients)]),
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
