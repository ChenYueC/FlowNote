import { defineStore } from "pinia";
import { ref } from "vue";
import type { Reminder, CreateReminderInput, UpdateReminderInput, ReminderCounts, ReminderListResponse } from "@/types";
import { invoke } from "@tauri-apps/api/core";

export type ReminderFilter = "all" | "today" | "overdue" | "cancelled" | "completed";

export const useRemindersStore = defineStore("reminders", () => {
  const reminders = ref<Reminder[]>([]);
  const counts = ref<ReminderCounts>({ all: 0, today: 0, overdue: 0, cancelled: 0, completed: 0 });
  const loading = ref(false);
  const activeFilter = ref<ReminderFilter>("all");

  function setFilter(filter: ReminderFilter) {
    activeFilter.value = filter;
    loadReminders();
  }

  async function loadReminders(noteId?: string) {
    loading.value = true;
    try {
      const res = await invoke<ReminderListResponse>("list_reminders", {
        noteId: noteId || null,
        filter: activeFilter.value,
      });
      reminders.value = res.reminders;
      counts.value = res.counts;
    } catch (e) {
      console.error("Failed to load reminders:", e);
    } finally {
      loading.value = false;
    }
  }

  async function createReminder(
    input: CreateReminderInput,
  ): Promise<Reminder> {
    try {
      const r = await invoke<Reminder>("create_reminder", { input });
      await loadReminders();
      return r;
    } catch (e) {
      console.error("Failed to create reminder:", e);
      throw e;
    }
  }

  async function updateReminder(
    input: UpdateReminderInput,
  ): Promise<Reminder | null> {
    try {
      const r = await invoke<Reminder>("update_reminder", { input });
      const idx = reminders.value.findIndex((x) => x.id === r.id);
      if (idx !== -1) reminders.value[idx] = r;
      return r;
    } catch (e) {
      console.error("Failed to update reminder:", e);
      return null;
    }
  }

  async function deleteReminder(id: string) {
    try {
      await invoke("delete_reminder", { id });
      await loadReminders();
    } catch (e) {
      console.error("Failed to delete reminder:", e);
    }
  }

  async function snoozeReminder(
    id: string,
    minutes: number,
  ): Promise<Reminder | null> {
    try {
      const r = await invoke<Reminder>("snooze_reminder", { id, minutes });
      await loadReminders();
      return r;
    } catch (e) {
      console.error("Failed to snooze reminder:", e);
      return null;
    }
  }

  async function completeReminder(id: string): Promise<Reminder | null> {
    try {
      const r = await invoke<Reminder>("complete_reminder", { id });
      await loadReminders();
      return r;
    } catch (e) {
      console.error("Failed to complete reminder:", e);
      return null;
    }
  }

  async function cancelReminder(id: string): Promise<Reminder | null> {
    try {
      const r = await invoke<Reminder>("cancel_reminder", { id });
      await loadReminders();
      return r;
    } catch (e) {
      console.error("Failed to cancel reminder:", e);
      return null;
    }
  }

  return {
    reminders,
    counts,
    loading,
    activeFilter,
    setFilter,
    loadReminders,
    createReminder,
    updateReminder,
    deleteReminder,
    snoozeReminder,
    completeReminder,
    cancelReminder,
  };
});
