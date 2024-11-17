<script setup lang="ts">
import {
  mdiAutoFix,
  mdiBackspaceOutline,
  mdiPalette,
  mdiRedo,
  mdiStrategy,
  mdiUndo,
} from "@mdi/js";

import { inject, ref, computed } from "vue";
import { Sudoku } from "@/models/sudoku";
import {
  Sudoku as RustSudoku,
  SudokuSolver,
  Techniques,
  StepKind,
} from "sudoku-solver";
import { PadMode } from "./SudokuPad/Pad";
import PadButton from "./SudokuPad/PadButton.vue";
import { Settings } from "@/models/settings";

const sudoku = inject<Sudoku>("sudoku")!;
const settings = inject<Settings>("settings")!;

const mode = ref<PadMode>("value");

const numberLayout = computed(() => {
  if (settings.appearance.sudoku.useCellphoneLayoutForPadNumbers) {
    return [
      [1, 2, 3],
      [4, 5, 6],
      [7, 8, 9],
    ];
  } else {
    return [
      [7, 8, 9],
      [4, 5, 6],
      [1, 2, 3],
    ];
  }
});

const deleteSelected = () => {
  sudoku.updateState(true, (state) => {
    if (mode.value === "value") {
      state.selectedCells.values().forEach((cell) => {
        state.getCell(cell).setValue(undefined);
        state.getCell(cell).candidates.clear();
        state.getCell(cell).pencilMarks.clear();
      });
    } else if (mode.value === "candidate") {
      state.selectedCells.values().forEach((cell) => {
        state.getCell(cell).candidates.clear();
      });
    } else if (mode.value === "pencilMark") {
      state.selectedCells.values().forEach((cell) => {
        state.getCell(cell).pencilMarks.clear();
      });
    } else if (mode.value === "color") {
      state.selectedCells.values().forEach((cell) => {
        state.getCell(cell).colors.clear();
      });
    }
  });
};

let solver: SudokuSolver | undefined = undefined;
const reloadSolver = () => {
  const rustSudoku = RustSudoku.from_candidates(
    sudoku.state.toCandidateString(),
  );
  solver = SudokuSolver.new(rustSudoku);
};
const fillPencilMarks = () => {
  const rustSudoku = RustSudoku.from_values(sudoku.toValueString());
  solver = SudokuSolver.new(rustSudoku);
  solver.initialize_candidates();
  const candidateStr = solver.take_sudoku().to_candidate_string();
  sudoku.updateState(true, (state) => {
    state.fillFromCandidateString(candidateStr);
  });
};
const solveOneStep = () => {
  if (solver === undefined) {
    fillPencilMarks();
  }
  solver = solver as SudokuSolver;
  console.time("wasm solveOneStep");
  const action = solver.solve_one_step(Techniques.default_techniques());
  console.timeEnd("wasm solveOneStep");
  if (action !== undefined) {
    console.log(action.to_string(solver.take_sudoku()));
    solver.apply_step(action);
    sudoku.updateState(true, (state) => {
      for (const step of action.steps) {
        const cell = state.cells[step.cell_index];
        if (step.kind === StepKind.ValueSet) {
          cell.setValue(step.value);
        } else if (step.kind === StepKind.CandidateEliminated) {
          if (cell.value === undefined && cell.pencilMarks.has(step.value)) {
            cell.pencilMarks.delete(step.value);
          }
        } else {
          console.log("Unknown step kind", step.kind);
        }
      }
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
    <v-btn class="pad-button"> </v-btn>
    <v-btn class="pad-button"> </v-btn>

    <pad-button
      v-for="idx in numberLayout[0]"
      :key="idx"
      :mode="mode"
      :idx="idx"
    />

    <!-- Value Switch -->
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

    <pad-button
      v-for="idx in numberLayout[1]"
      :key="idx"
      :mode="mode"
      :idx="idx"
    />

    <!-- Pencil Mark Switch -->
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

    <pad-button
      v-for="idx in numberLayout[2]"
      :key="idx"
      :mode="mode"
      :idx="idx"
    />

    <!-- Candidate Switch -->
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

    <v-btn
      class="pad-button"
      @click="
        () => {
          sudoku.undo();
          reloadSolver();
        }
      "
    >
      <v-icon :icon="mdiUndo" />
    </v-btn>

    <v-btn
      class="pad-button"
      @click="
        () => {
          sudoku.redo();
          reloadSolver();
        }
      "
    >
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
  min-width: auto;
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
