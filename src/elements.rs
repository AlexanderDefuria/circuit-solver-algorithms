use crate::component::Component;
use crate::component::Component::Ground;
use crate::util::PrettyPrint;
use crate::validation::Status::Valid;
use crate::validation::StatusError::Known;
use crate::validation::{Validation, ValidationResult};
use operations::math::{EquationMember, EquationRepr};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

/// Representation of a Schematic Element
#[derive(Debug, Deserialize, Clone)]
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

    pub(crate) fn new_full(
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

    pub(crate) fn connected_to_ground(&self) -> bool {
        self.positive.contains(&0) || self.negative.contains(&0)
    }

    pub(crate) fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

impl PrettyPrint for Element {
    fn pretty_string(&self) -> String {
        format!(
            "{}{}: {} {}",
            self.name,
            self.id,
            self.value,
            self.class.unit_string()
        )
    }

    fn basic_string(&self) -> String {
        format!("{}{}", self.name, self.id)
    }
}

impl Into<EquationRepr> for Element {
    fn into(self) -> EquationRepr {
        EquationRepr::new_with_latex(
            self.basic_string(),
            format!("{}_{{{}}}", self.name, self.id),
            self.value,
        )
    }
}

impl EquationMember for Element {
    fn equation_repr(&self) -> String {
        self.basic_string()
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn latex_string(&self) -> String {
        format!("{{{}}}_{{{}}}", self.name, self.id)
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

                // TODO: This should be a simplification not a validation?
                for x in self.positive.iter() {
                    if self.negative.contains(x) {
                        // return Err(Known(format!(
                        //     "Element {} cannot be shorted to id {}",
                        //     self.pretty_string(),
                        //     x
                        // )));
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

        if self.id == 0 && self.class != Ground {
            return Err(Known("Element cannot have id 0".to_string()));
        }

        Ok(Valid)
    }

    fn id(&self) -> usize {
        self.id
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.validate().unwrap();
        write!(f, "{}", self.pretty_string())
    }
}

impl Serialize for Element {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Element", 9)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("value", &self.value)?;
        state.serialize_field("current", &self.current)?;
        state.serialize_field("voltage_drop", &self.voltage_drop)?;
        state.serialize_field("class", &self.class)?;
        state.serialize_field("positive", &self.positive)?;
        state.serialize_field("negative", &self.negative)?;
        state.serialize_field("pretty_string", &self.pretty_string())?;
        state.serialize_field("latex_string", &self.latex_string())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_known_error;
    use crate::component::Component;
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
        let mut a = Element::new(Component::Resistor, 1.0, vec![3], vec![2]);
        assert_known_error!(a.validate(), "Element cannot have id 0");
        a.id = 1;
        assert!(a.validate().is_ok());
        a.value = -0.5;
        assert_known_error!(a.validate(), "Value cannot be zero or negative R1: -0.5 Ω");

        // TODO This was a result of removing the short validation. Should this be a validation?
        // let mut b = Element::new(Component::Resistor, 1.0, vec![1], vec![1]);
        // b.id = 1;
        // assert_known_error!(b.validate(), "Element R1: 1 Ω cannot be shorted to id 1");

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
