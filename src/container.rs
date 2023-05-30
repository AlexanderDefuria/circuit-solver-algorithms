use crate::components::Component;
use crate::components::Component::Ground;
use crate::elements::Element;
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
#[derive(PartialEq)]
enum SolutionState {
    Solved,
    Unknown,
    Partial,
}

/// Representation of a Schematic Container
///
/// Container is a collection of Elements and Tools we are using to solve the circuit
#[derive(PartialEq)]
pub struct Container {
    elements: Vec<Rc<Element>>,
    tools: Vec<Rc<Tool>>,
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

    fn add_tool(mut tool: Tool, tool_list: &mut Vec<Rc<Tool>>) {
        if !tool_list.is_empty() {
            let new_id: usize = tool_list.get(tool_list.len() - 1).unwrap().id + 1;
            tool.id = new_id;
        } else {
            tool.id = 0;
        }
        tool_list.push(Rc::new(tool));
    }

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
    pub fn check_validity(&self) -> ValidationResult {
        self.validate()
    }

    /// Create the Nodes and add them to the Container Tools
    ///
    /// This process can be done by sampling one side of every element and then
    /// comparing the samples to see if they are the same. If they are the same
    /// then they are connected and should be added to the same node.
    /// By by filtering our duplicates we can create a pure list of nodes.
    pub fn create_nodes(&mut self) -> &mut Self {
        for element in &self.elements {
            // Need a list of all elements connected to the positive side node.
            let mut node_elements: Vec<Weak<Element>> = element
                .positive
                .iter()
                .filter_map(|positive_id: &usize| {
                    self.elements
                        .iter()
                        .find(|container_element| container_element.id == *positive_id)
                })
                .map(|x| Rc::downgrade(x))
                .collect();
            node_elements.push(Rc::downgrade(element));

            // Check if the node already exists
            if !self.tools.iter().any(|x| {
                // Only care about nodes
                if x.class == ToolType::Node {
                    x.elements.iter().all(|tool_element| {
                        node_elements.iter().any(|node_element| {
                            node_element.upgrade().unwrap().id == tool_element.upgrade().unwrap().id
                        })
                    })
                } else {
                    false
                }
            }) {
                // Create the node and add it to the tools
                Container::add_tool(Tool::create_node(node_elements), &mut self.tools);
            }
        }

        self
    }

    pub fn create_super_nodes(&mut self) -> &mut Self {
        'element: for element in &self.elements {
            match element.class {
                Component::VoltageSrc => {
                    // Check that the source is not connected to a ground
                    if element.positive.contains(&self.ground)
                        || element.negative.contains(&self.ground)
                    {
                        continue 'element;
                    }
                    // Get the positive and negative nodes
                    let positive_node: &Tool = self
                        .tools
                        .iter()
                        .find(|tool| tool.contains(Rc::downgrade(&element)))
                        .unwrap();
                    let negative_node: &Tool = self
                        .tools
                        .iter()
                        .find(|tool|
                            tool.contains(Rc::downgrade(&element))
                            &&
                            tool.id != positive_node.id)
                        .unwrap();

                    Container::add_tool(Tool::create_supernode(vec![]), &mut self.tools);
                }
                _ => {}
            }
        }
        self
    }

    fn add_tool_self(&mut self, tool: Tool) {
        Container::add_tool(tool, &mut self.tools);
    }

    pub fn create_mesh(&mut self) {}
    pub fn create_super_mesh(&mut self) {}
    pub fn create_thevenin(&mut self) {}
    pub fn create_norton(&mut self) {}
}

impl Validation for Container {
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
    use crate::validation::{Status, StatusError, Validation};
    use std::rc::Rc;

    use crate::tools::ToolType::SuperNode;
    use crate::validation::Status::Valid;
    use regex::Regex;

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
        let re = Regex::new(r#"Container \{ elements: \["R0: 1 Ohm", "R1: 1 Ohm"] }"#).unwrap();

        let mut container = Container::new();
        container.add_element_core(Element::new(Resistor, 1.0, vec![2], vec![3]));
        container.add_element_core(Element::new(Resistor, 1.0, vec![2], vec![3]));
        assert!(re.is_match(&format!("{:?}", container)));
    }

    #[test]
    fn test_validate() {
        let mut container = create_basic_container();
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
    }

    #[test]
    fn test_create_nodes() {
        let mut container = create_basic_container();
        let x = container.create_nodes();
        assert_eq!(x.validate(), Ok(Valid));
        assert_eq!(x.tools.len(), 3);

        let mut test_vectors = vec![
            vec![x.elements[2].id, x.elements[3].id, x.elements[0].id],
            vec![x.elements[0].id, x.elements[1].id],
            vec![x.elements[1].id, x.elements[2].id],
        ];
        for test in 0..3 {
            for (i, c) in x.tools[test].elements.iter().enumerate() {
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
        // assert_eq!(
        //     container.tools.iter().find(|x| x.class == SuperNode).unwrap(),
        //     Tool {
        //         id: 0,
        //         class: SuperNode,
        //         elements: vec![],
        //     }
        // )
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
            // .field("tools", &self.tools)
            // .field("ground", &self.ground)
            // .field("state", &self.state)
            .finish()
    }
}
