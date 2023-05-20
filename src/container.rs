use crate::components::Component::Ground;
use crate::elements::Element;
use crate::tools::Tool;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fmt::Debug;

/// Current State of an Element
#[derive(Serialize, Deserialize, PartialEq)]
enum State {
    Solved,
    Unknown,
    Partial,
}

/// Representation of a Schematic Container
///
/// Container is a collection of Elements and Tools we are using to solve the circuit
#[derive(PartialEq)]
struct Container<'a> {
    elements: Vec<Element>,
    tools: Vec<Tool<'a>>,
    ground: usize,
    state: State,
}

pub(crate) trait Validation {
    fn validate(&self) -> bool;
}

impl Debug for Container<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Container<'_> {
    fn new() -> Container<'static> {
        Container {
            elements: Vec::new(),
            tools: Vec::new(),
            ground: 0,
            state: State::Unknown,
        }
    }

    fn add_element(&mut self, mut element: Element) {
        element.id = self.elements.len();
        self.elements.push(element);
    }
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
impl Validation for Container<'_> {
    fn validate(&self) -> bool {
        let mut valid: bool = true;

        // Check that all elements and tools are valid individually
        // - Floating, Opens, Values, etc.
        valid &= self.elements.iter().all(|x| x.validate());
        valid &= self.tools.iter().all(|x| x.validate());

        // Check that there are no duplicates in elements or tools
        valid &= !self
                .elements
                .iter()
                .any(|x| self.elements.iter().filter(|y| x == *y).count() > 1);
        valid &= !self
                .tools
                .iter()
                .any(|x| self.tools.iter().filter(|y| x == *y).count() > 1);

        // Check that there is at least one source and a single ground
        valid &= self.elements.iter().any(|x| x.class.is_source());
        valid &= (self.elements.iter().filter(|x| x.class == Ground).count() == 1);

        // TODO Shorted Elements

        valid
    }
}

#[cfg(test)]
mod tests {
    use crate::components::Component;
    use crate::components::Component::{Ground, Resistor, VoltageSrc};
    use crate::container::{Container, Validation};
    use crate::elements::Element;
    use crate::*;

    use regex::Regex;

    #[test]
    fn test_debug() {
        let re = Regex::new(r#"Container \{ elements: \["R0: 1 Ohm", "R1: 1 Ohm"] }"#).unwrap();

        let mut container = Container::new();
        container.add_element(Element::new(Resistor, 1.0, vec![2], vec![3]));
        container.add_element(Element::new(Resistor, 1.0, vec![2], vec![3]));
        assert!(re.is_match(&format!("{:?}", container)));
    }

    #[test]
    fn test_validate() {
        let mut container = Container::new();
        container.add_element(Element::new(Resistor, 1.0, vec![2], vec![3]));
        container.add_element(Element::new(Resistor, 1.0, vec![2], vec![3]));
        container.add_element(Element::new(Ground, 1.0, vec![2], vec![]));
        container.add_element(Element::new(VoltageSrc, 1.0, vec![2], vec![3]));
        assert_eq!(container.validate(), true);

        // Test duplicate elements, this should be invalid.
        container.elements[0].id = 1;
        assert_eq!(container.validate(), false);
    }
}
