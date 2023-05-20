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
    Simplification
}



/// Tools are used to solve circuits
///
/// Representation of a Tool (Node, Mesh, SuperNode, SuperMesh)
struct Tool<'a> {
    id: usize,
    pseudo_type: ToolType,
    elements: Vec<&'a Element>
}