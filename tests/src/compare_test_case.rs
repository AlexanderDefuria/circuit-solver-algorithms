use std::cell::RefCell;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use serde::Deserialize;
use circuit_solver_algorithms::container::Container;
use circuit_solver_algorithms::interfaces::ContainerSetup;
use circuit_solver_algorithms::solvers::node_step_solver::NodeStepSolver;
use circuit_solver_algorithms::solvers::solver::{Solver, SolverType};
use circuit_solver_algorithms::validation::Validation;

pub struct CasePaths {
    case_name: String,
    input: PathBuf,
    output: PathBuf,
}

#[derive(Deserialize)]
pub struct InputCaseSerde {
    pub solver: SolverType,
    pub container: ContainerSetup,
}


pub fn find_cases() -> Vec<CasePaths> {
    let mut cases: Vec<CasePaths> = vec![];
    let mut input_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    input_dir.push("data");

    for entry in input_dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.display().to_string().contains("!") {
            println!("Skipping: {}", path.display());
            continue;
        }
        if path.is_dir() {
            println!("Found: {}", path.display());
            let mut input = path.clone();
            input.push("input.json");
            let mut output = path.clone();
            output.push("output.json");
            cases.push(CasePaths {
                case_name: path.file_name().unwrap().to_str().unwrap().to_string(),
                input,
                output,
            });
        }
    }
    cases
}

fn load_input_case(paths: CasePaths) -> InputCaseSerde {
    let raw_json: &str = &std::fs::read_to_string(&paths.input).unwrap();
    let case: InputCaseSerde = serde_json::from_str(raw_json).unwrap();
    case
}


#[test]
fn test_case_discovery() {
    for case_paths in find_cases() {
        let input_str = if let Ok(input_str) = std::fs::read_to_string(&case_paths.input) {
            input_str
        } else {
            println!("Failed to find and read input file: {}", case_paths.input.display());
            continue;
        };

        let case: InputCaseSerde = if let Ok(case) = serde_json::from_str(&input_str) {
            case
        } else {
            println!("Failed to parse and deserialize case from file: {}", case_paths.input.display());
            continue;
        };

        let mut c: Container = Container::from(case.container);
        c.validate().unwrap();
        c.create_nodes().unwrap();
        c.create_super_nodes();
        let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
        let steps = solver.solve().unwrap();

        let mut output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        output_dir.push("output");
        output_dir.push(format!("{}", case_paths.case_name));
        output_dir.set_extension("json");

        serde_json::to_writer_pretty(File::create(output_dir.clone()).unwrap(), &steps).unwrap();

        assert_json_diff::assert_json_eq!(
            std::fs::read_to_string(&case_paths.output).unwrap(),
            std::fs::read_to_string(&output_dir).unwrap()
        );
    }
}
