use std::{collections::HashMap, time::Duration};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sudoku_solver::{solver, Sudoku, SudokuSolver};

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

fn load_sudoku(test_config: &RegressionTest) -> SudokuSolver {
    let mut solver: SudokuSolver;
    if let Some(initial_values) = &test_config.board.initial_values {
        let sudoku = Sudoku::from_values(initial_values.as_str());
        assert_eq!(&sudoku.to_value_string(), initial_values);
        solver = SudokuSolver::new(sudoku);
        solver.initialize_candidates();
        assert_eq!(&solver.sudoku().to_value_string(), initial_values);
        if let Some(initial_candidates) = &test_config.board.initial_candidates {
            assert_eq!(
                solver.sudoku().to_candidate_string().trim(),
                initial_candidates.trim()
            );
        }
    } else {
        assert!(test_config.board.initial_candidates.is_some());
        let sudoku = Sudoku::from_candidates(
            test_config
                .board
                .initial_candidates
                .as_ref()
                .unwrap()
                .as_str(),
        );
        assert_eq!(
            sudoku.to_candidate_string().trim(),
            test_config
                .board
                .initial_candidates
                .as_ref()
                .unwrap()
                .trim()
        );
        solver = SudokuSolver::new(sudoku);
    }
    solver
}

fn load_techinques(
    techniques: &Vec<String>,
) -> Vec<fn(&SudokuSolver) -> std::option::Option<sudoku_solver::Step>> {
    let mut technique_fns =
        Vec::<fn(&SudokuSolver) -> std::option::Option<sudoku_solver::Step>>::new();
    for technique in techniques {
        match technique.as_str() {
            "full_house" => technique_fns.push(SudokuSolver::solve_full_house),
            "naked_single" => technique_fns.push(SudokuSolver::solve_naked_single),
            "hidden_single" => technique_fns.push(SudokuSolver::solve_hidden_single),
            "locked_candidates" => technique_fns.push(SudokuSolver::solve_locked_candidates),
            "hidden_subset" => technique_fns.push(SudokuSolver::solve_hidden_subset),
            "naked_subset" => technique_fns.push(SudokuSolver::solve_naked_subset),
            "basic_fish" => technique_fns.push(solver::fish::solve_basic_fish),
            "finned_fish" => technique_fns.push(solver::fish::solve_finned_fish),
            "franken_fish" => technique_fns.push(solver::fish::solve_franken_fish),
            "mutant_fish" => technique_fns.push(solver::fish::solve_mutant_fish),
            "two_string_kite" => {
                technique_fns.push(solver::single_digit_patterns::solve_two_string_kite)
            }
            "skyscraper" => technique_fns.push(solver::single_digit_patterns::solve_skyscraper),
            "rectangle_elimination" => {
                technique_fns.push(solver::single_digit_patterns::solve_rectangle_elimination)
            }
            _ => panic!("Unknown technique: {}", technique),
        }
    }
    technique_fns
}

fn run_testcase(test_config: RegressionTest) {
    let mut solver = load_sudoku(&test_config);
    let techniques = load_techinques(&test_config.techniques);

    let mut steps = vec![];
    loop {
        let mut step_found = false;
        for technique in &techniques {
            if let Some(step) = technique(&solver) {
                step_found = true;
                solver.apply_step(&step);
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
        assert_eq!(
            solver.sudoku().to_value_string(),
            normalize_value_string(solution.as_str())
        );
    }

    if let Some(expected_steps) = test_config.board.steps {
        assert_eq!(
            steps
                .iter()
                .map(|s| s.to_string(solver.sudoku()).trim().to_string())
                .join("\n"),
            expected_steps.trim()
        );
    }
}

struct Statistic {
    total_count: usize,
    success_count: usize,
    fastest_count: usize,

    total_time: std::time::Duration,
    success_time: std::time::Duration,
    fastest_time: std::time::Duration,
}

fn analyze_testcase(test_config: RegressionTest, statistics: &mut HashMap<String, Statistic>) {
    let mut solver = load_sudoku(&test_config);
    let techniques = test_config
        .techniques
        .iter()
        .zip(load_techinques(&test_config.techniques).iter())
        .map(|(name, f)| (name.clone(), f.clone()))
        .collect::<Vec<_>>();

    let mut steps = vec![];
    loop {
        let mut step_found = false;
        let mut new_steps = vec![];
        for (name, technique) in &techniques {
            let start_time = std::time::Instant::now();
            let step = technique(&solver);
            let elapsed_time = start_time.elapsed();

            let statistic = statistics.entry(name.clone()).or_insert(Statistic {
                total_count: 0,
                success_count: 0,
                fastest_count: 0,
                total_time: std::time::Duration::new(0, 0),
                success_time: std::time::Duration::new(0, 0),
                fastest_time: std::time::Duration::new(0, 0),
            });
            statistic.total_count += 1;
            statistic.total_time += elapsed_time;

            if let Some(step) = step {
                statistic.success_count += 1;
                statistic.success_time += elapsed_time;

                step_found = true;
                steps.push(step.clone());
                new_steps.push((name, elapsed_time, step));
            }
        }
        if !step_found {
            break;
        }

        let fastest_time = new_steps.iter().map(|(_, time, _)| time).min().unwrap();
        let fastest_threshold = fastest_time.mul_f64(1.1);
        for (name, time, step) in new_steps {
            solver.apply_step(&step);
            if time <= fastest_threshold {
                statistics.get_mut(name).unwrap().fastest_count += 1;
                statistics.get_mut(name).unwrap().fastest_time += time;
            }
        }

        if solver.is_completed() {
            break;
        }
    }
}

fn generate_testcase(filename: String, mut test_config: RegressionTest) {
    let mut solver = load_sudoku(&test_config);
    let techniques = load_techinques(&test_config.techniques);

    if test_config.board.initial_candidates.is_none() {
        test_config.board.initial_candidates = Some(solver.sudoku().to_candidate_string());
    }

    let mut steps = vec![];
    loop {
        let mut step_found = false;
        for technique in &techniques {
            if let Some(step) = technique(&solver) {
                step_found = true;
                solver.apply_step(&step);
                steps.push(step);
                break;
            }
        }
        if !step_found || solver.is_completed() {
            break;
        }
    }

    test_config.board.steps = Some(
        steps
            .iter()
            .map(|s| s.to_string(solver.sudoku()).trim().to_string())
            .join("\n"),
    );

    if solver.is_completed() {
        test_config.board.solution = Some(solver.sudoku().to_value_string());
    }

    let parent_folder = std::path::Path::new(&filename).parent().unwrap();
    if !parent_folder.exists() {
        std::fs::create_dir_all(parent_folder).unwrap();
    }
    std::fs::write(filename, toml::to_string(&test_config).unwrap()).unwrap();
}

#[test]
fn regression_test() {
    let test_dir = "tests/regression_tests";
    let groups = std::fs::read_dir(test_dir).unwrap();
    for group in groups {
        let group = group.unwrap();
        let group_path = group.path();
        if !group_path.is_dir()
            || group_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(".")
        {
            continue;
        }
        let tests = std::fs::read_dir(group_path).unwrap();
        for test_path in tests {
            if let Ok(test_path) = test_path {
                if test_path
                    .path()
                    .extension()
                    .map_or(true, |ext| ext != "toml")
                    || test_path.file_name().to_str().unwrap().starts_with(".")
                {
                    continue;
                }
                let test_config: RegressionTest =
                    toml::from_str(std::fs::read_to_string(test_path.path()).unwrap().as_str())
                        .unwrap();
                println!("Testing {}", test_path.path().to_str().unwrap());
                run_testcase(test_config);
            }
        }
    }
}

#[test]
#[ignore]
fn generate_regression() {
    let sudokus = std::fs::read_to_string("tests/sudokus.txt").unwrap();
    for (idx, sudoku) in sudokus.trim().lines().enumerate() {
        println!("Analyzing {}", idx + 1);
        let test_config = RegressionTest {
            techniques: vec![
                "full_house".to_string(),
                "naked_single".to_string(),
                "hidden_single".to_string(),
                "locked_candidates".to_string(),
                "hidden_subset".to_string(),
                "naked_subset".to_string(),
                "basic_fish".to_string(),
                "finned_fish".to_string(),
                "franken_fish".to_string(),
                "mutant_fish".to_string(),
            ],
            board: Board {
                initial_values: Some(sudoku.to_string()),
                initial_candidates: None,
                solution: None,
                steps: None,
            },
        };
        generate_testcase(format!("collection/{}.toml", idx + 1), test_config);
    }
}

#[test]
#[ignore]
fn analyze_techniques() {
    let mut statictics = HashMap::<String, Statistic>::new();

    let sudokus = std::fs::read_to_string("tests/sudokus.txt").unwrap();
    for (idx, sudoku) in sudokus.trim().lines().enumerate() {
        println!("Analyzing {}", idx + 1);
        let test_config = RegressionTest {
            techniques: vec![
                "naked_single".to_string(),
                "hidden_single".to_string(),
                "locked_candidates".to_string(),
                "hidden_subset".to_string(),
                "naked_subset".to_string(),
                "basic_fish".to_string(),
                "finned_fish".to_string(),
                "franken_fish".to_string(),
                "two_string_kite".to_string(),
                "skyscraper".to_string(),
                "rectangle_elimination".to_string(),
            ],
            board: Board {
                initial_values: Some(sudoku.to_string()),
                initial_candidates: None,
                solution: None,
                steps: None,
            },
        };
        analyze_testcase(test_config, &mut statictics);
    }

    for (name, statistic) in statictics {
        let avg_fastest_time = statistic
            .fastest_time
            .checked_div(statistic.fastest_count as u32)
            .unwrap_or(Duration::new(0, 0));
        let avg_success_time = statistic
            .success_time
            .checked_div(statistic.success_count as u32)
            .unwrap_or(Duration::new(0, 0));
        let avg_total_time = statistic
            .total_time
            .checked_div(statistic.total_count as u32)
            .unwrap_or(Duration::new(0, 0));
        println!(
            "{}:\t{}/{}/{}\t{:.2?}/{:.2?}/{:.2?}",
            name,
            statistic.fastest_count,
            statistic.success_count,
            statistic.total_count,
            avg_fastest_time,
            avg_success_time,
            avg_total_time,
        );
    }
}
