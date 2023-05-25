use crate::components::Component;
use crate::components::Component::Ground;
use crate::validation::Status::Valid;
use crate::validation::StatusError::{Known, Unknown};
use crate::validation::{Validation, ValidationResult};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Representation of a Schematic Element
#[derive(Serialize, Deserialize, Debug)]
pub struct Element {
    pub(crate) name: String,
    pub(crate) id: usize,
    pub(crate) value: f64,
    pub(crate) class: Component,
    pub(crate) positive: Vec<usize>, // Link to other elements
    pub(crate) negative: Vec<usize>,
}

impl Element {
    /// Create a new Element
    ///
    /// It is recommended to use the `circuit::add_element` function with this function
    pub(crate) fn new(
        class: Component,
        value: f64,
        positive: Vec<usize>,
        negative: Vec<usize>,
    ) -> Element {
        if class == Ground {
            if positive.len() != 0 && negative.len() != 0 {
                panic!("Ground element cannot have dual polarity");
            } else {
                return Element::new_full(class, 0.0, positive, negative, 0);
            }
        }

        Element::new_full(class, value, positive, negative, 0)
    }

    fn new_full(
        class: Component,
        value: f64,
        positive: Vec<usize>,
        negative: Vec<usize>,
        id: usize,
    ) -> Element {
        Element {
            name: class.basic_string(),
            id,
            value,
            class,
            positive,
            negative,
        }
    }

    /// Return a pretty string representation of the Element
    pub(crate) fn pretty_string(&self) -> String {
        format!(
            "{}{}: {} {}",
            self.name,
            self.id,
            self.value,
            self.class.unit_string()
        )
    }
}

/// Implement PartialEq for Element
///
/// Compare two Elements by their id
impl PartialEq<Self> for Element {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Validation for Element {
    fn validate(&self) -> ValidationResult {
        match self.class {
            Ground => {
                if self.positive.len() != 0 && self.negative.len() != 0 {
                    return Err(Known(
                        "Ground element cannot have dual polarity".to_string(),
                    ));
                }
                if self.value != 0.0 {
                    return Err(Known("Ground element cannot have a value".to_string()));
                };
            }
            _ => {
                // TODO: Check if the element is valid for other components
                // Resistor, Capacitor, Inductor, VoltageSource, CurrentSource
                if self.value <= 0.0 {
                    return Err(Known("Value cannot be zero or negative".to_string()));
                }
                if self.negative.iter().any(|x| x == &self.id)
                    || self.positive.iter().any(|x| x == &self.id)
                {
                    return Err(Known(format!(
                        "Element cannot be connected to itself\n{}",
                        self.pretty_string()
                    )));
                }
            }
        }
        if self.positive.len() == 0 && self.negative.len() == 0 {
            return Err(Known("Element has no connections".to_string()));
        }

        Ok(Valid)
    }

    fn clean(&mut self) -> &Self {
        self.name = self.name.chars().filter(|c| !c.is_digit(10)).collect();
        self
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.validate().unwrap();
        write!(f, "{}", self.pretty_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::components::Component;
    use crate::elements::Element;

    #[test]
    fn test_new() {
        let element = Element::new(Component::Resistor, 1.0, vec![1], vec![2]);
        assert_eq!(element.name, "R");
        assert_eq!(element.id, 0);
        assert_eq!(element.value, 1.0);
        assert_eq!(element.class, Component::Resistor);
        assert_eq!(element.positive, vec![1]);
        assert_eq!(element.negative, vec![2]);

        let element = Element::new(Component::Ground, 1.0, vec![1], vec![]);
        assert_eq!(element.name, "GND");
        assert_eq!(element.id, 0);
        assert_eq!(element.value, 0.0);
        assert_eq!(element.class, Component::Ground);
        assert_eq!(element.positive, vec![1]);
        assert_eq!(element.negative, Vec::<usize>::new());
    }
}
