# sudoku
A online sudoku pad, sudoku editor and human style sudoku solver.

You can try it online [here](https://sudokupad.lihan.fun)

Currently, the solver is implemented in rust and compiled to wasm. The solver is a human style solver, which means it will try to solve the sudoku in the way a human would do. It will not use brute force to solve the sudoku unless it has to.

#### Implemented techniques

- Singles
    - [x] Full House / Lonely Singe: The last empty cell in a house (row, column, or box). A full house is also a naked single and a hidden single.
    - [x] Naked Single: There is only one remaining candidate in a cell.
    - [x] Hidden Single / Pinned Digit: For a specific digit, there is only one remaining cell in a house (row, column, or box).
- Intersections
    - [x] Locked Candidates / Pointing / Claiming: For a specific digit, if all candidates in a house are also located inside (the intersection with) another house, we can eliminate the candidates from the second house outside the intersection.
    - [ ] Almost Locked Candidates: If a digit is confined to two houses, and the digit is confined to the same two cells in both houses, we can eliminate the digit from the rest of the cells in the houses.
- Subsets
    - [x] Naked Subset: If a subset of n cells in a house contains exactly n candidates, we can eliminate these candidates from the rest of the house.
    - [x] Hidden Subset: If a subset of n digits in a house is confined to exactly n cells, we can eliminate other candidates from these cells.
- Fish
    - Basic Fish
        - [x] X-Wing
        - [x] Swordfish
        - [x] Jellyfish
    - [x] Finned Fish
    - [x] Sashimi Fish
    - [x] Franken Fish
    - [x] Mutant Fish
    - [ ] Kraken Fish
- Single Digit Patterns
    - [x] Sky Scrapper
    - [x] 2-String Kite
    - [x] Rectangle Elimination / Empty Rectangle: IMO, they are doing the same elimination. The only difference is the way to find the eliminations.
    - [ ] Turbot Fish
- Wing
    - [x] XY-Wing
    - [x] XYZ-Wing
    - [x] W-Wing
- Chain
    - [x] Forcing Chain

        A forcing chain is a logical sequence where an initial assumption about a cell's candidate value leads to a series of implications based on sudoku rules. For example, "cell r1c1 is 1" implies "cell r1c3 cannot be 1".

        If the chain leads to a contradiction, the original assumption is false.

        Types of Contradictions:
        - [x] The assumption leads to its own negation.
        - [ ] The assumption results in a value that cannot be placed anywhere in a house.
        - [ ] The assumption leads to a cell with no possible candidates.

        In a valid sudoku puzzle, if an assumption consistently holds true across all logical sequences started from the basis, it can be concluded that the assumption is correct.

        Basis:
        - [x] One of the candidates for a cell must be true.
        - [x] For a specific digit, one of the cells in a house (row, column, or block) must contain the digit.

    - [ ] Forcing Net

The techniques are implemented in increasing order of complexity, with the solver attempting the simplest methods first. If a puzzle remains unsolved, more advanced techniques are applied.

The table below provides a detailed analysis of each technique's performance. The metrics used include:

- **Total**: The average step time taken for all steps, regardless of whether they led to progress.
- **Success**: The average step time for steps that successfully advanced the puzzle.
- **Fastest**: The average step time if it is the fastest technique that finds a successful step.

```
       Technique        fastest / success / total count fastest  / success  / total time
      full_house            876 /   2622  /  9824       228.00ns / 392.00ns / 249.00ns
     naked_single          4994 /   5807  /  9824       390.00ns / 389.00ns / 503.00ns
     hidden_single         2229 /   7409  /  9824       724.00ns /   1.06µs /   1.44µs
   locked_candidates       1054 /   6263  /  9824         2.19µs /   3.47µs /   4.76µs
     hidden_subset          170 /   5523  /  9824        11.57µs /   7.13µs /   8.72µs
     naked_subset            30 /   5592  /  9824        11.81µs /  11.75µs /  16.89µs
    two_string_kite           0 /    337  /  9824         0.00ns /   8.25µs /   7.95µs
      skyscraper            515 /   1393  /  9824       835.00ns / 992.00ns / 627.00ns
 rectangle_elimination      277 /   1449  /  9824         1.51µs /   2.10µs /   1.68µs
        w_wing               12 /    153  /  9824         2.28µs /   2.39µs /   1.24µs
        xy_wing              50 /    582  /  9824         2.11µs /   2.23µs /   1.19µs
       xyz_wing             111 /    984  /  9824         2.12µs /   2.12µs /   1.45µs
      basic_fish             18 /   3124  /  9824         7.06µs /  21.92µs /  46.84µs
      finned_fish            93 /   5188  /  9824       245.50µs / 232.46µs / 635.60µs
     franken_fish            33 /   6071  /  9824         4.65ms /   1.64ms /   3.56ms
      mutant_fish             9 /   6094  /  9824       406.28µs /  24.91ms /  69.42ms
     forced_chain           467 /   7601  /  9824       801.59µs / 851.01µs / 698.63µs
         apply                0 /      0  / 66192         0.00ns /   0.00ns /   3.04µs
```

## Bulid

```
cargo install wasm-pack
wasm-pack build sudoku-solver --release
# or `nix develop . --command wasm-pack build sudoku-solver --release` if you are using nix
npm run build
```
