<script setup lang="ts">
import { inject } from "vue";
import { Sudoku } from "@/models/sudoku";

const sudoku = inject<Sudoku>("sudoku")!;
const { metadata } = sudoku;
const { rows, columns } = metadata;
const decorations = metadata.decorations;
const { boldRows, boldColumns } = decorations;
</script>

<template>
  <g class="sudoku-grid">
    <line
      v-for="(_, row) in rows + 1"
      :key="row"
      x1="0"
      :y1="row * 100"
      :x2="rows * 100"
      :y2="row * 100"
      stroke="black"
      stroke-width="1"
    />
    <line
      v-for="(_, column) in columns + 1"
      :key="column"
      :x1="column * 100"
      y1="0"
      :x2="column * 100"
      :y2="columns * 100"
      stroke="black"
      stroke-width="1"
    />

    <line
      v-for="row in boldRows"
      :key="row"
      :x1="row * 100"
      y1="0"
      :x2="row * 100"
      :y2="rows * 100"
      stroke="black"
      stroke-width="3"
    />
    <line
      v-for="column in boldColumns"
      :key="column"
      x1="0"
      :y1="column * 100"
      :x2="columns * 100"
      :y2="column * 100"
      stroke="black"
      stroke-width="3"
    />
  </g>
</template>

<style scoped></style>
