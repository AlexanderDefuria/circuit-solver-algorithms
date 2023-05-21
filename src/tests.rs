#[cfg(test)]
mod tests {
    use crate::components::Component;
    use crate::elements::Element;
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
            class: Component::Resistor,
            positive: vec![2],
            negative: vec![3],
        };
        assert_eq!(element.name, "R1");
        assert_json_include!(actual: element, expected: json);
    }
}
