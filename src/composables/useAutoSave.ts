// src/composables/useAutoSave.ts
import { type Ref, onUnmounted } from "vue";
import { useNotesStore } from "@/stores/notes";

interface UseAutoSaveOptions {
  noteId: Ref<string>;
  title: Ref<string>;
  content: Ref<string>;
}

export function useAutoSave(options: UseAutoSaveOptions) {
  const { noteId, title, content } = options;
  const notesStore = useNotesStore();

  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let lastSavedContent = "";
  let lastSavedTitle = "";
  let isSaving = false; // Mutex to prevent concurrent saves

  async function doSave() {
    if (isSaving) return;
    if (title.value === lastSavedTitle && content.value === lastSavedContent) return;

    // Capture current values before saving
    const currentTitle = title.value;
    const currentContent = content.value;

    isSaving = true;
    try {
      await notesStore.updateNote({
        id: noteId.value,
        title: currentTitle,
        content: currentContent,
      });
      // Only mark as saved after successful save
      lastSavedTitle = currentTitle;
      lastSavedContent = currentContent;
    } catch (err) {
      console.error("Auto-save failed:", err);
      // Don't update lastSaved — content will be retried on next save cycle
    } finally {
      isSaving = false;
    }
  }

  function markDirty() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      doSave();
    }, 500);
  }

  async function saveImmediately() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    await doSave();
  }

  async function flush() {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
    await doSave();
  }

  // Safety net: periodic save
  const intervalId = setInterval(() => {
    doSave();
  }, 3000);

  onUnmounted(() => {
    if (saveTimer) clearTimeout(saveTimer);
    clearInterval(intervalId);
  });

  return { markDirty, saveImmediately, flush };
}
