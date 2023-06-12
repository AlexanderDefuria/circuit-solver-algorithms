use std::fmt::{Debug, Display, Formatter};
use crate::simplification::Method;

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

