use crate::component::Component::{Ground, VoltageSrc};
use crate::component::Simplification;
use crate::elements::Element;
use crate::tools::{Tool, ToolType};
use crate::util::PrettyPrint;
use crate::validation::StatusError::Known;
use crate::validation::{
    check_duplicates, get_all_internal_status_errors, Status, StatusError, Validation,
    ValidationResult,
};
use petgraph::graph::UnGraph;
use petgraph::prelude::NodeIndex;
use rustworkx_core::connectivity;

use serde::{Serialize, Serializer};
use std::fmt::{Debug, Formatter};
use std::rc::{Rc, Weak};

/// Representation of a Schematic Container
///
/// Container is a collection of Elements and Tools we are using to solve the circuit
#[derive(Clone, Serialize)]
pub struct Container {
    elements: Vec<Rc<Element>>,
    tools: Vec<Rc<Tool>>,
    simplifications: Vec<Rc<Simplification>>,
    ground: usize,
}

/// Container is a collection of Elements and Tools we are using to solve the circuit
/// All Elements and Tools are stored in a Vec and are referenced by their index in the Vec
/// All Functions within Container are used to build out the circuit correctly.
///
/// <br>
impl Container {
    pub(crate) fn new() -> Container {
        Container {
            elements: Vec::new(),
            tools: Vec::new(),
            simplifications: vec![],
            ground: 0,
        }
    }

    /// Add an Element to the Container
    ///
    /// This function will add an Element to the Container and return the index of the Element
    pub fn add_element(&mut self, mut element: Element) -> Result<usize, StatusError> {
        element.id = self.elements.len();
        element.validate()?;
        let id: usize = self.add_element_core(element);
        let check = self.validate();
        if check.is_err() {
            self.elements.pop();
            return Err(check.unwrap_err());
        }
        Ok(id)
    }

    pub(crate) fn add_element_core(&mut self, mut element: Element) -> usize {
        let id: usize = self.elements.len();
        element.id = id;
        self.elements.push(Rc::new(element));
        id
    }

    fn add_tool(&mut self, mut tool: Tool) {
        if !self.tools.is_empty() {
            let new_id: usize = self.tools.get(self.tools.len() - 1).unwrap().id + 1;
            tool.id = new_id;
        } else {
            tool.id = 1;
        }
        self.tools.push(Rc::new(tool));
    }

    pub(crate) fn get_element_by_id(&self, id: usize) -> &Rc<Element> {
        self.elements.get(id).unwrap()
    }

    pub fn nodes(&self) -> Vec<Weak<Tool>> {
        self.tools
            .iter()
            .filter(|x| x.class == ToolType::Node)
            .map(|x| Rc::downgrade(x))
            .collect()
    }

    pub fn meshes(&self) -> Vec<Weak<Tool>> {
        self.tools
            .iter()
            .filter(|x| x.class == ToolType::Mesh)
            .map(|x| Rc::downgrade(x))
            .collect()
    }

    pub fn components(&self) -> Vec<Weak<Element>> {
        self.elements
            .iter()
            .filter(|x| x.class != Ground)
            .map(|x| Rc::downgrade(x))
            .collect()
    }

    /// Create the Nodes and add them to the Container Tools
    ///
    /// This process can be done by sampling one side of every element and then
    /// comparing the samples to see if they are the same. If they are the same
    /// then they are connected and should be added to the same node.
    /// By by filtering our duplicates we can create a pure list of nodes.
    pub fn create_nodes(&mut self) -> &mut Self {
        let mut new_nodes: Vec<Tool> = Vec::new();

        for element in &self.elements {
            // Need a list of all elements connected to the positive side node.
            let mut node_elements: Vec<Weak<Element>> = element
                .positive
                .iter()
                .map(|positive_id: &usize| self.get_element_by_id(*positive_id))
                .map(|x| Rc::downgrade(x))
                .collect();
            node_elements.push(Rc::downgrade(element)); // Include the element itself

            let ground: bool = node_elements
                .iter()
                .any(|x| x.upgrade().unwrap().class == Ground);
            let duplicate: bool = new_nodes.iter().any(|x| x.contains_all(&node_elements));
            let duplicate_node: bool = self.tools.iter().any(|x| {
                if x.class == ToolType::Node {
                    x.contains_all(&node_elements)
                } else {
                    false
                }
            });

            if ground || duplicate || duplicate_node {
                continue;
            }
            new_nodes.push(Tool::create_node(node_elements));
        }

        for node in new_nodes {
            self.add_tool(node);
        }

        self
    }

    pub fn print_nodes(&self) {
        println!("Nodes:");
        for node in self.nodes() {
            println!(
                "  {} {}",
                node.upgrade().unwrap().pretty_string(),
                node.upgrade().unwrap().basic_string()
            );
        }
    }

    pub fn create_super_nodes(&mut self) -> &mut Self {
        let mut super_nodes: Vec<Tool> = Vec::new();
        let mut valid_sources: Vec<Weak<Element>> = Vec::new();
        for element in &self.elements {
            match element.class {
                VoltageSrc => {
                    if !element.contains_ground() {
                        valid_sources.push(Rc::downgrade(element));
                    }
                }
                _ => continue,
            }
        }

        for source in valid_sources {
            let mut members: Vec<Weak<Element>> = Vec::new();
            for element in &source.upgrade().unwrap().positive {
                members.push(Rc::downgrade(self.get_element_by_id(*element)));
            }
            for element in &source.upgrade().unwrap().negative {
                members.push(Rc::downgrade(self.get_element_by_id(*element)));
            }
            super_nodes.push(Tool::create_supernode(members));
        }

        for node in super_nodes {
            self.add_tool(node);
        }

        self
    }

    pub fn create_meshes(&mut self) -> &mut Self {
        let graph: UnGraph<i32, ()> = Tool::nodes_to_graph(&self.nodes()).unwrap();
        let root = Some(self.ground);
        let x: Vec<Vec<usize>> = connectivity::cycle_basis(&graph, root.map(NodeIndex::new))
            .into_iter()
            .map(|res_map| res_map.into_iter().map(|x| x.index()).collect())
            .collect();

        for mesh in x {
            self.add_tool(Tool::create_mesh(
                mesh.iter()
                    .map(|x| self.get_element_by_id(*x))
                    .map(|x| Rc::downgrade(x))
                    .collect(),
            ));
        }

        self
    }

    pub fn create_super_mesh(&mut self) {}

    pub fn get_elements(&self) -> &Vec<Rc<Element>> {
        &self.elements
    }

    pub fn get_tools(&self) -> &Vec<Rc<Tool>> {
        &self.tools
    }

    pub fn get_ground(&self) -> &Rc<Element> {
        self.elements.get(self.ground).unwrap()
    }

    pub fn simplify(&mut self, _method: &Simplification) -> &mut Self {
        self
    }

    pub fn solve(&mut self, _method: &ToolType) -> &mut Self {
        self
    }

    pub fn get_tools_for_element(&self, element_id: usize) -> Vec<Weak<Tool>> {
        self.tools
            .iter()
            .filter(|x| {
                x.members
                    .iter()
                    .any(|y| y.upgrade().unwrap().id == element_id)
            })
            .map(|x| Rc::downgrade(x))
            .collect()
    }

    pub fn get_voltage_sources(&self) -> Vec<Weak<Element>> {
        self.elements
            .iter()
            .filter(|x| x.class == VoltageSrc)
            .map(|x| Rc::downgrade(x))
            .collect()
    }
}

impl Validation for Container {
    /// Validate the Container and the circuit within are usable.
    ///
    /// This function will check that the Container is in a valid state to be solved.
    /// It will make calls to validate functions in the elements themselves and let
    /// them handle their own internal validation. This will take care of the high
    /// level validation.
    ///
    /// * All Elements have a valid Component, Value, Positive, and Negative
    /// * No duplicate Elements or Tools
    /// * Contains at least one source and a single ground
    /// * No floating Elements, Tools, etc.
    /// * No shorted or open Elements
    fn validate(&self) -> ValidationResult {
        let mut errors: Vec<StatusError> = Vec::new();

        // Check that all elements and tools are valid individually
        errors.append(&mut get_all_internal_status_errors(&self.elements));
        errors.append(&mut get_all_internal_status_errors(&self.tools));

        // Check that there are no duplicates in elements or tools
        errors.append(&mut check_duplicates(&self.elements));
        errors.append(&mut check_duplicates(&self.tools));

        // Check that there is at least one source and a single ground
        if !self.elements.iter().any(|x| x.class.is_source()) {
            errors.push(Known("No Sources".parse().unwrap()));
        }
        if self.elements.iter().filter(|x| x.class == Ground).count() != 1 {
            errors.push(Known("Multiple Grounds".parse().unwrap()));
        }

        match errors.len() {
            0 => Ok(Status::Valid),
            1 => Err(errors[0].clone()),
            _ => Err(StatusError::Multiple(errors)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::Component::{Ground, Resistor};
    use crate::container::Container;
    use crate::elements::Element;
    use crate::tools::ToolType::SuperNode;
    use crate::util::*;
    use crate::validation::Status::Valid;
    use crate::validation::StatusError::Known;
    use crate::validation::{StatusError, Validation};
    use regex::Regex;
    use std::rc::Rc;

    #[test]
    fn test_debug() {
        let re = Regex::new(
            r#"Container \{ elements: \["R0: 1 Ω", "R1: 1 Ω"], tools: \[], state: .+\) }"#,
        )
        .unwrap();

        let mut container = Container::new();
        container.add_element_core(Element::new(Resistor, 1.0, vec![2], vec![3]));
        container.add_element_core(Element::new(Resistor, 1.0, vec![2], vec![3]));
        println!("{:?}", container);
        assert!(re.is_match(&format!("{:?}", container)));
    }

    #[test]
    fn test_validate() {
        let mut container = create_basic_container();
        assert_eq!(container.validate(), Ok(Valid));

        // Test no sources
        container.elements.remove(3);
        assert!(container.validate().is_err());

        // Test multiple grounds
        container = create_basic_container();
        container.add_element_core(Element::new(Ground, 1.0, vec![2], vec![]));
        assert!(container.validate().is_err());
    }

    #[test]
    fn test_add_element() {
        let mut container = create_basic_container();

        // Test add_element with invalid element
        let result: Result<usize, StatusError> =
            container.add_element(Element::new(Ground, 1.0, vec![2], vec![]));
        assert!(result.is_err());

        // Test add_element with valid element
        let result: Result<usize, StatusError> =
            container.add_element(Element::new(Resistor, 1.0, vec![2], vec![]));

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_nodes() {
        let mut container = create_basic_container();
        let x = container.create_nodes();
        let test_vectors = vec![
            vec![x.elements[3].id, x.elements[1].id],
            vec![x.elements[1].id, x.elements[2].id],
        ];

        assert_eq!(x.validate(), Ok(Valid));
        assert_eq!(x.tools.len(), test_vectors.len());

        for test in 0..test_vectors.len() {
            for (i, c) in x.tools[test].members.iter().enumerate() {
                assert_eq!(test_vectors[test][i], c.upgrade().unwrap().id);
            }
        }
    }

    #[test]
    fn test_get_nodes() {
        let mut x = create_basic_container();
        let c = x.create_nodes();
        let test_vectors = vec![
            vec![c.elements[3].id, c.elements[1].id],
            vec![c.elements[1].id, c.elements[2].id],
        ];
        assert_eq!(c.validate(), Ok(Valid));
        assert_eq!(c.tools.len(), test_vectors.len());

        for test in 0..test_vectors.len() {
            for (i, c) in c.tools[test].members.iter().enumerate() {
                assert_eq!(test_vectors[test][i], c.upgrade().unwrap().id);
            }
        }

        let mut x = create_mna_container();
        let c = x.create_nodes();
        assert!(c.validate().is_ok());
    }

    #[test]
    fn test_create_super_nodes() {
        let mut container = create_basic_supernode_container();
        container.create_nodes().create_super_nodes();
        assert_eq!(container.validate(), Ok(Valid));

        // Check that there is only one supernode
        // Expected to be around VoltageSource id: 1
        let expected_super_node_count = 1;
        assert_eq!(
            container
                .tools
                .iter()
                .filter(|x| x.class == SuperNode)
                .count(),
            expected_super_node_count
        );

        let super_node = container
            .tools
            .iter()
            .find(|x| x.class == SuperNode)
            .unwrap();
        let expected_ids: Vec<usize> = vec![2, 3, 4];
        assert_eq!(super_node.members.len(), expected_ids.len());
        for member in super_node.members.iter() {
            assert!(expected_ids.contains(&member.upgrade().unwrap().id));
        }
    }

    #[test]
    fn test_mna_supermesh() {
        let mut container = create_mna_container();
        container.create_nodes().create_super_nodes();
        assert_eq!(container.validate(), Ok(Valid));
    }

    #[test]
    fn test_create_mesh() {
        let mut basic: Container = create_basic_container();
        basic.create_nodes();
        basic.create_meshes();
        assert_eq!(basic.validate(), Ok(Valid));
        assert_eq!(basic.tools.len(), 3);

        let mesh_members: Vec<usize> = vec![0, 1, 2];
        let mesh = basic.meshes().get(0).unwrap().upgrade().unwrap();
        assert_eq!(mesh.members.len(), mesh_members.len());
        for member in mesh.members.iter() {
            assert!(mesh_members.contains(&member.upgrade().unwrap().id),);
        }
    }
}

impl Debug for Container {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field(
                "elements",
                &self
                    .elements
                    .iter()
                    .map(|x| x.pretty_string())
                    .collect::<Vec<String>>(),
            )
            .field("tools", &self.tools)
            .field("state", &self.validate())
            .finish()
    }
}
