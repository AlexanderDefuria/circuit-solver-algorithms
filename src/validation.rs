use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::rc::{Rc, Weak};

use wasm_bindgen::JsValue;

/// Possible Ok Statuses
///
/// Valid: Container is valid
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Status {
    New,
    Valid,
    Simplified,
    Solved,
}

/// Possible Issues
///
/// Valid: Container is valid
#[derive(Debug, Clone, PartialEq)]
pub enum StatusError {
    Unknown,
    Known(String),
    Multiple(Vec<StatusError>),
}

pub type ValidationResult = Result<Status, StatusError>;

pub trait Validation {
    fn validate(&self) -> ValidationResult;
    fn clean(&mut self) -> &Self {
        self
    }
    fn id(&self) -> usize;
    fn class(&self) -> String {
        String::from("Unknown")
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            Status::New => write!(f, "New"),
            Status::Valid => write!(f, "Valid"),
            Status::Simplified => write!(f, "Simplified"),
            Status::Solved => write!(f, "Solved"),
        }
    }
}

impl Display for StatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusError::Unknown => write!(f, "Unknown Issue"),
            StatusError::Known(str) => write!(f, "Known Issue: {}", str),
            StatusError::Multiple(error_list) => {
                write!(f, "Multiple Issues: {:?}", error_list)
            }
        }
    }
}

impl From<String> for StatusError {
    fn from(str: String) -> Self {
        StatusError::Known(str)
    }
}

impl From<StatusError> for JsValue {
    fn from(error: StatusError) -> Self {
        JsValue::from_str(&format!("{}", error))
    }
}

pub(crate) fn get_all_internal_status_errors<T: Validation>(list: &Vec<Rc<T>>) -> Vec<StatusError> {
    list.iter()
        .enumerate()
        .filter_map(|(_, x)| match x.validate() {
            Err(e) => Some(e),
            _ => None,
        })
        .collect()
}

pub(crate) fn check_weak_duplicates<T: Validation + PartialEq>(
    list: &Vec<Weak<RefCell<T>>>,
) -> Vec<StatusError> {
    let references: Vec<Rc<RefCell<T>>> = list.iter().filter_map(|x| x.upgrade()).collect();
    let mut errors: Vec<StatusError> = Vec::new();
    let mut seen: Vec<usize> = Vec::new();
    for x in references {
        if seen.contains(&x.borrow().id()) {
            errors.push(StatusError::Known(format!(
                "Duplicate: {}, {}",
                x.borrow().id(),
                x.borrow().class()
            )));
        }
        seen.push(x.borrow().id());
    }
    errors
}

/// Check for duplicates in a list
///
/// Returns a Vec of StatusError::KnownIssue. If the vec is empty, there are no duplicates.
pub(crate) fn check_duplicates<T: Validation + PartialEq>(list: &Vec<Rc<T>>) -> Vec<StatusError> {
    let mut errors: Vec<StatusError> = Vec::new();
    let mut seen: Vec<usize> = Vec::new();
    for x in list {
        if seen.contains(&x.id()) {
            errors.push(StatusError::Known(format!(
                "Duplicate: {}, {}",
                x.id(),
                x.class()
            )));
        }
        seen.push(x.id());
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printing() {
        let statuses = [(Status::Valid, "Valid"), (Status::Simplified, "Simplified")];

        let errors = [
            (StatusError::Known("Test".to_string()), "Known Issue: Test"),
            (
                StatusError::Multiple(vec![
                    StatusError::Known("Test".to_string()),
                    StatusError::Known("Test2".to_string()),
                ]),
                "Multiple Issues: [Known(\"Test\"), Known(\"Test2\")]",
            ),
        ];

        for test in statuses {
            assert_eq!(format!("{}", test.0), test.1);
        }

        for test in errors {
            assert_eq!(format!("{}", test.0), test.1);
        }
    }
}
