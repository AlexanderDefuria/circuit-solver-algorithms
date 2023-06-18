#[cfg(test)]
mod integration_tests {
    use circuit_solver_algorithms::controller::Controller;
    use circuit_solver_algorithms::elements::Element;

    #[test]
    fn test() {
        let mut file = std::env::current_dir().unwrap();
        file.push("tests/data/basic_container.json");
        println!("{:?}", file);
        let contents = std::fs::read_to_string(&file).unwrap();
        let json: Vec<Element> = serde_json::from_str(&contents).unwrap();

        let mut controller: Controller = json.into();
    }
}
