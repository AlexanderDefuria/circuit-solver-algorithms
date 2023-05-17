
/// Possible Component Types
enum Component {
    Ground,
    Voltage,
    Current,
    Resistor,
    // Inductor,
    // Capacitor,
}

/// Current State of an Element
enum State {
    Solved,
    Unknown,
    Partial,
}

/// Representation of a Schematic Element
struct Element {
    name: str,
    id: usize,
    value: f64,
    class: Component,
    positive: Vec<usize>, // Link to other elements
    negative: Vec<usize>,
}

/// Node Voltage used with KCL
struct Node<'a> {
    id: usize,
    elements: Vec<&'a Element>
}

/// Mesh used with KVL
struct Mesh<'a> {

}


#[cfg(test)]
mod tests {
    use super::*;


}
