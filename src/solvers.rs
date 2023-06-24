use crate::container::Container;
use std::rc::Rc;

/// This will take a container and solve it using the given method.
/// KCL and KVL will be used to solve the circuit.

struct Solver {
    container: Rc<Container>,
}
