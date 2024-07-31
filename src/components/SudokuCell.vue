<script setup lang="ts">
import { SudokuCell } from '@/models/sudoku'

const { cells } = defineProps<{ cells: SudokuCell[] }>()

const cellLength = 100
const cornerOffset = 30
const pencilMarkPosition = [
    { x: cellLength / 2 - cornerOffset * 0.5, y: cellLength / 2 - cornerOffset * 0.6 },

    { x: cellLength / 2 - cornerOffset, y: cellLength / 2 - cornerOffset },
    { x: cellLength / 2, y: cellLength / 2 - cornerOffset },
    { x: cellLength / 2 + cornerOffset, y: cellLength / 2 - cornerOffset },

    { x: cellLength / 2 - cornerOffset, y: cellLength / 2 },
    { x: cellLength / 2, y: cellLength / 2 },
    { x: cellLength / 2 + cornerOffset, y: cellLength / 2 },

    { x: cellLength / 2 - cornerOffset, y: cellLength / 2 + cornerOffset },
    { x: cellLength / 2, y: cellLength / 2 + cornerOffset },
    { x: cellLength / 2 + cornerOffset, y: cellLength / 2 + cornerOffset },
]
const fivePositionOnHavingCandidates = { x: cellLength / 2 + cornerOffset * 0.5, y: cellLength / 2 - cornerOffset * 0.6 }

</script>

<template>
    <g class="sudoku-cell">
        <template v-for="cell in cells">
            <text v-if="cell.value !== undefined" :x="cell.position.column * 100 + 50" :y="cell.position.row * 100 + 50"
                class="sudoku-cell-text" text-anchor="middle" dominant-baseline="central" font-size="60"
                :fill="cell.isGiven ? 'black' : 'blue'">
                {{ cell.value }}
            </text>
            <text v-if="cell.candidates.length > 0" :x="cell.position.column * 100 + 50"
                :y="cell.position.row * 100 + 50" class="sudoku-cell-text" text-anchor="middle"
                dominant-baseline="central" font-size="25" fill="blue">
                {{ cell.candidates.join('') }}
            </text>
            <template v-for="pencilMark in cell.pencilMarks" :key="pencilMark">
                <text v-if="!(pencilMark === 5 && cell.candidates.length > 0)"
                    :x="cell.position.column * 100 + pencilMarkPosition[pencilMark].x"
                    :y="cell.position.row * 100 + pencilMarkPosition[pencilMark].y" class="sudoku-cell-text"
                    text-anchor="middle" dominant-baseline="central" font-size="20" fill="gray">
                    {{ pencilMark }}
                </text>
                <text v-else="cell.pencilMarks.includes(5) && cell.candidates.length > 0"
                    :x="cell.position.column * 100 + fivePositionOnHavingCandidates.x"
                    :y="cell.position.row * 100 + fivePositionOnHavingCandidates.y" class="sudoku-cell-text"
                    text-anchor="middle" dominant-baseline="central" font-size="20" fill="gray">
                    5
                </text>
            </template>
        </template>
    </g>
</template>

<style scoped>
.sudoku-cell-text {
    user-select: none;
}
</style>
