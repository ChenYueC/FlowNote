<template>
  <div
    class="reminder-card group relative rounded-xl p-3"
    style="background: rgba(var(--interactive), 0.06); border: 1px solid rgba(var(--border), 0.08);"
    :class="{
      'overdue': isOverdue && !reminder.completed && !reminder.cancelled,
      'completed': reminder.completed,
      'cancelled': reminder.cancelled,
      'hover:bg-white/8': isActive,
      'z-50': showSnoozeOptions || showUnitDropdown,
    }"
    :style="isDimmed ? 'background: rgba(255,255,255,0.03)' : ''"
  >
    <!-- Delete button (only for completed/cancelled/overdue) -->
    <button
      v-if="isDimmed"
      class="no-drag absolute top-1.5 right-1.5 w-5 h-5 flex items-center justify-center rounded-full text-white/30 hover:text-white/70 hover:bg-white/10 opacity-0 group-hover:opacity-100 transition-[opacity,color,background-color] z-10"
      @click.stop="$emit('delete', reminder.id)"
    >
      <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12" />
      </svg>
    </button>

    <!-- Main content -->
    <div class="flex items-center justify-between gap-3">
      <div class="flex-1 min-w-0">
        <!-- First line: Title -->
        <div class="flex items-center gap-2 mb-1">
          <span class="text-sm font-medium truncate" :class="isDimmed ? 'text-white/50' : 'text-white/90'">
            {{ reminder.title }}
          </span>
        </div>

        <!-- Content -->
        <div class="text-[11px] mb-1" :class="[isDimmed ? 'text-white/25' : 'text-white/50', reminder.content ? 'line-clamp-2' : '']">
          {{ reminder.content || '还未填写任何内容~' }}
        </div>

        <!-- Second line: Date/Time -->
        <div class="flex items-center gap-1.5">
          <span class="text-[10px] text-white/40">
            {{ formatTime(reminder.remind_at) }}
          </span>
        </div>
      </div>

      <!-- Right side status -->
      <div class="flex-shrink-0 flex items-center">
        <!-- Completed -->
        <div v-if="reminder.completed" class="flex items-center gap-1 text-green-400">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-[11px]">已完成</span>
        </div>

        <!-- Cancelled -->
        <div v-else-if="reminder.cancelled" class="flex items-center gap-1 text-white/40">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 6L6 18M6 6l12 12" />
          </svg>
          <span class="text-[11px]">已取消</span>
        </div>

        <!-- Overdue -->
        <div v-else-if="isOverdue" class="flex items-center gap-1 text-red-400">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-[11px]">已过期</span>
        </div>

        <!-- Active: Cancel + Snooze (expand on hover) -->
        <div v-else class="flex items-center gap-1.5 whitespace-nowrap overflow-hidden transition-all duration-200 max-w-0 group-hover:max-w-[200px]">
          <button
            class="no-drag flex items-center gap-1 px-2 py-1 text-[10px] rounded-md bg-white/15 text-white/80 hover:bg-white/20 transition-colors active:scale-95 border-0"
            @click.stop="$emit('cancel', reminder.id)"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 6L6 18M6 6l12 12" />
            </svg>
            <span>取消</span>
          </button>
          <button
            class="no-drag flex items-center gap-1 px-2 py-1 text-[10px] rounded-md bg-white/15 text-white/80 hover:bg-white/20 transition-colors active:scale-95 border-0"
            @click.stop="showSnoozeOptions = !showSnoozeOptions"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>延后</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Snooze options -->
    <div
      v-if="showSnoozeOptions && !reminder.completed"
      class="mt-3 pt-3 border-t border-white/10"
    >
      <!-- Presets -->
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="minutes in snoozeOptions"
          :key="minutes"
          class="no-drag px-2.5 py-1 text-[10px] rounded-md bg-white/8 text-white/50 hover:bg-white/15 hover:text-white/80 transition-colors active:scale-95 border-0"
          @click.stop="handleSnooze(minutes)"
        >
          {{ minutes }}分钟
        </button>
        <button
          class="no-drag px-2.5 py-1 text-[10px] rounded-md bg-white/8 text-white/50 hover:bg-white/15 hover:text-white/80 transition-colors active:scale-95 border-0"
          @click.stop="showCustomSnooze = !showCustomSnooze"
        >
          自定义
        </button>
      </div>
      <!-- Custom input -->
      <div v-if="showCustomSnooze" class="flex items-center justify-center gap-2 mt-2">
        <input
          v-model.number="customValue"
          type="number"
          min="1"
          step="1"
          placeholder="数值"
          class="no-drag w-16 px-2 py-1 text-[11px] rounded-lg text-white/90 placeholder-white/25 focus:outline-none"
          style="background: rgba(var(--interactive), 0.10); border: 1px solid rgba(var(--border), 0.18);"
          @keyup.enter="handleCustomSnooze"
        />
        <div class="custom-select" style="position: relative;">
          <div
            class="no-drag flex items-center justify-between gap-1 px-2 py-1 text-[11px] rounded-lg text-white/90 cursor-pointer"
            style="background: rgba(255,255,255,0.10); border: 1px solid rgba(255,255,255,0.18); min-width: 48px;"
            @click.stop="showUnitDropdown = !showUnitDropdown"
          >
            <span>{{ unitLabels[customUnit] }}</span>
            <svg class="w-2.5 h-2.5 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
          </div>
          <div
            v-if="showUnitDropdown"
            class="custom-select-dropdown"
            style="min-width: 56px; left: 0; top: calc(100% + 2px);"
          >
            <div
              v-for="u in unitOptions" :key="u"
              class="custom-select-option"
              :class="{ active: u === customUnit }"
              @click.stop="customUnit = u; showUnitDropdown = false"
            >{{ unitLabels[u] }}</div>
          </div>
        </div>
        <button
          class="no-drag px-3 py-1 text-[10px] rounded-lg text-white/60 hover:text-white/80 transition-colors active:scale-95 border-0"
          style="background: rgba(var(--interactive), 0.10); border: 1px solid rgba(var(--border), 0.18);"
          @click.stop="handleCustomSnooze"
        >
          确定
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { Reminder } from "@/types";

const props = defineProps<{
  reminder: Reminder;
  noteTitle?: string;
}>();

const emit = defineEmits<{
  complete: [id: string];
  snooze: [id: string, minutes: number];
  openNote: [noteId: string];
  delete: [id: string];
  cancel: [id: string];
}>();

const showSnoozeOptions = ref(false);
const showCustomSnooze = ref(false);
const showUnitDropdown = ref(false);
const customValue = ref(10);
const customUnit = ref<"minute" | "hour" | "day">("minute");
const unitOptions: Array<"minute" | "hour" | "day"> = ["minute", "hour", "day"];
const unitLabels: Record<string, string> = { minute: "分钟", hour: "小时", day: "天" };

const snoozeOptions = [5, 10, 15, 30];

const isOverdue = computed(() => {
  if (props.reminder.completed || props.reminder.cancelled) return false;
  // 超过1分5秒才算过期
  return Date.now() > props.reminder.remind_at * 1000 + 65000;
});

const isDimmed = computed(() => {
  return props.reminder.completed || props.reminder.cancelled || isOverdue.value;
});

const isActive = computed(() => {
  return !isDimmed.value;
});

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const isToday = date.toDateString() === now.toDateString();
  const isTomorrow = new Date(now.getTime() + 24 * 60 * 60 * 1000).toDateString() === date.toDateString();

  const timeStr = date.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
  });

  if (isToday) return `今天 ${timeStr}`;
  if (isTomorrow) return `明天 ${timeStr}`;

  // Check if same year
  if (date.getFullYear() === now.getFullYear()) {
    return `${date.getMonth() + 1}月${date.getDate()}日 ${timeStr}`;
  }

  return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日 ${timeStr}`;
}

function handleSnooze(minutes: number) {
  emit('snooze', props.reminder.id, minutes);
  showSnoozeOptions.value = false;
}

function handleCustomSnooze() {
  const val = Math.floor(customValue.value);
  if (val <= 0) return;
  let minutes = val;
  if (customUnit.value === "hour") minutes = val * 60;
  else if (customUnit.value === "day") minutes = val * 1440;
  emit('snooze', props.reminder.id, minutes);
  showCustomSnooze.value = false;
  showSnoozeOptions.value = false;
}
</script>

<style scoped>
.reminder-card {
  position: relative;
  transition: opacity 0.2s ease, background-color 0.2s ease, border-color 0.2s ease;
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.reminder-card:hover {
  background: rgba(var(--interactive), 0.08);
}

.reminder-card.completed {
  opacity: 0.65;
}

.reminder-card.completed:hover {
  opacity: 0.85;
}

/* Remove button borders */
button {
  border: none;
  outline: none;
}

/* Hide number input spinners */
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
input[type="number"] {
  -moz-appearance: textfield;
}

/* Dropdown styles (matching ReminderView) */
.custom-select {
  position: relative;
  user-select: none;
}

.custom-select-dropdown {
  position: absolute;
  min-width: 100%;
  overflow-y: auto;
  background: rgba(var(--interactive), 0.10);
  backdrop-filter: blur(10px) saturate(140%);
  -webkit-backdrop-filter: blur(10px) saturate(140%);
  border: 1.5px solid rgba(var(--border), 0.18);
  border-radius: 10px;
  padding: 4px;
  z-index: 100;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.custom-select-option {
  padding: 5px 8px;
  font-size: 11px;
  color: rgba(var(--text-primary), 0.85);
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
  white-space: nowrap;
  text-align: center;
}

.custom-select-option:hover {
  background: rgba(var(--interactive), 0.10);
  color: rgba(var(--text-primary), 0.95);
}

.custom-select-option.active {
  background: rgba(var(--accent), 0.15);
  color: rgba(var(--text-primary), 0.9);
}
</style>
