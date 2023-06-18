use crate::components::Component;
use crate::components::Component::Ground;
use crate::util::PrettyString;
use crate::validation::Status::Valid;
use crate::validation::StatusError::Known;
use crate::validation::{Validation, ValidationResult};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;
use std::rc::Rc;

/// Representation of a Schematic Element
#[derive(Debug, Serialize, Deserialize)]
pub struct Element {
    #[serde(skip_deserializing)]
    pub(crate) name: String,
    pub(crate) id: usize,
    pub(crate) value: f64,
    #[serde(skip_deserializing)]
    pub(crate) current: f64,
    #[serde(skip_deserializing)]
    pub(crate) voltage_drop: f64,
    pub(crate) class: Component,
    pub(crate) positive: Vec<usize>, // Link to other elements
    pub(crate) negative: Vec<usize>,
}

impl Element {
    /// Create a new Element
    ///
    /// It is recommended to use the `circuit::add_element` function with this function
    pub fn new(
        class: Component,
        value: f64,
        positive: Vec<usize>,
        negative: Vec<usize>,
    ) -> Element {
        if class == Ground {
            // Ground element cannot have dual polarity
            let connections = [&positive[..], &negative[..]].concat();
            return Element::new_full(class, 0.0, connections, vec![], 0);
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
            current: 0.0,
            voltage_drop: 0.0,
            class,
            positive,
            negative,
        }
    }

    pub(crate) fn contains_ground(&self) -> bool {
        self.positive.contains(&0) || self.negative.contains(&0)
    }
}

impl PrettyString for Element {
    fn pretty_string(&self) -> String {
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
                    return Err(Known(format!(
                        "Value cannot be zero or negative {}",
                        self.pretty_string()
                    )));
                }

                for x in self.positive.iter() {
                    if self.negative.contains(x) {
                        return Err(Known(format!(
                            "Element cannot be shorted {}",
                            self.pretty_string()
                        )));
                    }
                }
            }
        }
        if self.positive.len() == 0 && self.negative.len() == 0 {
            return Err(Known("Element has no connections".to_string()));
        }

        if self.positive.contains(&self.id) || self.negative.contains(&self.id) {
            return Err(Known(format!(
                "Element cannot be connected to itself {}",
                self.pretty_string()
            )));
        }

        Ok(Valid)
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
    use crate::assert_known_error;
    use crate::components::Component;
    use crate::elements::Element;
    use crate::validation::StatusError::Known;
    use crate::validation::Validation;

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

    #[test]
    fn test_validate() {
        let mut a = Element::new(Component::Resistor, 1.0, vec![1], vec![2]);
        assert!(a.validate().is_ok());
        a.value = -0.5;
        assert_known_error!(a.validate(), "Value cannot be zero or negative R0: -0.5 Ω");

        let b = Element::new(Component::Resistor, 1.0, vec![1], vec![1]);
        assert_known_error!(b.validate(), "Element cannot be shorted R0: 1 Ω");

        let mut c = Element::new(Component::Resistor, 1.0, vec![1], vec![2]);
        c.id = 1;
        assert_known_error!(
            c.validate(),
            "Element cannot be connected to itself R1: 1 Ω"
        );

        let d = Element {
            name: "".to_string(),
            id: 0,
            value: 0.0,
            current: 0.0,
            voltage_drop: 0.0,
            class: Component::Ground,
            positive: vec![1],
            negative: vec![2],
        };
        assert_known_error!(d.validate(), "Ground element cannot have dual polarity");

        let mut e = Element::new(Component::Ground, 1.0, vec![1], vec![]);
        e.value = 1.0;
        assert_known_error!(e.validate(), "Ground element cannot have a value");

        let f = Element::new(Component::Resistor, 1.0, vec![], vec![]);
        assert_known_error!(f.validate(), "Element has no connections");
    }
}
