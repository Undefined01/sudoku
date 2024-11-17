<script setup lang="ts">
import { ref } from "vue";
const drawer = ref(false);
</script>

<template>
  <v-layout>
    <v-app-bar prominent>
      <v-app-bar-nav-icon
        variant="text"
        @click.stop="drawer = !drawer"
      ></v-app-bar-nav-icon>

      <v-toolbar-title>数独</v-toolbar-title>

      <v-spacer></v-spacer>

      <template v-if="$vuetify.display.mdAndUp">
        <v-btn icon="mdi-magnify" variant="text"></v-btn>

        <v-btn icon="mdi-filter" variant="text"></v-btn>
      </template>

      <v-btn icon="mdi-dots-vertical" variant="text"></v-btn>
    </v-app-bar>

    <v-navigation-drawer
      v-model="drawer"
      :location="$vuetify.display.mobile ? 'bottom' : undefined"
      temporary
    >
      <v-list>
        <v-list-item
          color="primary"
          :to="{ name: 'editor' }"
          title="编辑器"
        ></v-list-item>
        <v-list-item
          color="primary"
          :to="{ name: 'play' }"
          link
          title="游玩"
        ></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <RouterView />
  </v-layout>
</template>

<style scoped></style>
