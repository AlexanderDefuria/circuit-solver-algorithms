use crate::component::Component::{CurrentSrc, Ground, Resistor, VoltageSrc};
use crate::container::Container;
use crate::elements::Element;

pub(crate) trait PrettyPrint {
    fn pretty_string(&self) -> String;
    fn basic_string(&self) -> String;
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

#[allow(dead_code)]
pub fn create_basic_container() -> Container {
    let mut container = Container::new();
    container.add_element_core(Element::new(Ground, 1.0, vec![3, 2], vec![]));
    container.add_element_core(Element::new(Resistor, 1.0, vec![3], vec![2]));
    container.add_element_core(Element::new(Resistor, 1.0, vec![1], vec![0, 3]));
    container.add_element_core(Element::new(VoltageSrc, 1.0, vec![2, 0], vec![1]));
    container
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn create_four_mesh_container() -> Container {
    // GND(0) at -V1, R1, R2, R3
    let mut container = Container::new();
    container.add_element_core(Element::new(Ground, 0., vec![6], vec![]));
    container.add_element_core(Element::new(Resistor, 1., vec![2], vec![2, 0]));
    container.add_element_core(Element::new(Resistor, 2., vec![1], vec![1, 3, 0]));
    container.add_element_core(Element::new(Resistor, 3., vec![1, 2, 0], vec![5]));
    container.add_element_core(Element::new(Resistor, 5., vec![], vec![1, 5]));
    container.add_element_core(Element::new(Resistor, 7., vec![3], vec![4]));
    container.add_element_core(Element::new(VoltageSrc, 1., vec![1, 4, 5], vec![1, 2, 3]));
    container.add_element_core(Element::new(VoltageSrc, 2., vec![4], vec![3, 5]));
    container
}

#[cfg(test)]
mod tests {
    use crate::container::Container;
    use crate::elements::Element;
    use crate::util::*;
    use crate::validation::Status::Valid;
    use crate::validation::Validation;
    use assert_json_diff::assert_json_include;
    use serde_json::json;

    #[test]
    fn test_create_containers() {

        let mut containers: Vec<Container> = vec![
            create_basic_container(),
            create_basic_supernode_container(),
            create_basic_supermesh_container(),
            create_mna_container(),
        ];

        let mut id: usize = 0;
        containers.iter_mut().for_each(|container| {
            println!("Container {:?}:", id);
            id += 1;

            assert_eq!(container.validate(), Ok(Valid));
            container.create_nodes();
            container.create_super_nodes();
            container.create_meshes();
            container.create_super_meshes();
            println!("{:?}", container.get_elements());

            assert_eq!(container.validate(), Ok(Valid));
        });
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
            class: Resistor,
            positive: vec![2],
            negative: vec![3],
        };
        assert_eq!(element.name, "R1");
        assert_json_include!(actual: element, expected: json);
    }

    // #[test]
    // fn temporary_serialization() {
    //     for mut container in vec![
    //         create_basic_container(),
    //         create_mna_container(),
    //         create_basic_supermesh_container(),
    //         create_basic_supernode_container(),
    //     ] {
    //         container.create_nodes();
    //         container.create_super_nodes();
    //
    //         println!("{}", json!(container).to_string());
    //     }
    // }
}
