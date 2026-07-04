import { defineStore } from "pinia";
import { ref } from "vue";
import type { Note, CreateNoteInput, UpdateNoteInput, TimelineItem } from "@/types";
import { invoke } from "@tauri-apps/api/core";

export const useNotesStore = defineStore("notes", () => {
  const notes = ref<Note[]>([]);
  const timelineItems = ref<TimelineItem[]>([]);
  const currentNote = ref<Note | null>(null);
  const searchQuery = ref("");
  const loading = ref(false);
  const currentWorkspace = ref("default");
  const workspaces = ref<string[]>(["default"]);
  const timelinePage = ref(1);
  const timelineHasMore = ref(true);

  async function loadNotes(workspace?: string) {
    const ws = workspace || currentWorkspace.value;
    currentWorkspace.value = ws;
    loading.value = true;
    try {
      notes.value = await invoke<Note[]>("list_notes", { workspace: ws });
    } catch (e) {
      console.error("Failed to load notes:", e);
    } finally {
      loading.value = false;
    }
  }

  async function loadTimeline(page = 1) {
    loading.value = true;
    try {
      const result = await invoke<TimelineItem[]>("list_timeline_items", { page });
      if (page === 1) {
        timelineItems.value = result;
      } else {
        timelineItems.value.push(...result);
      }
      timelinePage.value = page;
      timelineHasMore.value = result.length >= 30;
    } catch (e) {
      console.error("Failed to load timeline:", e);
    } finally {
      loading.value = false;
    }
  }

  async function loadMoreTimeline() {
    if (!timelineHasMore.value || loading.value) return;
    await loadTimeline(timelinePage.value + 1);
  }

  async function loadWorkspaces() {
    try {
      workspaces.value = await invoke<string[]>("list_workspaces");
      if (!workspaces.value.includes("default")) {
        workspaces.value.unshift("default");
      }
    } catch (e) {
      console.error("Failed to load workspaces:", e);
    }
  }

  async function addWorkspace(name: string) {
    try {
      await invoke("create_workspace", { name });
      if (!workspaces.value.includes(name)) {
        workspaces.value.push(name);
      }
    } catch (e) {
      console.error("Failed to add workspace:", e);
    }
  }

  async function removeWorkspace(name: string) {
    try {
      await invoke("delete_workspace", { name });
      workspaces.value = workspaces.value.filter((w) => w !== name);
    } catch (e) {
      console.error("Failed to remove workspace:", e);
    }
  }

  async function renameWorkspace(oldName: string, newName: string) {
    try {
      await invoke("rename_workspace", { oldName, newName });
      const idx = workspaces.value.indexOf(oldName);
      if (idx !== -1) workspaces.value[idx] = newName;
      if (currentWorkspace.value === oldName) {
        currentWorkspace.value = newName;
      }
    } catch (e) {
      console.error("Failed to rename workspace:", e);
    }
  }

  async function searchNotes(query: string, workspace?: string) {
    searchQuery.value = query;
    const ws = workspace || currentWorkspace.value;
    if (!query.trim()) {
      await loadNotes(ws);
      return;
    }
    loading.value = true;
    try {
      notes.value = await invoke<Note[]>("search_notes", {
        query,
        workspace: ws,
      });
    } catch (e) {
      console.error("Failed to search notes:", e);
    } finally {
      loading.value = false;
    }
  }

  async function createNote(
    input: CreateNoteInput,
    workspace?: string,
  ): Promise<Note | null> {
    try {
      const note = await invoke<Note>("create_note", {
        input: { ...input, workspace: workspace || currentWorkspace.value },
      });
      if (
        !workspace ||
        workspace === currentWorkspace.value
      ) {
        notes.value.unshift(note);
      }
      return note;
    } catch (e) {
      console.error("Failed to create note:", e);
      return null;
    }
  }

  async function createDailyNote(): Promise<Note | null> {
    try {
      const note = await invoke<Note>("create_daily_note");
      return note;
    } catch (e) {
      console.error("Failed to create daily note:", e);
      return null;
    }
  }

  async function updateNote(input: UpdateNoteInput): Promise<Note | null> {
    try {
      const note = await invoke<Note>("update_note", { input });
      const idx = notes.value.findIndex((n) => n.id === note.id);
      if (idx !== -1) notes.value[idx] = note;
      if (currentNote.value?.id === note.id) currentNote.value = note;
      return note;
    } catch (e) {
      console.error("Failed to update note:", e);
      return null;
    }
  }

  async function deleteNote(id: string) {
    try {
      await invoke("delete_note", { id });
      notes.value = notes.value.filter((n) => n.id !== id);
      timelineItems.value = timelineItems.value.map((n) => n.id === id ? { ...n, archived: 1 } : n);
      if (currentNote.value?.id === id) currentNote.value = null;
    } catch (e) {
      console.error("Failed to delete note:", e);
    }
  }

  async function toggleFavorite(id: string) {
    try {
      const note = await invoke<Note>("toggle_favorite", { id });
      notes.value = notes.value.map((n) => n.id === id ? note : n);
      // TimelineItem doesn't have favorite field, no update needed
      if (currentNote.value?.id === id) currentNote.value = note;
    } catch (e) {
      console.error("Failed to toggle favorite:", e);
    }
  }

  async function moveToWorkspace(id: string, workspace: string) {
    try {
      const note = await invoke<Note>("move_note_to_workspace", {
        id,
        workspace,
      });
      notes.value = notes.value.filter((n) => n.id !== id);
      return note;
    } catch (e) {
      console.error("Failed to move note:", e);
      return null;
    }
  }

  async function getNote(id: string): Promise<Note | null> {
    try {
      return await invoke<Note>("get_note", { id });
    } catch (e) {
      console.error("Failed to get note:", e);
      return null;
    }
  }

  function setCurrentNote(note: Note | null) {
    currentNote.value = note;
  }

  return {
    notes,
    timelineItems,
    currentNote,
    searchQuery,
    loading,
    currentWorkspace,
    workspaces,
    loadNotes,
    loadTimeline,
    loadMoreTimeline,
    timelineHasMore,
    timelinePage,
    loadWorkspaces,
    addWorkspace,
    removeWorkspace,
    renameWorkspace,
    searchNotes,
    createNote,
    createDailyNote,
    updateNote,
    deleteNote,
    toggleFavorite,
    moveToWorkspace,
    getNote,
    setCurrentNote,
  };
});
