use crate::component::Component::Ground;
use crate::elements::Element;
use crate::tools::ToolType::*;
use crate::util::PrettyPrint;
use crate::validation::Status::Valid;
use crate::validation::StatusError::{Known, Multiple};
use crate::validation::{check_weak_duplicates, StatusError, Validation, ValidationResult};
use operations::prelude::EquationMember;
use petgraph::graph::UnGraph;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::{Rc, Weak};

/// Possible Tool Types
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum ToolType {
    Node,
    Mesh,
    SuperNode,
    SuperMesh,
}

/// Tools are used to solve circuits
///
/// Representation of a Tool (Node, Mesh, SuperNode, SuperMesh)
#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    pub(crate) id: usize,
    pub(crate) class: ToolType,
    pub(crate) members: Vec<Weak<RefCell<Element>>>,
    pub(crate) value: f64,
}

pub struct ToolIterator {
    tool: Tool,
    index: usize,
}

impl IntoIterator for Tool {
    type Item = Rc<RefCell<Element>>;
    type IntoIter = ToolIterator;

    fn into_iter(self) -> ToolIterator {
        ToolIterator {
            tool: self,
            index: 0,
        }
    }
}

impl Iterator for ToolIterator {
    type Item = Rc<RefCell<Element>>;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.tool.members.get(self.index);
        self.index += 1;
        if let Some(y) = x {
            return Some(y.upgrade().unwrap());
        }
        None
    }
}

impl Tool {
    /// Create a mesh from the elements
    pub(crate) fn create_mesh(elements: Vec<Weak<RefCell<Element>>>) -> Tool {
        Tool::create(Mesh, elements)
    }

    /// Create a node from the elements
    pub(crate) fn create_node(elements: Vec<Weak<RefCell<Element>>>) -> Tool {
        Tool::create(Node, elements)
    }

    /// Create a supernode from the elements
    pub(crate) fn create_supernode(elements: Vec<Weak<RefCell<Element>>>) -> Tool {
        Tool::create(SuperNode, elements)
    }

    fn create(class: ToolType, elements: Vec<Weak<RefCell<Element>>>) -> Tool {
        let mut tool = Tool {
            id: 0,
            class,
            members: vec![],
            value: f64::NAN,
        };
        tool.members = elements;
        tool
    }

    /// Check if the tool contains an element
    pub(crate) fn contains(&self, element: Rc<RefCell<Element>>) -> bool {
        self.members
            .iter()
            .any(|e| e.upgrade().unwrap().id() == element.id())
    }

    pub(crate) fn contains_all(&self, elements: &Vec<Weak<RefCell<Element>>>) -> bool {
        self.members
            .iter()
            .filter_map(|x| x.upgrade())
            .all(|tool_element| {
                elements.iter().any(|node_element| {
                    node_element.upgrade().unwrap().borrow().id == tool_element.borrow().id
                })
            })
    }

    fn node_edges(nodes: &Vec<Weak<RefCell<Tool>>>) -> Result<Vec<(u32, u32)>, StatusError> {
        // If no nodes are present, return an error
        if !nodes.iter().any(|p| {
            return if let Some(x) = p.upgrade() {
                x.borrow().class == Node
            } else {
                false
            };
        }) {
            return Err(Known("No nodes present".to_string()));
        }

        let mut edges: Vec<(u32, u32)> = Vec::new();

        // Check each permutation of nodes
        for node in nodes {
            let node = node.upgrade().unwrap();
            if node.borrow().class == Node {
                if node
                    .borrow()
                    .members
                    .iter()
                    .filter_map(|x| x.upgrade())
                    .any(|x| x.borrow().connected_to_ground())
                {
                    let x = (node.borrow().id as u32, 0);
                    if !edges.contains(&x) && (x.0 != x.1) {
                        edges.push(x);
                    }
                }

                for second in nodes {
                    // Check for a connection between the nodes (if they share an element)
                    for element in node.borrow().members.iter().filter_map(|x| x.upgrade()) {
                        if second.upgrade().unwrap().borrow().contains(element) {
                            let x = (
                                node.borrow().id as u32,
                                second.upgrade().unwrap().borrow().id as u32,
                            );
                            let y = (
                                second.upgrade().unwrap().borrow().id as u32,
                                node.borrow().id as u32,
                            );
                            if !edges.contains(&x) && !edges.contains(&y) && x.0 != x.1 {
                                edges.push(x);
                            }
                        }
                    }
                }
            }
        }

        Ok(edges)
    }

    pub fn nodes_to_graph(
        nodes: &Vec<Weak<RefCell<Tool>>>,
    ) -> Result<UnGraph<i32, ()>, StatusError> {
        let edges: Vec<(u32, u32)> = Tool::node_edges(nodes)?;
        Ok(UnGraph::<i32, ()>::from_edges(edges.as_slice()))
    }

    pub fn member_ids(&self) -> Vec<usize> {
        self.members
            .iter()
            .filter_map(|x| x.upgrade())
            .map(|x| x.id())
            .collect()
    }

    pub fn members_weak(&self) -> Vec<Weak<RefCell<Element>>> {
        self.members.clone()
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }
}

/// Implement PartialEq for Tool
///
/// Compare two Tool by their id
impl PartialEq<Self> for Tool {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.class == other.class
    }
}

impl EquationMember for Tool {
    fn equation_repr(&self) -> String {
        format!("T_{{{}}}", self.id)
    }

    fn value(&self) -> f64 {
        self.value
    }

    fn latex_string(&self) -> String {
        match self.class {
            Node => format!("N_{{{}}}", self.id),
            Mesh => format!("M_{{{}}}", self.id),
            SuperNode => format!("SN_{{{}}}", self.id),
            SuperMesh => format!("SM_{{{}}}", self.id),
        }
    }
}

impl Validation for Tool {
    fn validate(&self) -> ValidationResult {
        if self.members.len() == 0 {
            return Err(Known("Tool has no members".to_string()));
        }

        if self.class == Node
            && self
                .members
                .iter()
                .filter_map(|x| x.upgrade())
                .any(|x| x.borrow().class == Ground)
        {
            return Err(Known("Tool contains a ground element".to_string()));
        }

        let mut duplicates = check_weak_duplicates(&self.members);
        if duplicates.len() > 0 {
            duplicates.append(&mut vec![Known(format!(
                "Tool {} has duplicate members",
                self.id
            ))]);
            println!(
                "{:?}",
                &self
                    .members
                    .iter()
                    .filter_map(|x| x.upgrade())
                    .map(|x| x.borrow().id)
                    .collect::<Vec<usize>>()
            );
            println!("{:?}", self);
            return Err(Multiple(duplicates));
        }

        Ok(Valid)
    }

    fn id(&self) -> usize {
        self.id
    }

    fn class(&self) -> String {
        format!("{:?}", self.class)
    }
}

impl Validation for RefCell<Tool> {
    fn validate(&self) -> ValidationResult {
        self.borrow().validate()
    }

    fn id(&self) -> usize {
        self.borrow().id
    }

    fn class(&self) -> String {
        self.borrow().class.to_string()
    }
}

impl Display for Tool {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Tool: {} Id:{} Elements:{:?}",
            self.class,
            self.id,
            self.members
                .iter()
                .filter_map(|x| x.upgrade())
                .map(|x| x.borrow().id)
                .collect::<Vec<usize>>()
        )
    }
}

impl Display for ToolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl PrettyPrint for Tool {
    fn pretty_string(&self) -> String {
        format!("{}: {}", self.class, self.id)
    }

    fn basic_string(&self) -> String {
        format!(
            "{:?}",
            self.members
                .iter()
                .filter_map(|x| x.upgrade())
                .map(|x| x.borrow().id)
                .collect::<Vec<usize>>()
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::container::Container;
    use crate::tools::{Tool, ToolType};
    use crate::util::{create_basic_container, create_basic_supermesh_container};
    use crate::validation::StatusError::Known;
    use crate::validation::Validation;
    use petgraph::graph::UnGraph;
    use std::cell::RefCell;
    use std::rc::Weak;

    #[test]
    fn test_validate() {
        let bad_tool = Tool::create(ToolType::Node, vec![]);
        assert_eq!(
            bad_tool.validate().unwrap_err(),
            Known("Tool has no members".parse().unwrap())
        );
    }

    #[test]
    fn test_create_node_graph() {
        let mut basic: Container = create_basic_container();
        let container: Vec<Weak<RefCell<Tool>>> = basic.create_nodes().nodes();
        let edges = Tool::node_edges(&container).unwrap();
        let expected = vec![(1, 0), (1, 2), (2, 0)];

        assert_eq!(edges.len(), expected.len());
        for edge in edges {
            assert!(expected.contains(&edge));
        }

        let container: Vec<Weak<RefCell<Tool>>> = basic.create_nodes().nodes();
        let graph = Tool::nodes_to_graph(&container).unwrap();
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 3);

        let mut super_node = create_basic_supermesh_container();
        let graph: UnGraph<i32, ()> =
            Tool::nodes_to_graph(&super_node.create_nodes().nodes()).unwrap();
        println!("{:?}", graph);
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 7);
    }
}
