use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

/// Possible Ok Statuses
///
/// Valid: Container is valid
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Status {
    Valid,
    Simplified,
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
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            Status::Valid => write!(f, "Valid"),
            Status::Simplified => write!(f, "Simplified"),
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

impl Error for StatusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            _ => None,
        }
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

/// Check for duplicates in a list
///
/// Returns a Vec of StatusError::KnownIssue. If the vec is empty, there are no duplicates.
pub(crate) fn check_duplicates<T: Validation + PartialEq + Display>(
    list: &Vec<Rc<T>>,
) -> Vec<StatusError> {
    let mut errors: Vec<StatusError> = Vec::new();
    let mut seen: Vec<&Rc<T>> = Vec::new();
    for x in list {
        if seen.contains(&x) {
            errors.push(StatusError::Known(format!("Duplicate: {}", x)));
        }
        seen.push(x);
    }
    errors
}
