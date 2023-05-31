use crate::components::Component::{Ground, VoltageSrc};
use crate::elements::Element;
use crate::simplification::Simplification;
use crate::tools::{Tool, ToolType};
use crate::util::PrettyString;
use crate::validation::StatusError::Known;
use crate::validation::{
    check_duplicates, get_all_internal_status_errors, Status, StatusError, Validation,
    ValidationResult,
};
use std::fmt::{Debug, Formatter};
use std::rc::{Rc, Weak};

/// Current State of an Element
#[derive(Debug)]
enum SolutionState {
    Solved,
    Unknown,
    Partial,
}

/// Representation of a Schematic Container
///
/// Container is a collection of Elements and Tools we are using to solve the circuit
pub struct Container {
    elements: Vec<Rc<Element>>,
    tools: Vec<Rc<Tool>>,
    simplifications: Vec<Rc<Simplification>>,
    ground: usize,
    state: SolutionState,
}

/// Container is a collection of Elements and Tools we are using to solve the circuit
/// All Elements and Tools are stored in a Vec and are referenced by their index in the Vec
/// All Functions within Container are used to build out the circuit correctly.
///
/// <br>
impl Container {
    fn new() -> Container {
        Container {
            elements: Vec::new(),
            tools: Vec::new(),
            simplifications: vec![],
            ground: 0,
            state: SolutionState::Unknown,
        }
    }

    /// Add an Element to the Container
    ///
    /// This function will add an Element to the Container and return the index of the Element
    pub fn add_element(&mut self, element: Element) -> Result<usize, StatusError> {
        let id: usize = self.add_element_core(element);
        let check = self.validate();
        if check.is_err() {
            self.elements.pop();
            return Err(check.unwrap_err());
        }
        Ok(id)
    }

    fn add_element_core(&mut self, mut element: Element) -> usize {
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
            tool.id = 0;
        }
        self.tools.push(Rc::new(tool));
    }

    fn get_element_by_id(&self, id: usize) -> &Rc<Element> {
        self.elements.get(id).unwrap()
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

    // pub fn create_mesh(&mut self) {
    //
    // }
    //
    // pub fn create_super_mesh(&mut self) {}
    // pub fn create_thevenin(&mut self) {}
    // pub fn create_norton(&mut self) {}
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
    use crate::components::Component::{Ground, Resistor, VoltageSrc};
    use crate::container::Container;
    use crate::elements::Element;
    use crate::tools::ToolType::SuperNode;
    use crate::validation::Status::Valid;
    use crate::validation::{StatusError, Validation};
    use regex::Regex;
    use std::rc::Rc;

    fn create_basic_container() -> Container {
        let mut container = Container::new();
        container.add_element_core(Element::new(VoltageSrc, 1.0, vec![2, 3], vec![1]));
        container.add_element_core(Element::new(Resistor, 1.0, vec![0], vec![2]));
        container.add_element_core(Element::new(Resistor, 1.0, vec![1], vec![0, 3]));
        container.add_element_core(Element::new(Ground, 1.0, vec![0, 2], vec![]));
        container
    }

    fn create_basic_supernode_container() -> Container {
        let mut container = Container::new();
        container.add_element_core(Element::new(Ground, 10., vec![5, 3], vec![]));
        container.add_element_core(Element::new(VoltageSrc, 10., vec![4], vec![2, 3]));
        container.add_element_core(Element::new(Resistor, 10., vec![1, 3], vec![4, 5]));
        container.add_element_core(Element::new(Resistor, 10., vec![1, 2], vec![0, 5]));
        container.add_element_core(Element::new(Resistor, 10., vec![1], vec![2, 5]));
        container.add_element_core(Element::new(VoltageSrc, 10., vec![2, 4], vec![0, 3]));
        container
    }

    #[test]
    fn test_debug() {
        let re = Regex::new(
            r#"Container \{ elements: \["R0: 1 Ω", "R1: 1 Ω"], tools: \[], state: Unknown }"#,
        )
        .unwrap();

        let mut container = Container::new();
        container.add_element_core(Element::new(Resistor, 1.0, vec![2], vec![3]));
        container.add_element_core(Element::new(Resistor, 1.0, vec![2], vec![3]));
        assert!(re.is_match(&format!("{:?}", container)));
    }

    #[test]
    fn test_validate() {
        let mut container = create_basic_container();
        assert_eq!(container.validate(), Ok(Valid));
        assert_eq!(container.validate(), Ok(Valid));

        // Test duplicate elements
        let element = Element::new(Resistor, 1.0, vec![2], vec![3]);
        container.elements.push(Rc::new(element));
        assert!(container.validate().is_err());

        // Test no sources
        container.elements.remove(3);
        assert!(container.validate().is_err());

        // Test multiple grounds
        container = create_basic_container();
        container
            .elements
            .push(Rc::from(Element::new(Ground, 1.0, vec![2], vec![])));
        assert!(container.validate().is_err());
    }

    #[test]
    fn test_add_element() {
        let mut container = create_basic_container();

        // Test add_element with invalid element
        let result: Result<usize, StatusError> =
            container.add_element(Element::new(Ground, 1.0, vec![2], vec![]));
        assert!(result.is_err());

        let result: Result<usize, StatusError> =
            container.add_element(Element::new(Resistor, 1.0, vec![2], vec![]));
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_nodes() {
        let mut container = create_basic_container();
        let x = container.create_nodes();
        let test_vectors = vec![
            vec![x.elements[0].id, x.elements[1].id],
            vec![x.elements[1].id, x.elements[2].id],
        ];
        println!("{:?}", x);
        assert_eq!(x.validate(), Ok(Valid));
        assert_eq!(x.tools.len(), test_vectors.len());

        for test in 0..test_vectors.len() {
            for (i, c) in x.tools[test].members.iter().enumerate() {
                assert_eq!(test_vectors[test][i], c.upgrade().unwrap().id);
            }
        }
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
            .field("state", &self.state)
            .finish()
    }
}
