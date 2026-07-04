<template>
  <div
    v-if="visible"
    class="absolute inset-0 flex items-center justify-center z-[9999] rounded-3xl overflow-hidden"
    style="background: rgba(0, 0, 0, 0.30);"
    @click.self="close"
  >
    <div class="rounded-3xl p-5 w-[320px] max-w-[90vw] shadow-2xl border border-white/10"
      style="background: rgba(var(--bg-panel), 0.95);">
      <h3 class="text-base font-semibold text-white/90 mb-4 text-center">新建提醒</h3>

      <div class="space-y-3">
        <!-- Title (required) -->
        <div>
          <label class="block text-xs text-white/50 mb-1.5">
            标题 <span class="text-red-400">*</span>
          </label>
          <input
            v-model="form.title"
            type="text"
            placeholder="请输入提醒标题"
            maxlength="15"
            class="w-full px-3 py-2 text-sm rounded-xl text-white/90 placeholder-white/25 focus:outline-none transition-all"
            style="background: rgba(var(--interactive), 0.08); border: 1px solid rgba(var(--border), 0.12);"
          />
        </div>

        <!-- Content -->
        <div>
          <label class="block text-xs text-white/50 mb-1.5">内容</label>
          <textarea
            v-model="form.content"
            placeholder="提醒内容（可选）"
            rows="2"
            class="w-full px-3 py-2 text-sm rounded-xl text-white/90 placeholder-white/25 focus:outline-none transition-all resize-none"
            style="background: rgba(var(--interactive), 0.08); border: 1px solid rgba(var(--border), 0.12);"
          />
        </div>

        <!-- Date picker -->
        <div>
          <label class="block text-xs text-white/50 mb-1.5">提醒日期</label>
          <div class="flex items-center gap-1.5">
            <div class="custom-select flex-1">
              <div class="custom-select-trigger" @click="toggleDropdown('year')">
                <span>{{ pickerYear }}年</span>
                <svg class="w-3 h-3 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
              </div>
              <div v-if="openDropdown === 'year'" class="custom-select-dropdown" @wheel.stop>
                <div v-for="y in yearRange" :key="y" class="custom-select-option" :class="{ active: y === pickerYear }" @click="selectValue('year', y)">{{ y }}年</div>
              </div>
            </div>
            <div class="custom-select w-[76px]">
              <div class="custom-select-trigger" @click="toggleDropdown('month')">
                <span>{{ pickerMonth }}月</span>
                <svg class="w-3 h-3 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
              </div>
              <div v-if="openDropdown === 'month'" class="custom-select-dropdown" @wheel.stop>
                <div v-for="m in availableMonths" :key="m" class="custom-select-option" :class="{ active: m === pickerMonth }" @click="selectValue('month', m)">{{ m }}月</div>
              </div>
            </div>
            <div class="custom-select w-[76px]">
              <div class="custom-select-trigger" @click="toggleDropdown('day')">
                <span>{{ pickerDay }}日</span>
                <svg class="w-3 h-3 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
              </div>
              <div v-if="openDropdown === 'day'" class="custom-select-dropdown" @wheel.stop>
                <div v-for="d in availableDays" :key="d" class="custom-select-option" :class="{ active: d === pickerDay }" @click="selectValue('day', d)">{{ d }}日</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Time picker -->
        <div>
          <label class="block text-xs text-white/50 mb-1.5">提醒时间</label>
          <div class="flex items-center justify-center gap-2">
            <input
              type="text"
              v-model="hourDisplay"
              @blur="onHourBlur"
              @keydown.enter="($event.target as HTMLInputElement).blur()"
              @wheel.prevent="onHourWheel"
              class="time-input"
              maxlength="2"
            />
            <span class="text-base font-bold text-white/40 select-none">:</span>
            <input
              type="text"
              v-model="minuteDisplay"
              @blur="onMinuteBlur"
              @keydown.enter="($event.target as HTMLInputElement).blur()"
              @wheel.prevent="onMinuteWheel"
              class="time-input"
              maxlength="2"
            />
          </div>
        </div>
      </div>

      <div class="flex gap-3 mt-5">
        <button
          class="flex-1 py-2.5 rounded-xl bg-white/8 text-white/50 hover:bg-white/15 border border-white/10 transition-all duration-200 active:scale-[0.97] text-sm"
          @click="handleCreate"
        >
          创建
        </button>
        <button
          class="flex-1 py-2.5 rounded-xl bg-white/8 text-white/50 hover:bg-white/15 border border-white/10 transition-all duration-200 active:scale-[0.97] text-sm"
          @click="close"
        >
          取消
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRemindersStore } from "@/stores/reminders";
import { useToast } from "@/composables/useToast";
import { emit as tauriEmit } from "@tauri-apps/api/event";

const props = defineProps<{
  visible: boolean;
  initialTitle?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const remindersStore = useRemindersStore();
const { toast } = useToast();

const form = ref({ title: "", content: "" });

const now = new Date();
const defaultTime = new Date(Date.now() + 60 * 60 * 1000);
const pickerYear = ref(defaultTime.getFullYear());
const pickerMonth = ref(defaultTime.getMonth() + 1);
const pickerDay = ref(defaultTime.getDate());
const pickerHour = ref(defaultTime.getHours());
const pickerMinute = ref(defaultTime.getMinutes());
const hourDisplay = ref(formatTimeValue(defaultTime.getHours()));
const minuteDisplay = ref(formatTimeValue(defaultTime.getMinutes()));

function formatTimeValue(v: number): string {
  return String(v).padStart(2, "0");
}

function onHourBlur() {
  const num = parseInt(hourDisplay.value.replace(/\D/g, ""), 10);
  const clamped = isNaN(num) ? 0 : Math.min(23, Math.max(0, num));
  pickerHour.value = clamped;
  hourDisplay.value = formatTimeValue(clamped);
}

function onMinuteBlur() {
  const num = parseInt(minuteDisplay.value.replace(/\D/g, ""), 10);
  const clamped = isNaN(num) ? 0 : Math.min(59, Math.max(0, num));
  pickerMinute.value = clamped;
  minuteDisplay.value = formatTimeValue(clamped);
}

function onHourWheel(e: WheelEvent) {
  const delta = e.deltaY < 0 ? 1 : -1;
  let next = pickerHour.value + delta;
  if (next > 23) next = 0;
  if (next < 0) next = 23;
  pickerHour.value = next;
  hourDisplay.value = formatTimeValue(next);
}

function onMinuteWheel(e: WheelEvent) {
  const delta = e.deltaY < 0 ? 1 : -1;
  let next = pickerMinute.value + delta;
  if (next > 59) next = 0;
  if (next < 0) next = 59;
  pickerMinute.value = next;
  minuteDisplay.value = formatTimeValue(next);
}
const openDropdown = ref<"" | "year" | "month" | "day">("");

const yearRange = computed(() => {
  const y = now.getFullYear();
  return [y, y + 1, y + 2];
});

const daysInMonth = computed(() => {
  return new Date(pickerYear.value, pickerMonth.value, 0).getDate();
});

const availableMonths = computed(() => {
  if (pickerYear.value > now.getFullYear()) return Array.from({ length: 12 }, (_, i) => i + 1);
  return Array.from({ length: 12 - now.getMonth() }, (_, i) => now.getMonth() + 1 + i);
});

const availableDays = computed(() => {
  const total = daysInMonth.value;
  if (pickerYear.value > now.getFullYear() || pickerMonth.value > now.getMonth() + 1) {
    return Array.from({ length: total }, (_, i) => i + 1);
  }
  if (pickerMonth.value === now.getMonth() + 1) {
    const today = now.getDate();
    return Array.from({ length: total - today + 1 }, (_, i) => today + i);
  }
  return Array.from({ length: total }, (_, i) => i + 1);
});

watch(() => props.visible, (val) => {
  if (val && props.initialTitle) {
    form.value.title = props.initialTitle;
  }
});

watch([pickerYear, pickerMonth], () => {
  if (pickerDay.value > daysInMonth.value) pickerDay.value = daysInMonth.value;
  if (pickerYear.value === now.getFullYear() && pickerMonth.value < now.getMonth() + 1) pickerMonth.value = now.getMonth() + 1;
  if (pickerYear.value === now.getFullYear() && pickerMonth.value === now.getMonth() + 1 && pickerDay.value < now.getDate()) pickerDay.value = now.getDate();
});

function toggleDropdown(name: typeof openDropdown.value) {
  openDropdown.value = openDropdown.value === name ? "" : name;
}

function selectValue(field: string, value: any) {
  if (field === "year") pickerYear.value = value;
  else if (field === "month") pickerMonth.value = value;
  else if (field === "day") pickerDay.value = value;
  openDropdown.value = "";
}

function clampHour() {
  if (pickerHour.value < 0) pickerHour.value = 0;
  if (pickerHour.value > 23) pickerHour.value = 23;
  if (isNaN(pickerHour.value)) pickerHour.value = 0;
}

function clampMinute() {
  if (pickerMinute.value < 0) pickerMinute.value = 0;
  if (pickerMinute.value > 59) pickerMinute.value = 59;
  if (isNaN(pickerMinute.value)) pickerMinute.value = 0;
}

async function handleCreate() {
  if (!form.value.title.trim()) {
    toast("请先输入标题！");
    return;
  }
  clampHour();
  clampMinute();
  const dt = new Date(pickerYear.value, pickerMonth.value - 1, pickerDay.value, pickerHour.value, pickerMinute.value);
  if (dt.getTime() <= Date.now()) {
    toast("提醒时间需大于当前时间！");
    return;
  }
  const remindAt = Math.floor(dt.getTime() / 1000);
  try {
    await remindersStore.createReminder({ title: form.value.title, content: form.value.content, remind_at: remindAt });
    tauriEmit("reminder:updated").catch(() => {});
    toast("新建提醒成功~");
    close();
  } catch (e: any) {
    toast(e?.message || "创建失败");
  }
}

function close() {
  resetForm();
  emit("close");
}

function resetForm() {
  const next = new Date(Date.now() + 60 * 60 * 1000);
  form.value = { title: "", content: "" };
  pickerYear.value = next.getFullYear();
  pickerMonth.value = next.getMonth() + 1;
  pickerDay.value = next.getDate();
  pickerHour.value = next.getHours();
  pickerMinute.value = next.getMinutes();
  hourDisplay.value = formatTimeValue(pickerHour.value);
  minuteDisplay.value = formatTimeValue(pickerMinute.value);
  openDropdown.value = "";
}
</script>

<style scoped>
.custom-select {
  position: relative;
  user-select: none;
}

.custom-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 4px;
  padding: 6px 10px;
  font-size: 15px;
  font-weight: 600;
  color: rgba(var(--text-primary), 0.9);
  background: rgba(var(--interactive), 0.08);
  border: 1.5px solid rgba(var(--border), 0.18);
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
}

.custom-select-trigger:hover {
  background: rgba(var(--interactive), 0.14);
}

.custom-select-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 100%;
  max-height: 160px;
  overflow-y: auto;
  background: rgba(var(--interactive), 0.08);
  backdrop-filter: blur(10px) saturate(140%);
  -webkit-backdrop-filter: blur(10px) saturate(140%);
  border: 1.5px solid rgba(var(--border), 0.18);
  border-radius: 10px;
  padding: 4px;
  z-index: 100;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.custom-select-option {
  padding: 6px 10px;
  font-size: 13px;
  color: rgba(var(--text-primary), 0.85);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.12s;
  white-space: nowrap;
}

.custom-select-option:hover {
  background: rgba(var(--interactive), 0.10);
  color: rgba(var(--text-primary), 0.95);
}

.custom-select-option.active {
  background: rgba(var(--accent), 0.15);
  color: rgba(var(--text-primary), 0.9);
}

.custom-select-dropdown::-webkit-scrollbar {
  width: 4px;
}

.custom-select-dropdown::-webkit-scrollbar-track {
  background: transparent;
}

.custom-select-dropdown::-webkit-scrollbar-thumb {
  background: rgba(var(--scrollbar), 0.1);
  border-radius: 999px;
}

.time-input {
  width: 76px;
  height: 36px;
  text-align: center;
  font-size: 15px;
  font-weight: 600;
  color: rgba(var(--text-primary), 0.9);
  background: rgba(var(--interactive), 0.08);
  border: 1.5px solid rgba(var(--border), 0.18);
  border-radius: 10px;
  font-variant-numeric: tabular-nums;
  outline: none;
  transition: background 0.15s;
  -moz-appearance: textfield;
}
.time-input::-webkit-inner-spin-button,
.time-input::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
.time-input:focus {
  background: rgba(var(--interactive), 0.14);
  border-color: rgba(var(--border), 0.3);
}
</style>
