pub(crate) mod helpers {
    use crate::components::Component::{CurrentSrc, Ground, Resistor, VoltageSrc};
    use crate::container::Container;
    use crate::elements::Element;

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

    pub fn create_basic_super_mesh_container() -> Container {
        let mut container = Container::new();
        container.add_element_core(Element::new(Ground, 0., vec![1, 2, 3, 4], vec![]));
        container.add_element_core(Element::new(VoltageSrc, 3., vec![5], vec![0, 2, 3, 4]));
        container.add_element_core(Element::new(CurrentSrc, 1.5, vec![0, 1, 3, 4], vec![5, 6]));
        container.add_element_core(Element::new(Resistor, 2., vec![6, 7], vec![0, 1, 2, 4]));
        container.add_element_core(Element::new(CurrentSrc, 2., vec![0, 1, 2, 3], vec![7]));
        container.add_element_core(Element::new(Resistor, 2., vec![1], vec![2, 6]));
        container.add_element_core(Element::new(Resistor, 4., vec![5, 2], vec![3, 7]));
        container.add_element_core(Element::new(Resistor, 1., vec![3, 7], vec![4]));
        container
    }
}

#[cfg(test)]
mod tests {
    use crate::components::Component;
    use crate::elements::Element;
    use crate::test_support::helpers::create_basic_container;
    use assert_json_diff::assert_json_include;
    use serde_json::json;

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

        println!("{:?}", create_basic_container());
    }
}
