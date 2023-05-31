use crate::container::Container;

enum Method {
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
    // The original has to be preserved.
    // original: Vec<Element>
    // original: PartialContainer
}
