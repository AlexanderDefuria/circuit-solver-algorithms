use std::path::{Path, PathBuf};
use serde::Deserialize;
use circuit_solver_algorithms::interfaces::ContainerSetup;
use circuit_solver_algorithms::solvers::solver::{Solver, SolverType};

pub struct CasePaths {
    input: PathBuf,
    output: PathBuf,
}

#[derive(Deserialize)]
pub struct InputCaseSerde {
    solver: SolverType,
    container: ContainerSetup,
}


pub fn find_cases() -> Vec<CasePaths> {
    let mut cases: Vec<CasePaths> = vec![];
    let mut input_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    input_dir.push("tests");
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
        let case: InputCaseSerde = serde_json::from_str(&std::fs::read_to_string(&case_paths.input).unwrap()).unwrap();
    }
}
