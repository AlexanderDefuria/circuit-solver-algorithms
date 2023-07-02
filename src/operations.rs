use std::ops::Deref;
use std::rc::{Rc, Weak};
use serde::{Deserialize, Serialize};
use crate::component::Simplification;
use crate::container::Container;
use crate::tools::ToolType;
use crate::validation::{StatusError, Validation, ValidationResult};

pub struct Operation {
    pub origin: Weak<Container>,
    pub method: OpMethod,
    pub result: Option<Rc<Container>>,
}

#[derive(Serialize, Deserialize)]
pub enum OpMethod {
    Simplify(Simplification),
    Tool(ToolType),
    Validation,
}

impl Operation {
    pub fn new(origin: Weak<Container>, method: OpMethod) -> Operation {
        Operation {
            origin,
            method,
            result: None,
        }
    }

    pub fn completed(&self) -> bool {
        self.result.is_some()
    }

    pub fn run(&mut self) -> Result<&mut Self, StatusError> {
        let mut result: Container = Container::new();
        match &self.method {
            OpMethod::Simplify(method) => {
                result = self.origin.upgrade().unwrap().deref().clone();
            }
            OpMethod::Tool(_) => {

            }
            OpMethod::Validation => {

            }
        }
        result.validate()?;
        self.result = Some(Rc::new(result));
        Ok(self)
    }

    pub fn has_origin(&self) -> bool {
        self.origin.upgrade().is_some()
    }
}

impl From<OpMethod> for Operation {
    fn from(method: OpMethod) -> Self {
        Operation::new(Weak::new(), method)
    }
}

impl Validation for Operation {
    fn validate(&self) -> ValidationResult {
        todo!()
    }
}