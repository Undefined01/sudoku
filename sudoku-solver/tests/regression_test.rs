use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sudoku_solver::{Sudoku, SudokuSolver, solver};

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    initial_values: Option<String>,
    initial_candidates: Option<String>,
    solution: Option<String>,
    steps: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegressionTest {
    techniques: Vec<String>,
    board: Board,
}

fn normalize_value_string(value_string: &str) -> String {
    value_string
        .chars()
        .filter(|c| c.is_digit(10))
        .collect::<String>()
}

fn run_testcase(test_config: RegressionTest) {
    let mut solver: SudokuSolver;
    if let Some(initial_values) = test_config.board.initial_values {
        let sudoku = Sudoku::from_values(initial_values.as_str());
        assert_eq!(sudoku.to_value_string(), initial_values);
        solver = SudokuSolver::new(sudoku);
        solver.initialize_candidates();
        assert_eq!(solver.sudoku().to_value_string(), initial_values);
        if let Some(initial_candidates) = test_config.board.initial_candidates {
            assert_eq!(solver.sudoku().to_candidate_string().trim(), initial_candidates.trim());
        }
    } else {
        assert!(test_config.board.initial_candidates.is_some());
        let sudoku = Sudoku::from_candidates(test_config.board.initial_candidates.as_ref().unwrap().as_str());
        assert_eq!(
            sudoku.to_candidate_string().trim(),
            test_config.board.initial_candidates.as_ref().unwrap().trim()
        );
        solver = SudokuSolver::new(sudoku);
    }

    let mut techniques = Vec::<for<'a> fn(&'a SudokuSolver) -> std::option::Option<sudoku_solver::Step>>::new();
    for technique in test_config.techniques {
        match technique.as_str() {
            "full_house" => techniques.push(SudokuSolver::solve_full_house),
            "naked_single" => techniques.push(SudokuSolver::solve_naked_single),
            "hidden_single" => techniques.push(SudokuSolver::solve_hidden_single),
            "locked_candidates" => techniques.push(SudokuSolver::solve_locked_candidates),
            "hidden_subset" => techniques.push(SudokuSolver::solve_hidden_subset),
            "naked_subset" => techniques.push(SudokuSolver::solve_naked_subset),
            "basic_fish" => techniques.push(solver::fish::solve_basic_fish),
            "finned_fish" => techniques.push(solver::fish::solve_finned_fish),
            "franken_fish" => techniques.push(solver::fish::solve_franken_fish),
            "mutant_fish" => techniques.push(solver::fish::solve_mutant_fish),
            _ => panic!("Unknown technique: {}", technique),
        }
    }

    let mut steps = vec![];
    loop {
        let mut step_found = false;
        for technique in &techniques {
            if let Some(step) = technique(&solver) {
                step_found = true;
                println!("{}", solver.sudoku().to_candidate_string());
                solver.apply_step(&step);
                println!("{}", step.to_string(solver.sudoku()));
                steps.push(step);
                break;
            }
        }
        if !step_found || solver.is_completed() {
            break;
        }
    }

    assert!(solver.get_invalid_positions().is_empty());

    if let Some(solution) = test_config.board.solution {
        assert!(solver.is_completed());
        assert_eq!(solver.sudoku().to_value_string(), normalize_value_string(solution.as_str()));
    }

    if let Some(expected_steps) = test_config.board.steps {
        assert_eq!(steps.iter().map(|s| s.to_string(solver.sudoku()).trim().to_string()).join("\n"), expected_steps.trim());
    }
}

#[test]
fn regression_test() {
    let test_dir = "tests/regression_tests";
    let groups = std::fs::read_dir(test_dir).unwrap();
    for group in groups {
        let group = group.unwrap();
        let group_path = group.path();
        if group_path.is_dir() {
            let tests = std::fs::read_dir(group_path).unwrap();
            for test_path in tests {
                if let Ok(test_path) = test_path {
                    let test_config: RegressionTest =
                        toml::from_str(std::fs::read_to_string(test_path.path()).unwrap().as_str())
                            .unwrap();
                    println!("Testing {}", test_path.path().to_str().unwrap());
                    run_testcase(test_config);
                }
            }
        }
    }
}
