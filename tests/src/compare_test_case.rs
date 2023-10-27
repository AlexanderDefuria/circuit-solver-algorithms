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
use circuit_solver_algorithms::validation::StatusError;

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
            let contents = std::fs::read_dir(&path).unwrap();

            let mut input = path.clone();
            let mut output = path.clone();
            for i in contents {
                let osname = i.unwrap().file_name();
                let name = osname.to_str().unwrap();
                if name.contains("input.json") {
                    println!("Found: {}", path.display());
                    input.push("input.json");
                    output.push("output.json");
                } else if name.contains("container") {
                    println!("Found: {}", path.display());
                    input.push(name);
                    output.push("steps_".to_string() + name.split("_").collect::<Vec<_>>()[1]);
                }
            }

            if !input.exists() || !output.exists() {
                println!("Skipping: {}", path.display());
                continue;
            }

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
fn test_cases() {
    let mut failed_cases: Vec<(CasePaths, String)> = Vec::new();

    for case_paths in find_cases() {
        let input_str = if let Ok(input_str) = std::fs::read_to_string(&case_paths.input) {
            input_str
        } else {
            println!("Failed to find and read input file: {}", case_paths.input.display());
            failed_cases.push((case_paths, "Failed to find and read input file".to_string()));
            continue;
        };

        let case: InputCaseSerde = if let Ok(case) = serde_json::from_str(&input_str) {
            // This is the proper and default implementation following the defined conventions.
            case

        } else if case_paths.input.file_name().unwrap().to_str().unwrap().contains("container") {
            // This defaults to NodeStepSolver in the case of this being just a container
            // without extra information. For default cases from the web interface, this is fine.
            InputCaseSerde {
                solver: SolverType::NodeStep,
                container: serde_json::from_str(&input_str).unwrap()
            }

        } else {
            println!("Failed to parse and deserialize input case from file: {}", case_paths.input.display());
            failed_cases.push((case_paths, "Failed to parse and deserialize input case".to_string()));
            continue;
        };


        let output_dir = run_test_case(case.container);
        if output_dir.is_err() {
            println!("Failed to run test case: {}", output_dir.clone().err().unwrap());
            failed_cases.push((case_paths, output_dir.err().unwrap().to_string()));
        } else if let Ok(output_dir) = output_dir {
            assert_json_diff::assert_json_matches_no_panic(
                &std::fs::read_to_string(&case_paths.output).unwrap(),
                &std::fs::read_to_string(&output_dir).unwrap(),
                assert_json_diff::Config::new(assert_json_diff::CompareMode::Strict),
            );
        }
    }

    println!("\nFailed {} cases\n", failed_cases.len());
    for (case, error) in &failed_cases {
        println!("Failed case: {}", case.case_name);
        println!("Error: {}", error);
    }

    if failed_cases.len() > 0 {
        panic!("Failed {} cases\n", failed_cases.len());
    }

}

fn run_test_case(container_input: ContainerSetup) -> Result<PathBuf, StatusError> {
    let mut c: Container = Container::from(container_input);
    c.validate()?;
    c.create_nodes()?;
    c.create_super_nodes();
    let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
    let steps = solver.solve()?;

    let mut output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    output_dir.push("output");
    output_dir.push("test");
    output_dir.set_extension("json");

    if let Err(e) = serde_json::to_writer_pretty(File::create(output_dir.clone()).unwrap(), &steps) {
        println!("Failed to write output file: {}", e);
        return Err(StatusError::Known("Failed to write output file".to_string()));
    }

    Ok(output_dir)
}
