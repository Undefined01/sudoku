<script setup lang="ts">
import SudokuMain from "./components/SudokuMain.vue";
import SudokuPad from "./components/SudokuPad.vue";

import { provide } from "vue";

import { Sudoku } from "@/models/sudoku";
import { defaultSettings } from "@/models/settings";

const sudoku = Sudoku.fromString(
  // "..67...91..9....623............3...4..72...1.4....1....31..8.75...9......65....3.",
  "9.7..5...1..7..9..86..9.57..8...61.9316.59..72.91..65.....2..96.9...4..8...9..3.5",
  // ".5..346..........8.3.879....15.....6...26..5.......92..4..27.13.73...........87..",
);
const settings = defaultSettings;
provide("sudoku", sudoku);
provide("settings", settings);
</script>

<template>
  <div class="sudoku-container">
    <SudokuMain class="sudoku-main" />
    <SudokuPad class="sudoku-pad" />
  </div>
</template>

<style scoped>
.sudoku-container {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5vw;
}

.sudoku-main {
  flex: 0 2 min(50vw, 70vh);
  aspect-ratio: 1;
}

.sudoku-pad {
  flex: 0 3 25vw;
}

@media (min-width: 1400px) {
  .sudoku-pad {
    flex-basis: auto;
    max-width: 360px;
  }
}

@media (max-width: 800px) {
  .sudoku-container {
    flex-direction: column;
    gap: 10vw;
  }

  .sudoku-main {
    flex-basis: auto;
    width: 80%;
  }
  .sudoku-pad {
    flex-basis: auto;
    width: 75%;
  }
}
</style>
