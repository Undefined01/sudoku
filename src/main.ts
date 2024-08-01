import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { enableMapSet } from "immer"

enableMapSet()

createApp(App).mount('#app')
