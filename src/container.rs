use crate::components::Component;
use crate::components::Component::Ground;
use crate::container::SolutionState::Partial;
use crate::elements::Element;
use crate::tools::{Tool, ToolType};
use crate::validation::StatusError::Known;
use crate::validation::{
    check_duplicates, get_all_internal_status_errors, Status, StatusError, Validation,
    ValidationResult,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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
pub struct Container<'a> {
    elements: Vec<Element>,
    tools: Vec<Tool<'a>>,
    ground: usize,
    state: SolutionState,
}

impl Debug for Container<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field(
                "elements",
                &self
                    .elements
                    .iter()
                    .map(|x: &Element| x.pretty_string())
                    .collect::<Vec<String>>(),
            )
            // .field("tools", &self.tools)
            // .field("ground", &self.ground)
            // .field("state", &self.state)
            .finish()
    }
}

/// Container is a collection of Elements and Tools we are using to solve the circuit
/// All Elements and Tools are stored in a Vec and are referenced by their index in the Vec
/// All Functions within Container are used to build out the circuit correctly.
///
/// <br>
impl<'a> Container<'a> {
    fn new() -> Container<'a> {
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
    pub fn add_element(&mut self, mut element: Element) -> Result<usize, StatusError> {
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
        self.elements.push(element);
        id
    }

    fn add_tool<'tool: 'a>(mut tool: Tool<'tool>, tool_list: &mut  Vec<Tool<'tool>>) {
        if !tool_list.is_empty() {
            let new_id: usize = tool_list.get(tool_list.len() - 1).unwrap().id + 1;
            tool.id = new_id;
        } else {
            tool.id = 0;
        }
        tool_list.push(tool);
    }

    /// Manually set the ground element
    ///
    /// This function is used to manually set the ground element.
    /// This should be avoided if possible and will <strong>replace the current ground</strong>.
    fn manually_set_ground(&mut self, ground: Element) {
        assert_eq!(ground.class, Ground);
        self.ground = self.add_element(ground).unwrap();
    }

    /// Set the ground element
    pub fn set_ground(&mut self) -> Result<(), StatusError> {
        let ground: Element = Element::new(Ground, 0.0, vec![], vec![]);
        self.manually_set_ground(ground);
        let check = self.validate();
        if check.is_err() {
            self.elements.pop();
            return Err(check.unwrap_err());
        }
        Ok(())
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
    pub fn create_nodes(&'a mut self) -> &'a Self {
        for element in &self.elements {
            let mut positive_elements: Vec<&Element> = element
                .positive
                .iter()
                .filter_map(|positive_id: &usize| {
                    self.elements
                        .iter()
                        .find(|container_element| container_element.id == *positive_id)
                })
                .collect();

            positive_elements.push(element);

            if !self.tools.iter().any(|x| {
                // Only care about nodes
                if x.pseudo_type == ToolType::Node {
                    x.elements
                        .iter()
                        .all(|tool_elements| positive_elements.contains(tool_elements))
                } else {
                    false
                }
            }) {
                Container::add_tool(Tool::create_node(positive_elements), &mut self.tools);
            }
        }

        self
    }

    pub fn create_mesh(&mut self) {}
    pub fn create_super_mesh(&mut self) {}
    pub fn create_super_nodes(&mut self) {}
    pub fn create_thevenin(&mut self) {}
    pub fn create_norton(&mut self) {}
}

impl Validation for Container<'_> {
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

    use crate::tools::ToolType::Node;
    use regex::Regex;
    use crate::validation::Status::Valid;

    fn create_basic_container<'a>() -> Container<'a> {
        let mut container = Container::new();
        container.add_element_core(Element::new(VoltageSrc, 1.0, vec![2, 3], vec![1]));
        container.add_element_core(Element::new(Resistor, 1.0, vec![0], vec![2]));
        container.add_element_core(Element::new(Resistor, 1.0, vec![1], vec![0, 3]));
        container.add_element_core(Element::new(Ground, 1.0, vec![0, 2], vec![]));
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
        assert_eq!(container.validate(), Ok(Status::Valid));

        // Test duplicate elements, this should be invalid.
        container.elements[0].id = 1;
        assert!(container.validate().is_err());

        // Test no sources, this should be invalid.
        container.elements[3].class = Resistor;
        assert!(container.validate().is_err());

        // Test multiple grounds, this should be invalid.
        container.elements[3].class = VoltageSrc;
        container
            .elements
            .push(Element::new(Ground, 1.0, vec![2], vec![]));
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
        assert_eq!(x.tools[0].elements, vec![&x.elements[2], &x.elements[3], &x.elements[0]]);
        assert_eq!(x.tools[1].elements, vec![&x.elements[0], &x.elements[1]]);
        assert_eq!(x.tools[2].elements, vec![&x.elements[1], &x.elements[2]]);
    }
}
