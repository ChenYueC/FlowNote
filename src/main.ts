import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./assets/styles/global.css";

// Disable browser default context menu on all pages
document.addEventListener("contextmenu", (e) => e.preventDefault());
// Disable browser shortcuts that trigger unwanted behavior
document.addEventListener("keydown", (e) => {
  // Function keys (F1-F11, skip F12 for DevTools)
  if (e.key.startsWith("F") && /^F\d+$/.test(e.key) && e.key !== "F12") {
    e.preventDefault();
    return;
  }
  // Ctrl+key browser shortcuts
  if (e.ctrlKey && !e.shiftKey && !e.altKey) {
    const blocked = ["r", "p", "s", "d", "f", "h", "j", "u", "l"];
    if (blocked.includes(e.key.toLowerCase())) {
      e.preventDefault();
      return;
    }
  }
  // Alt+Left/Right (history back/forward)
  if (e.altKey && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
    e.preventDefault();
  }
});

const app = createApp(App);
app.use(createPinia());
app.mount("#app");
