use crate::elements::Element;
use crate::validation::Status::Valid;
use crate::validation::{Status, Validation, ValidationResult};
use serde::{Deserialize, Serialize};
use std::fmt::{write, Display, Formatter};

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
pub(crate) struct Tool<'a> {
    id: usize,
    pseudo_type: ToolType,
    elements: Vec<&'a Element>,
}

impl<'a> Tool<'a> {
    /// Create a new Tool
    ///
    /// It is recommended to use the `circuit::add_tool` function with this function
    pub(crate) fn new(pseudo_type: ToolType) -> Tool<'a> {
        Tool {
            id: 0,
            pseudo_type,
            elements: vec![],
        }
    }

    /// Create a mesh from the elements
    pub(crate) fn create_mesh(elements: Vec<&'a Element>) -> Tool<'a> {
        // TODO
        let mut mesh = Tool::new(ToolType::Mesh);
        mesh.elements = elements;
        mesh
    }

    /// Create a node from the elements
    pub(crate) fn create_node(elements: Vec<&'a Element>) -> Tool<'a> {
        // TODO
        let mut node = Tool::new(ToolType::Node);
        node.elements = elements;
        node
    }
}

/// Implement PartialEq for Tool
///
/// Compare two Tool by their id
impl PartialEq<Self> for Tool<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Validation for Tool<'_> {
    fn validate(&self) -> ValidationResult {
        // TODO
        Ok(Valid)
    }
}

impl Display for Tool<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Tool: {}", self.pseudo_type)
    }
}

impl Display for ToolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
