// src/stores/settings.ts
import { defineStore } from "pinia";
import { ref, reactive } from "vue";
import { emit, listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export const DEFAULT_SHORTCUTS: Record<string, string> = {
  bold: "mod-b",
  italic: "mod-i",
  heading: "mod-h",
  strikethrough: "mod-d",
  underline: "mod-e",
  bulletList: "shift-mod-u",
  orderedList: "shift-mod-o",
  taskList: "shift-mod-t",
  pastePlain: "shift-mod-c",
};

export const DEFAULT_SCREENSHOT_SHORTCUT = "alt-q";

export const SHORTCUT_LABELS: Record<string, string> = {
  bold: "加粗",
  italic: "斜体",
  heading: "标题",
  strikethrough: "删除线",
  underline: "下划线",
  bulletList: "无序列表",
  orderedList: "有序列表",
  taskList: "待办事项",
  pastePlain: "粘贴纯文本",
};

export type ThemeId = "default" | "hello-kitty" | "sakura-cream" | "sweet-lavender" | "cotton-lavender" | "galaxy-lavender";

export const THEMES: { id: ThemeId; label: string; desc: string; colors: string[] }[] = [
  { id: "default", label: "默认", desc: "深邃星空", colors: ["#1e1e2e", "#93c5fd", "#ffffff"] },
  { id: "hello-kitty", label: "Hello Kitty", desc: "草莓牛奶", colors: ["#3A2535", "#ff8fab", "#ffc4d2"] },
  { id: "sakura-cream", label: "樱花奶油", desc: "柔粉樱花", colors: ["#422B3D", "#ffa2ba", "#ffd2dc"] },
  { id: "sweet-lavender", label: "甜薰衣草", desc: "梦幻薰衣草", colors: ["#47304E", "#ffaac8", "#ffd7e6"] },
  { id: "cotton-lavender", label: "棉花薰衣草", desc: "柔紫棉花", colors: ["#2D2037", "#ffa0d2", "#dcb4ff"] },
  { id: "galaxy-lavender", label: "银河薰衣草", desc: "星空薰衣草", colors: ["#201830", "#be8cff", "#c8aaff"] },
];

export const useSettingsStore = defineStore("settings", () => {
  const shortcuts = ref<Record<string, string>>({ ...DEFAULT_SHORTCUTS });
  const appearance = reactive({ mainOpacity: 80, editorOpacity: 82, theme: "default" as ThemeId });
  const screenshot = reactive({
    fixedX: 0, fixedY: 0, fixedWidth: 800, fixedHeight: 600,
    fixedArea: false, shortcut: DEFAULT_SCREENSHOT_SHORTCUT,
  });
  const autostart = ref(false);
  const checkUpdate = ref(true);

  async function load() {
    try {
      const data = await invoke<Record<string, unknown>>("load_settings");
      if (data.shortcuts) shortcuts.value = { ...DEFAULT_SHORTCUTS, ...data.shortcuts as Record<string, string> };
      if (data.appearance) {
        const a = data.appearance as { mainOpacity?: number; editorOpacity?: number; theme?: string };
        if (typeof a.mainOpacity === "number") appearance.mainOpacity = a.mainOpacity;
        if (typeof a.editorOpacity === "number") appearance.editorOpacity = a.editorOpacity;
        if (typeof a.theme === "string") appearance.theme = a.theme as ThemeId;
      }
      if (data.screenshot) {
        const sc = data.screenshot as { fixedX?: number; fixedY?: number; fixedWidth?: number; fixedHeight?: number; fixedArea?: boolean; shortcut?: string };
        if (typeof sc.fixedX === "number") screenshot.fixedX = sc.fixedX;
        if (typeof sc.fixedY === "number") screenshot.fixedY = sc.fixedY;
        if (typeof sc.fixedWidth === "number") screenshot.fixedWidth = sc.fixedWidth;
        if (typeof sc.fixedHeight === "number") screenshot.fixedHeight = sc.fixedHeight;
        if (typeof sc.fixedArea === "boolean") screenshot.fixedArea = sc.fixedArea;
        if (typeof sc.shortcut === "string") screenshot.shortcut = sc.shortcut;
      }
      if (typeof data.checkUpdate === "boolean") checkUpdate.value = data.checkUpdate;
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
    applyAppearanceVars();
  }

  async function save(data: Record<string, unknown>) {
    try {
      await invoke("save_settings", { data });
      notifyOtherWindows();
    } catch (e) {
      console.error("Failed to save settings:", e);
      throw e;
    }
  }

  // 获取完整配置对象，新增字段只需改这一处
  function getSettings() {
    return {
      shortcuts: shortcuts.value,
      appearance,
      screenshot,
      checkUpdate: checkUpdate.value,
    };
  }

  async function saveSettings() {
    await save(getSettings());
  }

  async function notifyOtherWindows() {
    try { await emit("settings:updated"); } catch (e) { console.warn("Failed to notify other windows:", e); }
  }

  async function saveShortcuts(s: Record<string, string>) {
    shortcuts.value = { ...s };
    await saveSettings();
  }

  async function saveAppearance(a: { mainOpacity?: number; editorOpacity?: number; theme?: ThemeId }) {
    if (a.mainOpacity !== undefined) appearance.mainOpacity = Math.max(60, Math.min(100, a.mainOpacity));
    if (a.editorOpacity !== undefined) appearance.editorOpacity = Math.max(60, Math.min(100, a.editorOpacity));
    if (a.theme !== undefined) appearance.theme = a.theme;
    applyAppearanceVars();
    await saveSettings();
  }

  async function saveScreenshot(sc: { fixedX?: number; fixedY?: number; fixedWidth?: number; fixedHeight?: number; fixedArea?: boolean; shortcut?: string }) {
    if (sc.fixedX !== undefined) screenshot.fixedX = sc.fixedX;
    if (sc.fixedY !== undefined) screenshot.fixedY = sc.fixedY;
    if (sc.fixedWidth !== undefined) screenshot.fixedWidth = sc.fixedWidth;
    if (sc.fixedHeight !== undefined) screenshot.fixedHeight = sc.fixedHeight;
    if (sc.fixedArea !== undefined) screenshot.fixedArea = sc.fixedArea;
    if (sc.shortcut !== undefined) screenshot.shortcut = sc.shortcut;
    await saveSettings();
  }

  async function saveCheckUpdate(val: boolean) {
    checkUpdate.value = val;
    await saveSettings();
  }

  function applyAppearanceVars() {
    const root = document.documentElement;
    root.style.setProperty("--main-opacity", String(appearance.mainOpacity / 100));
    root.style.setProperty("--editor-opacity", String(appearance.editorOpacity / 100));
    // Apply theme
    if (appearance.theme && appearance.theme !== "default") {
      root.setAttribute("data-theme", appearance.theme);
    } else {
      root.removeAttribute("data-theme");
    }
  }

  async function loadAutostart() {
    try {
      autostart.value = await invoke<boolean>("get_autostart");
    } catch (e) {
      console.error("Failed to load autostart setting:", e);
    }
  }

  async function saveAutostart(enable: boolean) {
    autostart.value = enable;
    try {
      await invoke("set_autostart", { enable });
    } catch (e) {
      console.error("Failed to save autostart setting:", e);
      autostart.value = !enable; // Revert on failure
    }
  }

  let unlistenFn: (() => void) | null = null;

  async function initSync() {
    if (unlistenFn) return;
    unlistenFn = await listen("settings:updated", async () => {
      try {
        await load();
      } catch (e) {
        console.error("Failed to reload settings after update event:", e);
      }
    });
  }

  function cleanup() {
    if (unlistenFn) { unlistenFn(); unlistenFn = null; }
  }

  function findConflict(commandId: string, key: string): string | null {
    for (const [cmd, k] of Object.entries(shortcuts.value)) {
      if (cmd !== commandId && k === key) return cmd;
    }
    return null;
  }

  // --- Init (sequential, with error recovery) ---
  const initialized = ref(false);
  (async () => {
    try {
      await load();
    } catch (e) {
      console.error("Settings load failed:", e);
    }
    try {
      await loadAutostart();
    } catch (e) {
      console.error("Autostart load failed:", e);
    }
    try {
      await initSync();
    } catch (e) {
      console.error("Settings sync init failed:", e);
    }
    initialized.value = true;
  })();

  return {
    shortcuts, appearance, screenshot, autostart, checkUpdate, initialized,
    load, saveShortcuts, saveAppearance, saveScreenshot, saveAutostart, saveCheckUpdate,
    findConflict, cleanup,
  };

});
