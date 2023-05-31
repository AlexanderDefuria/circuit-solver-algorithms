use crate::container::Container;
use crate::validation::{Validation, ValidationResult};
use std::fmt::{Debug, Formatter};

pub enum Method {
    None,
    Basic,
    Norton,
    Thevinin,
}

pub struct Simplification {
    method: Method,
    value: f64,
    positive: Vec<usize>,
    negative: Vec<usize>,
    // The original has to be preserved?
    // original: Vec<Element>
    // original: PartialContainer // This seems the move...
}

impl Simplification {
    pub fn new(
        method: Method,
        value: f64,
        positive: Vec<usize>,
        negative: Vec<usize>,
    ) -> Simplification {
        Simplification {
            method,
            value,
            positive,
            negative,
        }
    }

    pub fn basic(value: f64, positive: Vec<usize>, negative: Vec<usize>) -> Simplification {
        Simplification::new(Method::Basic, value, positive, negative)
    }

    pub fn norton(value: f64, positive: Vec<usize>, negative: Vec<usize>) -> Simplification {
        Simplification::new(Method::Norton, value, positive, negative)
    }

    pub fn thevinin(value: f64, positive: Vec<usize>, negative: Vec<usize>) -> Simplification {
        Simplification::new(Method::Thevinin, value, positive, negative)
    }

    pub fn simplify(&mut self, _container: &mut Container) -> &mut Self {
        match self.method {
            Method::None => {
                todo!()
            }
            Method::Basic => {
                todo!()
            }
            Method::Norton => {
                todo!()
            }
            Method::Thevinin => {
                todo!()
            }
        }
    }
}

impl Validation for Simplification {
    fn validate(&self) -> ValidationResult {
        todo!()
    }
}

impl PartialEq for Simplification {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl Debug for Simplification {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
