import { createApp } from "vue";
import { enableMapSet } from "immer";
import Vuetify from "./plugins/Vuetify";
import router from "./plugins/router";

import "./style.css";
import App from "./App.vue";

enableMapSet();

createApp(App).use(Vuetify).use(router).mount("#app");
