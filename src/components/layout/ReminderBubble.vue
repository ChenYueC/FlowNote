<template>
  <div
    class="h-full w-full select-none overflow-hidden"
    data-window="reminder"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
  >
    <div
      class="glass-strong w-full h-full flex flex-col animate-slide-in"
      style="background: rgba(var(--bg-primary), 0.95); backdrop-filter: blur(10px) saturate(130%); border: 1px solid rgba(var(--border), 0.12); border-radius: 16px;"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-4 pt-3 pb-1">
        <div class="flex items-center gap-1.5 min-w-0">
          <svg class="w-3.5 h-3.5 text-amber-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <h3 class="text-sm font-semibold text-white/90 truncate">提醒 · {{ reminder?.title || "" }}</h3>
        </div>
        <div class="flex-shrink-0 ml-2">
          <div class="w-5 h-5 rounded-full flex items-center justify-center" :style="countdownRingStyle">
            <span class="text-[8px] font-bold text-white/70">{{ countdown }}</span>
          </div>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 px-4 py-1 overflow-hidden">
        <p class="text-xs text-white/50 line-clamp-2">{{ reminder?.content || '您有一条新的提醒通知~' }}</p>
      </div>

      <!-- Actions -->
      <div class="flex gap-2 px-4 pb-3 pt-1">
        <button
          class="no-drag action-btn flex-1 py-1.5 rounded-lg text-[11px] font-medium transition-all duration-150 active:scale-[0.97]"
          @click.stop="handleComplete"
        >
          完成
        </button>
        <button
          class="no-drag action-btn flex-1 py-1.5 rounded-lg text-[11px] font-medium transition-all duration-150 active:scale-[0.97]"
          @click.stop="handleSnooze"
        >
          稍后提醒
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useRemindersStore } from "@/stores/reminders";
import { invoke } from "@tauri-apps/api/core";
import { emit as tauriEmit } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { Reminder } from "@/types";

const props = defineProps<{
  reminderId: string;
}>();

const store = useRemindersStore();
const reminder = ref<Reminder | null>(null);
const isHovered = ref(false);
const countdown = ref(60);
let countdownTimer: ReturnType<typeof setInterval> | null = null;
let autoSnoozeTimer: ReturnType<typeof setTimeout> | null = null;

const TOTAL_SECONDS = 60;

const countdownRingStyle = computed(() => {
  const pct = countdown.value / TOTAL_SECONDS;
  const deg = Math.round(pct * 360);
  return {
    background: `conic-gradient(rgba(255,255,255,0.3) ${deg}deg, rgba(255,255,255,0.05) ${deg}deg)`,
  };
});

function startCountdown() {
  countdownTimer = setInterval(() => {
    if (isHovered.value) return;
    countdown.value--;
    if (countdown.value <= 0) {
      handleSnooze();
    }
  }, 1000);
}

async function handleComplete() {
  if (!reminder.value) return;
  cleanup();
  const id = reminder.value.id;
  // Fire-and-forget: call backend directly, then close immediately
  invoke("complete_reminder", { id }).catch(() => {});
  tauriEmit("reminder:updated").catch(() => {});
  getCurrentWebviewWindow().close();
}

async function handleSnooze() {
  if (!reminder.value) return;
  cleanup();
  const id = reminder.value.id;
  invoke("snooze_reminder", { id, minutes: 5 }).catch(() => {});
  tauriEmit("reminder:updated").catch(() => {});
  getCurrentWebviewWindow().close();
}

function cleanup() {
  if (countdownTimer) { clearInterval(countdownTimer); countdownTimer = null; }
  if (autoSnoozeTimer) { clearTimeout(autoSnoozeTimer); autoSnoozeTimer = null; }
}

onMounted(async () => {
  await store.loadReminders();
  reminder.value = store.reminders.find((r) => r.id === props.reminderId) || null;
  if (!reminder.value) {
    try {
      const r = await invoke("get_reminder", { id: props.reminderId });
      reminder.value = r as Reminder;
    } catch { /* ignore */ }
  }
  startCountdown();
});

onUnmounted(() => {
  cleanup();
});
</script>

<style scoped>
@keyframes slide-in {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.animate-slide-in {
  animation: slide-in 0.35s cubic-bezier(0.2, 0.8, 0.2, 1);
}

.action-btn {
  background: rgba(var(--interactive), 0.06);
  color: rgba(var(--text-primary), 0.6);
  border: 1px solid rgba(var(--border), 0.1);
}

.action-btn:hover {
  background: rgba(var(--interactive), 0.12);
  color: rgba(var(--text-primary), 0.85);
}
</style>
