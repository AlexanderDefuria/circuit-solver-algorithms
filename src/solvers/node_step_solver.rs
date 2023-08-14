use crate::component::Component::{Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;
use crate::solvers::solver::{Solver, Step};
use operations::math::EquationMember;
use operations::operations::Operation;
use operations::prelude::{Divide, Equal, Negate, Sum, Text, Value};
use std::cell::RefCell;
use std::rc::Rc;

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
        steps.push(Step {
            label: "Steps to solve the circuit:".to_string(),
            sub_steps: None,
        });

        // Step 1 Declare
        let mut sub_steps: Vec<Operation> = Vec::new();
        node_pairs.iter().for_each(|(node1, node2, element)| {
            if element.class == VoltageSrc {
                let ls: Operation = Text(format!("{{V_{{{}, {}}}}}", node1, node2));
                let rs: Operation = Text(format!(
                    "voltage & current from node {} to node {}",
                    node1, node2
                ));
                sub_steps.push(Equal(Some(Box::new(ls)), Some(Box::new(rs))));
            }
        });
        steps.push(Step {
            label: "Voltage Sources have 0 resistance.".to_string(),
            sub_steps: Some(sub_steps),
        });

        // Step 2 Find Voltages
        // Step 2.1 Find voltages
        let mut summation_steps: Vec<Operation> = Vec::new();
        // Step 2.1.1 Find all resistors going between nodes including ground
        let resistor_node_pairs: Vec<&(usize, usize, Rc<Element>)> = node_pairs
            .iter()
            .filter(|(node1, node2, element)| element.class == Resistor)
            .collect::<Vec<&(usize, usize, Rc<Element>)>>();
        resistor_node_pairs
            .iter()
            .for_each(|(node1, node2, element)| {
                let mut tools: Vec<Operation> = Vec::new();
                let mut id_1 = *node1;
                let mut id_2 = *node2;

                if *node1 != 0 {
                    id_1 -= 1;
                    tools.push(Text(
                        self.container.borrow().get_tool_by_id(id_1).latex_string(),
                    ));
                }
                if *node2 != 0 {
                    id_2 -= 1;
                    tools.push(Negate(Some(Box::new(Text(
                        self.container.borrow().get_tool_by_id(id_2).latex_string(),
                    )))));
                }

                summation_steps.push(Negate(Some(Box::new(Divide(
                    Some(Box::new(Sum(tools).simplify().unwrap())),
                    Some(Box::new(Text(element.latex_string()))),
                )))));
            });

        summation_steps = summation_steps
            .iter()
            .map(|x| x.simplify().unwrap_or_else(|| x.clone()))
            .collect();

        steps.push(Step {
            label: "Find current through each resistor:".to_string(),
            sub_steps: Some(vec![Equal(
                Some(Box::new(Value(Rc::new(0.0)))),
                Some(Box::new(Sum(summation_steps.clone()))),
            )]),
        });

        let mut eq_steps: Vec<Operation> = Vec::new();
        // Step 2.1.2 Find all voltage sources going between nodes including ground
        let voltage_src_node_pairs: Vec<&(usize, usize, Rc<Element>)> = node_pairs
            .iter()
            .filter(|(node1, node2, element)| element.class == VoltageSrc)
            .collect::<Vec<&(usize, usize, Rc<Element>)>>();
        voltage_src_node_pairs
            .iter()
            .for_each(|(node1, node2, element)| {
                let mut tool2: Operation = Value(Rc::new(0.0));
                let mut tool1: Operation = Value(Rc::new(0.0));
                let mut id_1 = *node1;
                let mut id_2 = *node2;
                if *node1 != 0 {
                    id_1 -= 1;
                    tool1 = Value(self.container.borrow().get_tool_by_id(id_1).clone());
                }
                if *node2 != 0 {
                    id_2 -= 1;
                    tool2 = Value(self.container.borrow().get_tool_by_id(id_2).clone());
                }

                tool2 = Negate(Some(Box::new(tool2)));

                eq_steps.push(Equal(
                    Some(Box::new(Value(Rc::new(element.value)))),
                    Some(Box::new(Sum(vec![tool1, tool2]))),
                ))
            });

        steps.push(Step {
            label: "Find voltage across each voltage source:".to_string(),
            sub_steps: Some(eq_steps),
        });

        Ok(steps)
    }
}
