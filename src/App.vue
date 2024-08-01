<script setup lang="ts">
import SudokuMain from './components/SudokuMain.vue'
import SudokuPad from './components/SudokuPad.vue'

import { provide } from 'vue'

import { Sudoku } from '@/models/sudoku'
import { defaultSettings } from '@/models/settings'

const sudoku = Sudoku.fromString(
  '.....6....637....22.....15.6..2.85....8...6....46.5..3.36.....11....328....1.....'
)
const settings = defaultSettings
provide('sudoku', sudoku)
provide('settings', settings)

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
