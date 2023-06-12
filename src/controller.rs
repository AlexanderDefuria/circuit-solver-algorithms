use crate::container::Container;
use crate::simplification::Method;
use crate::tools::ToolType;
use crate::validation::{StatusError, Validation};

/// This will be the main interface for the user to interact with the program.
///
/// Control and options will be processed here, calling setup and solving steps
/// as needed. Major program control and logic are within the controller. This
/// should be completed with a GUI, after the container is done V1. Most likely
/// this will begin development when the solver is structurally complete or V0.1.
pub struct Controller {
    pub container: Container,
    pub operations: Vec<Operations>,
}


pub enum Operations {
    Simplify(Method),
    Solve(ToolType),
    Validate,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            container: Container::new(),
            operations: vec![],
        }
    }

    pub fn add_operation(&mut self, operation: Operations) {
        self.operations.push(operation);
    }

    pub fn run(&mut self) -> Result<(), StatusError> {
        Err(StatusError::Known("Not Implemented".parse().unwrap()))
    }
}


