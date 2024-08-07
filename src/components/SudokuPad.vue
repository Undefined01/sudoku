<script setup lang="ts">
import {
  mdiAutoFix,
  mdiBackspaceOutline,
  mdiPalette,
  mdiRedo,
  mdiStrategy,
  mdiUndo,
} from "@mdi/js";

import { inject, ref } from "vue";
import { Sudoku } from "@/models/sudoku";
import { Sudoku as RustSudoku, SudokuSolver } from "sudoku-solver";

const sudoku = inject<Sudoku>("sudoku")!;

const deleteSelected = () => {
  sudoku.updateState(true, (state) => {
    state.selectedCells.values().forEach((cell) => {
      if (mode.value === "value") {
        state.getCell(cell).setValue(undefined);
      }
      if (mode.value === "candidate") {
        state.getCell(cell).clearCandidates();
      }
      if (mode.value === "pencilMark") {
        state.getCell(cell).clearPencilMarks();
      }
    });
  });
};

const toggleValue = (value: number) => {
  sudoku.updateState(true, (state) => {
    state.selectedCells.values().forEach((cell) => {
      if (mode.value === "value") {
        state.getCell(cell).setValue(value);
      }
      if (mode.value === "candidate") {
        state.getCell(cell).toggleCandidate(value);
      }
      if (mode.value === "pencilMark") {
        state.getCell(cell).togglePencilMark(value);
      }
    });
  });
};

type PadMode = "value" | "candidate" | "pencilMark" | "color";
const mode = ref<PadMode>("value");

let rustSudoku = RustSudoku.from_str(sudoku.toValueString());
let solver = SudokuSolver.new(rustSudoku);
const fillPencilMarks = () => {
  rustSudoku = RustSudoku.from_str(sudoku.toValueString());
  solver = SudokuSolver.new(rustSudoku);
  solver.initialize_candidates(rustSudoku);
  const candidateStr = rustSudoku.to_candidate_string();
  sudoku.updateState(true, (state) => {
    state.fromCandidateString(candidateStr);
  });
};
const solveOneStep = () => {
  console.time("wasm solveOneStep");
  const step = solver.solve_one_step(rustSudoku);
  console.timeEnd("wasm solveOneStep");
  if (step !== undefined) {
    console.log(step.to_string(rustSudoku));
    solver.apply_step(rustSudoku, step);
    const candidateStr = rustSudoku.to_candidate_string();
    sudoku.updateState(true, (state) => {
      state.fromCandidateString(candidateStr);
    });
  } else {
    console.log("No avaliable step");
  }
};
</script>

<template>
  <div class="sudokupad-container">
    <v-btn class="pad-button" @click="() => fillPencilMarks()">
      <v-icon :icon="mdiAutoFix" />
      <v-tooltip activator="parent" location="bottom"
        >自动填充 pencil marks</v-tooltip
      >
    </v-btn>
    <v-btn class="pad-button" @click="() => solveOneStep()">
      <v-icon :icon="mdiStrategy" />
      <v-tooltip activator="parent" location="bottom">求解下一步</v-tooltip>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(1)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          1
        </text>
      </svg>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(1)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          1
        </text>
      </svg>
    </v-btn>

    <v-btn class="pad-button" @click="() => toggleValue(1)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          1
        </text>
      </svg>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(2)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          2
        </text>
      </svg>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(3)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          3
        </text>
      </svg>
    </v-btn>

    <v-btn
      class="pad-button"
      :active="mode === 'value'"
      @click="() => (mode = 'value')"
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <rect
          x="2"
          y="2"
          width="20"
          height="20"
          rx="4"
          ry="4"
          fill="transparent"
          stroke="black"
          stroke-width="1.2"
        />
        <text
          font-size="12"
          x="12.6"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
          font-weight="550"
        >
          1
        </text>
      </svg>
    </v-btn>

    <v-btn class="pad-button" @click="() => toggleValue(4)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          4
        </text>
      </svg>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(5)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          5
        </text>
      </svg>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(6)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          6
        </text>
      </svg>
    </v-btn>

    <v-btn
      class="pad-button"
      :active="mode === 'pencilMark'"
      @click="() => (mode = 'pencilMark')"
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <rect
          x="2"
          y="2"
          width="20"
          height="20"
          rx="4"
          ry="4"
          fill="transparent"
          stroke="black"
          stroke-width="1.2"
        />
        <text
          font-size="6"
          x="7"
          y="8"
          text-anchor="middle"
          dominant-baseline="central"
          font-weight="bold"
        >
          1
        </text>
        <text
          font-size="6"
          x="12"
          y="8"
          text-anchor="middle"
          dominant-baseline="central"
          font-weight="bold"
        >
          2
        </text>
      </svg>
    </v-btn>

    <v-btn class="pad-button" @click="() => toggleValue(7)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          7
        </text>
      </svg>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(8)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          8
        </text>
      </svg>
    </v-btn>
    <v-btn class="pad-button" @click="() => toggleValue(9)">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <text
          font-size="16"
          x="12"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
        >
          9
        </text>
      </svg>
    </v-btn>

    <v-btn
      class="pad-button"
      :active="mode === 'candidate'"
      @click="() => (mode = 'candidate')"
    >
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <rect
          x="2"
          y="2"
          width="20"
          height="20"
          rx="4"
          ry="4"
          fill="transparent"
          stroke="black"
          stroke-width="1.2"
        />
        <text
          font-size="8"
          x="10"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
          font-weight="bold"
        >
          1
        </text>
        <text
          font-size="8"
          x="14"
          y="12"
          text-anchor="middle"
          dominant-baseline="central"
          font-weight="bold"
        >
          2
        </text>
      </svg>
    </v-btn>

    <v-btn class="pad-button" @click="() => sudoku.undo()">
      <v-icon :icon="mdiUndo" />
    </v-btn>

    <v-btn class="pad-button" @click="() => sudoku.redo()">
      <v-icon :icon="mdiRedo" />
    </v-btn>

    <v-btn class="pad-button" @click="() => deleteSelected()">
      <v-icon :icon="mdiBackspaceOutline" />
    </v-btn>

    <v-btn
      class="pad-button"
      :active="mode === 'color'"
      @click="() => (mode = 'color')"
    >
      <div class="stack">
        <div>
          <v-icon :icon="mdiPalette" />
        </div>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
          <rect
            x="2"
            y="2"
            width="20"
            height="20"
            rx="4"
            ry="4"
            fill="transparent"
            stroke="black"
            stroke-width="1.2"
          />
        </svg>
      </div>
    </v-btn>
  </div>
</template>

<style scoped>
.sudokupad-container {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  max-width: 600px;
  row-gap: 1em;
  column-gap: 1em;
}

.pad-button {
  width: 100%;
  height: auto !important;
  aspect-ratio: 1 / 1;
}

.pad-button svg {
  width: 100%;
  height: 100%;
}

.pad-button i {
  display: block;
  width: 50%;
  height: 50%;
}

.stack {
  position: relative;
  text-indent: 0 !important;
}

.stack > div {
  display: flex;
  justify-content: center;
  align-items: center;
  position: absolute;
  width: 100%;
  height: 100%;
  top: -2px;
  left: 0px;
  margin: auto;
}

.stack > svg {
  position: relative;
}

@media (max-width: 800px) {
  .sudokupad-container {
    max-width: 80%;
  }
}
</style>
