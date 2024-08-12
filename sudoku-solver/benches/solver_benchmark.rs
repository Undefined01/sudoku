use criterion::{black_box, criterion_group, criterion_main, Criterion};

use sudoku_solver::{Sudoku, SudokuSolver};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sudoku hard", |b| {
        b.iter(|| {
            let sudoku = Sudoku::from_values(black_box(
                "9.7..5...1..7..9..86..9.57..8...61.9316.59..72.91..65.....2..96.9...4..8...9..3.5",
            ));
            let mut solver = SudokuSolver::new(sudoku);
            solver.initialize_candidates();
            while let Some(step) = solver.solve_one_step() {
                solver.apply_step(&step);
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
