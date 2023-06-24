use crate::container::Container;
use crate::elements::Element;
use crate::simplification::Method;
use crate::tools::ToolType;
use crate::validation::{Status, StatusError, Validation, ValidationResult};
use ndarray::Array2;
use serde_json::Value::Array;

/// This will be the main interface for the user to interact with the program.
///
/// Control and options will be processed here, calling setup and solving steps
/// as needed. Major program control and logic are within the controller. This
/// should be completed with a GUI, after the container is done V1. Most likely
/// this will begin development when the solver is structurally complete or V0.1.
pub struct Controller {
    pub container: Container,
    pub operations: Vec<Operation>,
    pub status: ValidationResult,
}

pub struct Operation {
    pub tool: ToolType,
    pub method: Method,
    pub args: Vec<String>,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            container: Container::new(),
            operations: vec![],
            status: Ok(Status::New),
        }
    }

    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    pub fn run(&mut self) -> Result<(), StatusError> {
        Err(StatusError::Known("Not Implemented".parse().unwrap()))
    }

    pub fn get_output(&self) -> Result<String, StatusError> {
        Err(StatusError::Known("Not Implemented".parse().unwrap()))
    }

    pub fn load_from_file(x: &str) -> Controller {
        let mut file = std::env::current_dir().unwrap();
        file.push(x);
        let contents = std::fs::read_to_string(&file).unwrap();
        Controller::load_from_json(&contents).unwrap()
    }

    pub fn load_from_json(json_str: &str) -> Result<Controller, StatusError> {
        let json: Vec<Element> = serde_json::from_str(&json_str).unwrap();
        let mut controller: Controller = json.into();
        if controller.status.is_err() {
            return Err(controller.status.unwrap_err());
        }
        Ok(controller)
    }
}

impl From<Vec<Element>> for Controller {
    fn from(elements: Vec<Element>) -> Controller {
        let mut container: Container = Container::new();
        for element in elements {
            container.add_element_core(element);
        }
        let status = container.validate();
        Controller {
            container,
            operations: vec![],
            status,
        }
    }
}

impl Validation for Controller {
    fn validate(&self) -> ValidationResult {
        self.status.clone()
    }
}

fn nodal_analysis() {
    let mut matrix = Array2::<f64>::zeros((3, 3)); // 3 by 3 Matrix
}

#[cfg(test)]
mod tests {
    use crate::controller::Controller;
    use crate::helpers::create_basic_container;

    #[test]
    fn test_load() {
        let controller = Controller::load_from_file("tests/data/basic_container.json");
        assert_eq!(create_basic_container().get_elements(), controller.container.get_elements());
    }

    #[test]
    fn test_load_json() {
        let mut file = std::env::current_dir().unwrap();
        file.push("tests/data/basic_container.json");
        let contents = std::fs::read_to_string(&file).unwrap();

        let controller = Controller::load_from_json(&contents).unwrap();
        assert_eq!(create_basic_container().get_elements(), controller.container.get_elements());
    }

    #[test]
    fn test_controller() {

    }
}