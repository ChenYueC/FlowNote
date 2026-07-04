// src/composables/useWindowClose.ts
import { ref } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit } from "@tauri-apps/api/event";

interface UseWindowCloseOptions {
  saveImmediately: () => Promise<void>;
}

export function useWindowClose(options: UseWindowCloseOptions) {
  const { saveImmediately } = options;
  const showCloseConfirm = ref(false);

  function handleClose() {
    showCloseConfirm.value = true;
  }

  async function confirmClose() {
    await saveImmediately();
    // Clean up image preview localStorage entries
    for (let i = localStorage.length - 1; i >= 0; i--) {
      const key = localStorage.key(i);
      if (key?.startsWith("image-preview-")) {
        localStorage.removeItem(key);
      }
    }
    // Destroy image preview windows (in case they're still open)
    const allWindows = await WebviewWindow.getAll();
    for (const w of allWindows) {
      if (w.label.startsWith("image-preview-")) {
        w.destroy();
      }
    }
    emit("editor:closed");
    getCurrentWebviewWindow().destroy();
  }

  function cancelClose() {
    showCloseConfirm.value = false;
  }

  return { showCloseConfirm, handleClose, confirmClose, cancelClose };
}
