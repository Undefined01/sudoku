import { createWebHashHistory, createRouter } from "vue-router";

import EditorView from "../EditorView.vue";
import PlayView from "../PlayView.vue";

const routes = [
  { name: "play", path: "/", component: PlayView },
  { name: "editor", path: "/editor", component: EditorView },
];

export default createRouter({
  history: createWebHashHistory(),
  routes,
});
