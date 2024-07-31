<script setup lang="ts">
import { SudokuCell } from '@/models/sudoku'

const { cells } = defineProps<{ cells: SudokuCell[] }>()

const cellLength = 100
const cornerOffset = 30
const pencilMarkPosition = [
    { x: 0, y: 0 },

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

</script>

<template>
    <g>
        <template v-for="cell in cells">
            <text v-if="cell.value !== undefined" :x="cell.position.column * 100 + 50" :y="cell.position.row * 100 + 50"
                class="sudoku-cell-text" text-anchor="middle" dominant-baseline="central" font-size="60"
                :fill="cell.isGiven ? 'black' : 'blue'">
                {{ cell.value }}
            </text>
            <text v-if="cell.pencilMarks.length > 0"
                :x="cell.position.column * 100 + 50"
                :y="cell.position.row * 100 + 50" class="sudoku-cell-text"
                text-anchor="middle" dominant-baseline="central" font-size="20" fill="blue">
                {{ cell.pencilMarks.join('') }}
            </text>
            <text v-for="candidate in cell.candidates" :key="candidate"
                :x="cell.position.column * 100 + pencilMarkPosition[candidate].x"
                :y="cell.position.row * 100 + pencilMarkPosition[candidate].y" class="sudoku-cell-text"
                text-anchor="middle" dominant-baseline="central" font-size="20" fill="blue">
                {{ candidate }}
            </text>
        </template>
    </g>
</template>

<style scoped>
.sudoku-cell-text {
    user-select: none;
}
</style>
