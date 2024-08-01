import { createApp } from "vue";
import { enableMapSet } from "immer";
import Vuetify from "./plugins/Vuetify";

import "./style.css";
import App from "./App.vue";

enableMapSet();

createApp(App).use(Vuetify).mount("#app");
