import { defineStore } from "pinia";
import { ref } from "vue";
import type {
  FloatingWindow,
  CreateFloatingWindowInput,
  UpdateWindowStateInput,
} from "@/types";
import { invoke } from "@tauri-apps/api/core";

export const useWindowsStore = defineStore("windows", () => {
  const floatingWindows = ref<FloatingWindow[]>([]);

  async function createFloating(
    input: CreateFloatingWindowInput,
  ): Promise<FloatingWindow | null> {
    try {
      const fw = await invoke<FloatingWindow>("create_floating_window", {
        input,
      });
      floatingWindows.value.push(fw);
      return fw;
    } catch (e) {
      console.error("Failed to create floating window:", e);
      return null;
    }
  }

  async function closeFloating(windowId: string) {
    try {
      await invoke("close_floating_window", { windowId });
      floatingWindows.value = floatingWindows.value.filter(
        (w) => w.id !== windowId,
      );
    } catch (e) {
      console.error("Failed to close floating window:", e);
    }
  }

  async function updateWindowState(input: UpdateWindowStateInput) {
    try {
      await invoke("update_window_state", { input });
      const idx = floatingWindows.value.findIndex((w) => w.id === input.id);
      if (idx !== -1) {
        const w = floatingWindows.value[idx];
        if (input.x !== undefined) w.x = input.x;
        if (input.y !== undefined) w.y = input.y;
        if (input.pinned !== undefined) w.pinned = input.pinned;
        if (input.opacity !== undefined) w.opacity = input.opacity;
        if (input.auto_hide !== undefined) w.auto_hide = input.auto_hide;
      }
    } catch (e) {
      console.error("Failed to update window state:", e);
    }
  }

  async function loadFloatingWindows() {
    try {
      floatingWindows.value =
        await invoke<FloatingWindow[]>("list_floating_windows");
    } catch (e) {
      console.error("Failed to load floating windows:", e);
    }
  }

  return {
    floatingWindows,
    createFloating,
    closeFloating,
    updateWindowState,
    loadFloatingWindows,
  };
});
