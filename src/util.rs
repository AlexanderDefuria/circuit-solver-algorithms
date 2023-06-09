use std::fmt::{Debug, Display, Formatter};

pub(crate) trait PrettyString {
    fn pretty_string(&self) -> String;
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

/// Current State of an Element
pub(crate) struct SolutionState {
    node: bool,
    mesh: bool,
    super_node: bool,
    super_mesh: bool,
}

impl SolutionState {
    pub(crate) fn new() -> SolutionState {
        SolutionState {
            node: false,
            mesh: false,
            super_node: false,
            super_mesh: false,
        }
    }
}

impl PrettyString for SolutionState {
    fn pretty_string(&self) -> String {
        String::from("Unknown")
    }
}

impl Display for SolutionState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_string())
    }
}

impl Debug for SolutionState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty_string())
    }
}
