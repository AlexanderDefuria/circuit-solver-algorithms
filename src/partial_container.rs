use crate::elements::Element;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct PartialContainer {
    container: Vec<Rc<Element>>,
    positive: usize,
    negative: usize,
}

