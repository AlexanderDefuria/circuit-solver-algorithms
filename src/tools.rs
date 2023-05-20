use crate::container::Validation;
use crate::elements::Element;
use serde::{Deserialize, Serialize};
use serde_json::Result;

/// Possible Tool Types
#[derive(Serialize, Deserialize)]
enum ToolType {
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

/// Implement PartialEq for Tool
///
/// Compare two Tool by their id
impl PartialEq<Self> for Tool<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Validation for Tool<'_> {
    fn validate(&self) -> bool {
        // TODO
        true
    }
}
