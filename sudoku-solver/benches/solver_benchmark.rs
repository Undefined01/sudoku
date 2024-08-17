use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sudoku_solver::{solver::Techniques, Sudoku, SudokuSolver};

pub fn combination_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("combinations");

    group.bench_function("itertools", |b| {
        let n = black_box(9);
        let k = black_box(4);
        let arr = (0..n).collect::<Vec<_>>();
        b.iter(|| {
            use itertools::Itertools;
            for result in arr.iter().copied().combinations(k) {
                black_box(result);
            }
        })
    });

    group.bench_function("utils", |b| {
        let n = black_box(9);
        let k = black_box(4);
        let arr = (0..n).collect::<Vec<_>>();
        b.iter(|| {
            let config = sudoku_solver::utils::CombinationOptions::default();
            for result in sudoku_solver::utils::combinations(&arr, k, config) {
                black_box(result);
            }
        })
    });

    group.bench_function("utils::cached", |b| {
        let n = black_box(9);
        let k = black_box(4);
        let arr = (0..n).collect::<Vec<_>>();
        b.iter(|| {
            for result in sudoku_solver::utils::comb(&arr, k) {
                black_box(result);
            }
        })
    });
}

pub fn solver_benchmark(c: &mut Criterion) {
    c.bench_function("sudoku hard", |b| {
        let techniques = Techniques::new();
        b.iter(|| {
            let sudoku = Sudoku::from_values(black_box(
                "9.7..5...1..7..9..86..9.57..8...61.9316.59..72.91..65.....2..96.9...4..8...9..3.5",
            ));
            let mut solver = SudokuSolver::new(sudoku);
            solver.initialize_candidates();
            while let Some(step) = solver.solve_one_step(&techniques) {
                solver.apply_step(&step);
            }
        })
    });
}

criterion_group!(benches, combination_benchmark, solver_benchmark);
criterion_main!(benches);
