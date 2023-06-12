use crate::components::Component;
use crate::container::Container;
use crate::elements::Element;
use std::rc::{Rc, Weak};

#[derive(Debug, PartialEq)]
pub struct PartialContainer {
    container: Vec<Rc<Element>>,
    positive: usize,
    negative: usize,
}

