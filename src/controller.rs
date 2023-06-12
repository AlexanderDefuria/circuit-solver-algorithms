use crate::container::Container;
use crate::simplification;
use crate::tools::ToolType;

/// This will be the main interface for the user to interact with the program.
///
/// Control and options will be processed here, calling setup and solving steps
/// as needed. Major program control and logic are within the controller. This
/// should be completed with a GUI, after the container is done V1. Most likely
/// this will begin development when the solver is structurally complete or V0.1.
pub struct Controller {
    pub container: Container,
}


pub enum Operations {
    Simplify(simplification::Method),
    Solve(ToolType),
    Validate,
}
