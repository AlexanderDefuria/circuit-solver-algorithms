use serde::{Deserialize, Serialize};
use serde_json::Result;

/// Current State of an Element
enum State {
    Solved,
    Unknown,
    Partial,
}


/// Representation of a Schematic Container
///
/// Container is a collection of Elements and Tools we are using to solve the circuit
struct Container<'a> {
    elements: Vec<Element>,
    tools: Vec<Tool<'a>>,
    ground: usize,
    state: State,
}

