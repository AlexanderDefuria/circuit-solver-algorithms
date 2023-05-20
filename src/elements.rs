use crate::components::Component;

/// Representation of a Schematic Element
#[derive(Serialize, Deserialize)]
struct Element {
    name: String,
    id: usize,
    value: f64,
    class: Component ,
    positive: Vec<usize>, // Link to other elements
    negative: Vec<usize>,
}

