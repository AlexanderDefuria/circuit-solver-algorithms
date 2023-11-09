use std::cell::RefCell;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use serde::Deserialize;
use circuit_solver_algorithms::container::Container;
use circuit_solver_algorithms::interfaces::ContainerSetup;
use circuit_solver_algorithms::solvers::node_step_solver::NodeStepSolver;
use circuit_solver_algorithms::solvers::solver::{Step, Solver, SolverType};
use circuit_solver_algorithms::validation::Validation;
use circuit_solver_algorithms::validation::StatusError;

/// Data provided by the user to run a test case
/// Error is optional and is used to test the expected error handling of the container
#[derive(Debug, Clone)]
pub struct CasePaths {
    case_name: String,
    input: PathBuf,
    output: PathBuf,
    error: Option<PathBuf>,
}

#[derive(Deserialize)]
pub struct InputCaseSerde {
    pub solver: SolverType,
    pub container: ContainerSetup,
    pub error: Option<String>,
}


#[test]
fn test_cases() {
    let mut failed_cases: Vec<(CasePaths, String)> = Vec::new();

    for case_paths in find_cases() {
        // Steup The Test Case
        println!("Running: {}", case_paths.case_name.clone());
        let case: InputCaseSerde = match setup_test_case(case_paths.clone()) {
            Ok(case) => case,
            Err(e) => {
                failed_cases.push((case_paths, e));
                continue;
            }
        };

        // Run The Test Case
        let output_dir = run_test_case(case.container, case_paths.case_name.clone());

        let result: Result<(), String> = if let Some(e) = &case_paths.error {
            assert_json_diff::assert_json_matches_no_panic(
                &std::fs::read_to_string(e).unwrap(),
                &std::fs::read_to_string(&output_dir.clone().unwrap()).unwrap(),
                assert_json_diff::Config::new(assert_json_diff::CompareMode::Strict),
            )
        } else {
            // Compare The Good Test Case
             assert_json_diff::assert_json_matches_no_panic(
                &std::fs::read_to_string(&case_paths.output).unwrap(),
                &std::fs::read_to_string(&output_dir.clone().unwrap()).unwrap(),
                assert_json_diff::Config::new(assert_json_diff::CompareMode::Strict),
            )
        };

        // Handle The Results
        if result.is_err() {
            println!("Failed compare test case: {}", case_paths.case_name.clone());
            failed_cases.push((case_paths, result.err().unwrap().to_string()));
            continue;
        } else {
            // println!("Passed");
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
            let mut error: Option<PathBuf> = None;
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

                // Error is Optional
                if name.contains("error.json") {
                    error = Some(path.clone().join(name));
                } else if name.contains("error") {
                    error = Some(path.clone().join(name));
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
                error,
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

fn setup_test_case(case_paths: CasePaths) -> Result<InputCaseSerde, String> {
    let input_string: String = read_input_file(&case_paths.input)?;
    let error_string: Option<String> = read_error_file(&case_paths.error);

    let mut category = case_paths.input.file_name().unwrap().to_str().unwrap();
    category = category.split(".").collect::<Vec<_>>()[0];
    category = category.split("_").collect::<Vec<_>>()[0];

    return match category {
        "input" => serde_json::from_str(&input_string).map_err(|e| e.to_string()),
        "container" => {
            let container:ContainerSetup = if let Ok(case) = serde_json::from_str(&input_string) {
                case
            } else {
                println!("Failed to parse and deserialize input case from file: {}", case_paths.input.display());
                return Err("Failed to parse and deserialize input case".to_string());
            };

            Ok(InputCaseSerde {
                solver: SolverType::NodeStep,
                container: serde_json::from_str(&input_string).unwrap(),
                error: None,
            })
        },
        _ => {
            println!("Failed to parse and deserialize input case from file: {}", case_paths.input.display());
            return Err("Failed to parse and deserialize input case".to_string());
        }
    };
}

fn read_input_file<P: AsRef<Path>>(path: P) -> Result<String, String> {
    return if let Ok(result) = std::fs::read_to_string(path) {
        Ok(result)
    } else {
        Err("Failed to read input file".to_string())
    }
}

fn read_error_file<P: AsRef<Path>>(path: &Option<P>) -> Option<String> {
    let path = if let Some(path) = path {
        path
    } else {
        return None;
    };

    return if let Ok(result) = std::fs::read_to_string(path) {
        Some(result)
    } else {
        None
    }
}

fn run_test_case(container_input: ContainerSetup, name: String) -> Result<PathBuf, StatusError> {
    let get_steps_and_errors = || -> Result<Vec<Step>, StatusError> {
        let mut c: Container = Container::from(container_input);
        c.validate()?;
        c.create_nodes()?;
        c.create_super_nodes()?;
        let mut solver: NodeStepSolver = Solver::new(Rc::new(RefCell::new(c)));
        let steps = solver.solve()?;
        Ok(steps)
    };

    let steps = get_steps_and_errors();

    let mut output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    output_dir.push("output");
    output_dir.push(name);
    output_dir.set_extension("json");

    if let Ok(steps) = steps {
        if let Err(e) = serde_json::to_writer_pretty(File::create(output_dir.clone()).unwrap(), &steps) {
            println!("Failed to write output file: {}", e);
            return Err(StatusError::Known("Failed to write output file".to_string()));
        }
    } else {
        let error: StatusError = steps.err().unwrap();
        let json_error = String::from(error.clone());
        if let Err(e) = serde_json::to_writer_pretty(File::create(output_dir.clone()).unwrap(), &json_error) {
            println!("Failed to write output file: {}", e);
            return Err(StatusError::Known("Failed to write output file".to_string()));
        }
    }

    Ok(output_dir)
}
