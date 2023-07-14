use crate::component::Component::{CurrentSrc, Ground, Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;


pub(crate) trait PrettyPrint {
    fn pretty_string(&self) -> String;
    fn basic_string(&self) -> String;
    fn latex_string(&self) -> String {
        self.pretty_string()
    }
}

#[macro_export]
macro_rules! assert_known_error {
    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (Err(Known(left)), str) => assert_eq!(left, &str.to_string()),
            _ => assert!(false),
        }
    };
}

pub fn create_basic_container() -> Container {
    let mut container = Container::new();
    container.add_element_core(Element::new(VoltageSrc, 1.0, vec![2, 3], vec![1]));
    container.add_element_core(Element::new(Resistor, 1.0, vec![0], vec![2]));
    container.add_element_core(Element::new(Resistor, 1.0, vec![1], vec![0, 3]));
    container.add_element_core(Element::new(Ground, 1.0, vec![0, 2], vec![]));
    container
}

pub fn create_basic_supernode_container() -> Container {
    let mut container = Container::new();
    container.add_element_core(Element::new(Ground, 10., vec![5, 3], vec![]));
    container.add_element_core(Element::new(VoltageSrc, 10., vec![4], vec![2, 3]));
    container.add_element_core(Element::new(Resistor, 10., vec![1, 3], vec![4, 5]));
    container.add_element_core(Element::new(Resistor, 10., vec![1, 2], vec![0, 5]));
    container.add_element_core(Element::new(Resistor, 10., vec![1], vec![2, 5]));
    container.add_element_core(Element::new(VoltageSrc, 10., vec![2, 4], vec![0, 3]));
    container
}

pub fn create_basic_supermesh_container() -> Container {
    let mut container = Container::new();
    container.add_element_core(Element::new(Ground, 0., vec![1, 2, 3, 4], vec![]));
    container.add_element_core(Element::new(VoltageSrc, 3., vec![5], vec![0, 2, 3, 4]));
    container.add_element_core(Element::new(CurrentSrc, 1.5, vec![0, 1, 3, 4], vec![5, 6]));
    container.add_element_core(Element::new(Resistor, 2., vec![6, 7], vec![0, 1, 2, 4]));
    container.add_element_core(Element::new(CurrentSrc, 2., vec![7], vec![0, 1, 2, 3]));
    container.add_element_core(Element::new(Resistor, 2., vec![1], vec![2, 6]));
    container.add_element_core(Element::new(Resistor, 4., vec![5, 2], vec![3, 7]));
    container.add_element_core(Element::new(Resistor, 1., vec![3, 6], vec![4]));
    container
}

pub fn create_mna_container() -> Container {
    let mut container = Container::new();
    container.add_element_core(Element::new(Ground, 0., vec![1, 3, 5], vec![]));
    container.add_element_core(Element::new(Resistor, 2., vec![0], vec![4]));
    container.add_element_core(Element::new(Resistor, 4., vec![5], vec![3, 4]));
    container.add_element_core(Element::new(Resistor, 8., vec![2, 4], vec![0]));
    container.add_element_core(Element::new(VoltageSrc, 32., vec![1], vec![2, 3]));
    container.add_element_core(Element::new(VoltageSrc, 20., vec![2], vec![0]));
    container
}

#[cfg(test)]
mod tests {
    use crate::component::Component;
    use crate::elements::Element;
    use crate::util::{create_basic_container, create_basic_supermesh_container, create_basic_supernode_container, create_mna_container};
    use assert_json_diff::assert_json_include;
    use serde_json::json;
    use crate::validation::Validation;

    #[test]
    fn test_create_containers() {
        let mut container = create_basic_container();
        let mut supernode_container = create_basic_supernode_container();
        let mut supermesh_container = create_basic_supermesh_container();
        let mut mna_container = create_mna_container();

        container.validate().expect("");
        supernode_container.validate().expect("");
        supermesh_container.validate().expect("");
        mna_container.validate().expect("");

        println!("create_basic_container() -> {:?}", container);
        println!("create_basic_supernode_container() -> {:?}", supernode_container);
        println!("create_basic_super_mesh_container() -> {:?}", supermesh_container);
        println!("create_mna_container() -> {:?}", mna_container);
    }

    #[test]
    fn test_serde() {
        let json = json!({
            "name": "R1",
            "id": 1,
            "value": 1.0,
            "class": "Resistor",
            "positive": [2],
            "negative": [3]
        });
        let element: Element = Element {
            name: "R1".to_string(),
            id: 1,
            value: 1.0,
            current: 0.0,
            voltage_drop: 0.0,
            class: Component::Resistor,
            positive: vec![2],
            negative: vec![3],
        };
        assert_eq!(element.name, "R1");
        assert_json_include!(actual: element, expected: json);
    }
}
