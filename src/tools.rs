use crate::elements::Element;
use crate::validation::Status::Valid;
use crate::validation::{Validation, ValidationResult};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::rc::Weak;

/// Possible Tool Types
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum ToolType {
    Node,
    Mesh,
    SuperNode,
    SuperMesh,
    Thevenin,
    Norton,
    Simplification,
}

/// Tools are used to solve circuits
///
/// Representation of a Tool (Node, Mesh, SuperNode, SuperMesh)
#[derive(Debug)]
pub(crate) struct Tool {
    pub(crate) id: usize,
    pub(crate) class: ToolType,
    pub(crate) elements: Vec<Weak<Element>>,
}

impl Tool {
    /// Create a mesh from the elements
    pub(crate) fn create_mesh(elements: Vec<Weak<Element>>) -> Tool {
        Tool::create(ToolType::Mesh, elements)
    }

    /// Create a node from the elements
    pub(crate) fn create_node(elements: Vec<Weak<Element>>) -> Tool {
        Tool::create(ToolType::Node, elements)
    }

    /// Create a supernode from the elements
    pub(crate) fn create_supernode(elements: Vec<Weak<Element>>) -> Tool {
        Tool::create(ToolType::SuperNode, elements)
    }

    fn create(class: ToolType, elements: Vec<Weak<Element>>) -> Tool {
        let mut tool = Tool {
            id: 0,
            class,
            elements: vec![],
        };
        tool.elements = elements;
        tool
    }

    /// Check if the tool contains an element
    pub(crate) fn contains(&self, element: Weak<Element>) -> bool {
        let element = element.upgrade().unwrap();
        self.elements
            .iter()
            .any(|e| e.upgrade().unwrap().id == element.id)
    }
}

/// Implement PartialEq for Tool
///
/// Compare two Tool by their id
impl PartialEq<Self> for Tool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Validation for Tool {
    fn validate(&self) -> ValidationResult {
        // TODO
        // Check if the elements are valid
        // check id != ZERO
        Ok(Valid)
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Tool: {} Id:{} Elements:{:?}",
            self.class,
            self.id,
            self.elements
                .iter()
                .map(|x| x.upgrade().unwrap().id)
                .collect::<Vec<usize>>()
        )
    }
}

impl Display for ToolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_create() {
        // TODO
    }

    #[test]
    fn test_contains() {
        // TODO
    }

    #[test]
    fn test_validate() {
        // TODO
    }
}
