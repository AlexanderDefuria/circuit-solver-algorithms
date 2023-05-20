use crate::components::Component;
use crate::components::Component::Ground;
use crate::container::Validation;
use serde::{Deserialize, Serialize};

/// Representation of a Schematic Element
#[derive(Serialize, Deserialize)]
pub(crate) struct Element {
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
    /// It is recommended to use the `circuit::add_element` function instead
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
    ///
    /// ```
    /// let element = Element::new(Resistor, 1.0, vec![1], vec![2]);
    /// assert_eq!(element.pretty_string(), "R1: 1.0 Ohm");
    /// ```
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
    fn validate(&self) -> bool {
        // TODO
        true
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