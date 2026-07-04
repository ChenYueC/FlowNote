<template>
  <div class="reminder-view h-full flex flex-col">
    <!-- Header -->
    <div class="drag-region flex items-center justify-between px-4 pt-4 pb-3">
      <button
        class="no-drag flex items-center gap-1 px-2 py-1 rounded-lg text-white/50 hover:text-white hover:bg-white/8 transition-all"
        @click="$emit('close')"
      >
        <svg
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M15 18l-6-6 6-6" />
        </svg>
        <span class="text-xs">返回</span>
      </button>
      <h2 class="text-sm font-medium text-white/80 absolute left-1/2 -translate-x-1/2">提醒列表</h2>
      <div class="w-16"></div>
    </div>

    <!-- Filter tabs -->
    <div class="flex justify-center gap-0.5 px-2 mb-3">
      <button
        v-for="filter in filters"
        :key="filter.value"
        class="no-drag whitespace-nowrap px-2 py-1 rounded-full text-[11px] font-medium transition-all duration-200"
        :class="activeFilter === filter.value ? 'bg-white/15 text-white' : 'text-white/40 hover:text-white/70'"
        @click="handleFilterChange(filter.value)"
      >
        {{ filter.label }}{{ formatFilterCount(filter.value) }}
      </button>
    </div>

    <!-- Content area -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- Loading state -->
      <div
        v-if="loading"
        class="flex-1 flex items-center justify-center"
      >
        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-white/30" />
      </div>

      <!-- Empty state -->
      <div
        v-else-if="filteredReminders.length === 0"
        class="flex-1 flex flex-col items-center justify-center text-white/30"
      >
        <svg
          class="w-12 h-12 mb-3 opacity-50"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <span class="text-sm">暂无提醒</span>
      </div>

      <!-- Reminder cards -->
      <div v-else class="flex-1 overflow-y-auto px-4 py-3 space-y-[10px]">
        <ReminderCard
          v-for="reminder in filteredReminders"
          :key="reminder.id"
          :reminder="reminder"
          :note-title="getNoteTitle(reminder.note_id ?? undefined)"
          @complete="handleComplete"
          @snooze="handleSnooze"
          @open-note="handleOpenNote"
          @delete="handleDeleteRequest"
          @cancel="handleCancelRequest"
        />
      </div>
    </div>

    <!-- Delete confirmation dialog -->
    <div
      v-if="deleteTarget"
      class="absolute inset-0 z-[9999] flex items-center justify-center bg-black/40 rounded-3xl overflow-hidden"
      @click.self="deleteTarget = null"
    >
      <div class="glass-strong rounded-2xl p-6 w-72 animate-scale-in">
        <p class="text-sm text-white/80 mb-2">确认删除这条提醒？</p>
        <p class="text-xs text-white/40 mb-4 truncate">{{ deleteTarget.title || "无标题" }}</p>
        <div class="flex gap-2 justify-end">
          <button class="no-drag px-4 py-1.5 rounded-full text-xs text-white/50 hover:text-white/80 hover:bg-white/8 transition-all" @click="deleteTarget = null">取消</button>
          <button class="no-drag px-4 py-1.5 rounded-full text-xs text-red-300 bg-red-400/10 hover:bg-red-400/20 transition-all" @click="confirmDelete">删除</button>
        </div>
      </div>
    </div>

    <!-- Cancel confirmation dialog -->
    <div
      v-if="cancelTarget"
      class="absolute inset-0 z-[9999] flex items-center justify-center bg-black/40 rounded-3xl overflow-hidden"
      @click.self="cancelTarget = null"
    >
      <div class="glass-strong rounded-2xl p-6 w-72 animate-scale-in">
        <p class="text-sm text-white/80 mb-2">是否取消提醒？</p>
        <p class="text-xs text-white/40 mb-4 truncate">{{ cancelTarget.title || "无标题" }}</p>
        <div class="flex gap-2 justify-end">
          <button class="no-drag px-4 py-1.5 rounded-full text-xs text-white/50 hover:text-white/80 hover:bg-white/8 transition-all" @click="cancelTarget = null">返回</button>
          <button class="no-drag px-4 py-1.5 rounded-full text-xs text-white/60 bg-white/8 hover:bg-white/15 transition-all" @click="confirmCancel">取消提醒</button>
        </div>
      </div>
    </div>

    <!-- Quick add button -->
    <div class="px-4 py-3">
      <button
        class="no-drag w-full py-2.5 rounded-xl text-white/40 hover:text-white/60 hover:bg-white/5 transition-all duration-200 active:scale-98 flex items-center justify-center gap-2"
        style="background: rgba(var(--interactive), 0.05); border: 1px solid rgba(var(--border), 0.08);"
        @click="showCreateModal = true"
      >
        <svg
          class="w-4 h-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 4v16m8-8H4"
          />
        </svg>
        <span class="text-sm font-medium">新建提醒</span>
      </button>
    </div>

    <!-- Create reminder modal -->
    <CreateReminderModal :visible="showCreateModal" @close="showCreateModal = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRemindersStore } from "@/stores/reminders";
import { useNotesStore } from "@/stores/notes";
import ReminderCard from "@/components/notes/ReminderCard.vue";
import CreateReminderModal from "@/components/layout/CreateReminderModal.vue";
import type { ReminderFilter } from "@/stores/reminders";
import type { Reminder } from "@/types";
import { emit as tauriEmit } from "@tauri-apps/api/event";

defineEmits<{
  close: [];
}>();

const remindersStore = useRemindersStore();
const notesStore = useNotesStore();

const loading = computed(() => remindersStore.loading);
const activeFilter = computed(() => remindersStore.activeFilter);

const filteredReminders = computed(() => remindersStore.reminders);

function handleFilterChange(filter: ReminderFilter) {
  remindersStore.setFilter(filter);
}

const showCreateModal = ref(false);

const filters: { label: string; value: ReminderFilter }[] = [
  { label: "全部", value: "all" },
  { label: "今天", value: "today" },
  { label: "已过期", value: "overdue" },
  { label: "已取消", value: "cancelled" },
  { label: "已完成", value: "completed" },
];

const notesMap = computed(() => {
  const map = new Map<string, string>();
  notesStore.notes.forEach((note) => {
    map.set(note.id, note.title);
  });
  return map;
});

function formatFilterCount(filter: ReminderFilter): string {
  const count = remindersStore.counts[filter] || 0;
  if (count > 99) return " 99+";
  return ` ${count}`;
}

function getNoteTitle(noteId?: string): string {
  if (!noteId) return "";
  return notesMap.value.get(noteId) || noteId;
}

async function handleComplete(id: string) {
  await remindersStore.completeReminder(id);
  tauriEmit("reminder:updated").catch(() => {});
}

async function handleSnooze(id: string, minutes: number) {
  await remindersStore.snoozeReminder(id, minutes);
  tauriEmit("reminder:updated").catch(() => {});
}

const deleteTarget = ref<Reminder | null>(null);

function handleDeleteRequest(id: string) {
  const reminder = remindersStore.reminders.find((r) => r.id === id);
  if (reminder) deleteTarget.value = reminder;
}

async function confirmDelete() {
  if (!deleteTarget.value) return;
  await remindersStore.deleteReminder(deleteTarget.value.id);
  tauriEmit("reminder:updated").catch(() => {});
  deleteTarget.value = null;
}

const cancelTarget = ref<Reminder | null>(null);

function handleCancelRequest(id: string) {
  const reminder = remindersStore.reminders.find((r) => r.id === id);
  if (reminder) cancelTarget.value = reminder;
}

async function confirmCancel() {
  if (!cancelTarget.value) return;
  await remindersStore.cancelReminder(cancelTarget.value.id);
  tauriEmit("reminder:updated").catch(() => {});
  cancelTarget.value = null;
}

function handleOpenNote(noteId: string) {
  // This will be handled by the parent component
  // For now, we'll emit an event or use the notes store
  notesStore.getNote(noteId);
}

onMounted(async () => {
  await remindersStore.loadReminders();
  if (notesStore.notes.length === 0) {
    await notesStore.loadNotes();
  }
});
</script>

<style scoped>
.reminder-view {
  position: relative;
}

/* Custom scrollbar */
.reminder-view ::-webkit-scrollbar {
  width: 5px;
}

.reminder-view ::-webkit-scrollbar-track {
  background: transparent;
}

.reminder-view ::-webkit-scrollbar-thumb {
  background: rgba(var(--scrollbar), 0.1);
  border-radius: 999px;
}

.reminder-view ::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--scrollbar-hover), 0.2);
}
</style>
