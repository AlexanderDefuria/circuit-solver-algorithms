use crate::container::Container;
use crate::solvers::solver::{Solver, Step};
use std::cell::RefCell;
use std::rc::Rc;

// TODO MeshMatrixSolver
#[allow(dead_code)]
pub struct MeshMatrixSolver {
    container: Rc<RefCell<Container>>,
}

impl Solver for MeshMatrixSolver {
    fn new(container: Rc<RefCell<Container>>) -> Self {
        MeshMatrixSolver { container }
    }

    fn solve(&mut self) -> Result<Vec<Step>, String> {
        todo!()
    }
}

// TODO: Mesh Tests
// #[cfg(test)]
// mod test {
//     use crate::container::Container;
//     use crate::elements::Element;
//     use crate::tools::Tool;
//     use crate::tools::ToolType::{Mesh, SuperMesh};
//     use crate::util::create_basic_supermesh_container;
//     use std::rc::{Rc, Weak};
//
//     #[test]
//     fn test_get_meshes() {
//         let mut container: Container = create_basic_supermesh_container();
//         container.create_nodes();
//         container.create_super_nodes();
//         container.create_meshes();
//         container.create_super_meshes();
//         let meshes: Vec<Weak<Tool>> = container.get_tools_by_type(Mesh);
//         assert_eq!(meshes.len(), 3);
//
//         let mesh_members: Vec<Vec<usize>> = meshes
//             .iter()
//             .map(|x| x.upgrade().unwrap().members())
//             .collect();
//         let expected_members: Vec<Vec<usize>> = vec![vec![1, 2, 5], vec![2, 3, 6], vec![3, 4, 7]];
//
//         println!(
//             "{:?}",
//             meshes
//                 .iter()
//                 .map(|x| x
//                     .upgrade()
//                     .unwrap()
//                     .members
//                     .iter()
//                     .map(|y| y.upgrade().unwrap())
//                     .collect::<Vec<Rc<Element>>>())
//                 .collect::<Vec<Vec<Rc<Element>>>>()
//         );
//
//         for i in 0..meshes.len() {
//             println!("{:?}", mesh_members[i]);
//             // assert_eq!(mesh_members[i], expected_members[i]);
//         }
//         todo!()
//     }
// }
