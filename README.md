# sudoku
A online sudoku pad, sudoku editor and human style sudoku solver.

You can try it [here](https://sudokupad.lihan.fun)

Currently, the solver is implemented in rust and compiled to wasm. The solver is a human style solver, which means it will try to solve the sudoku in the way a human would do. It will not use brute force to solve the sudoku unless it has to.

Implemented solving techniques:

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
    - [ ] XY-Wing
    - [ ] XYZ-Wing
    - [ ] W-Wing

## Bulid

```
cargo install wasm-pack
wasm-pack build sudoku-solver --release
npm run build
```
