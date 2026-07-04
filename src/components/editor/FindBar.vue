<template>
  <div class="find-bar" :class="{ 'has-replace': showReplace }">
    <div class="find-row">
      <button tabindex="-1" class="find-btn find-btn-toggle" :class="{ expanded: showReplace }" title="展开替换" @mousedown.prevent="showReplace = !showReplace">
        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>
      </button>
      <input
        ref="inputRef"
        v-model="query"
        type="text"
        class="find-input"
        placeholder="查找..."
        spellcheck="false"
        @keydown.enter.prevent="onEnter"
        @keydown.escape.prevent="$emit('close')"
        @input="onInput"
      />
      <span class="find-count">{{ countText }}</span>
      <button tabindex="-1" class="find-btn" title="上一个 (Shift+Enter)" @mousedown.prevent="emit('prev')">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M18 15l-6-6-6 6"/></svg>
      </button>
      <button tabindex="-1" class="find-btn" title="下一个 (Enter)" @mousedown.prevent="emit('next')">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>
      </button>
      <button tabindex="-1" class="find-btn find-btn-close" title="关闭 (Esc)" @mousedown.prevent="emit('close')">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
    <div v-if="showReplace" class="find-row replace-row">
      <div class="find-btn-toggle-placeholder"></div>
      <input
        v-model="replaceText"
        type="text"
        class="find-input"
        placeholder="替换..."
        spellcheck="false"
        @keydown.escape.prevent="$emit('close')"
      />
      <button tabindex="-1" class="find-btn find-btn-replace" title="替换" @mousedown.prevent="emit('replace', replaceText)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M3 12h18"/><path d="M3 18h18"/><path d="M17 4l2 2-2 2"/></svg>
      </button>
      <button tabindex="-1" class="find-btn find-btn-replace" title="全部替换" @mousedown.prevent="emit('replaceAll', replaceText)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M3 12h18"/><path d="M3 18h18"/><path d="M15 4l4 4-4 4"/><path d="M15 12l4 4-4 4"/></svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed, onUnmounted } from "vue";

const props = defineProps<{ matchCount?: number; currentMatch?: number }>();
const emit = defineEmits<{ search: [query: string]; next: []; prev: []; close: []; replace: [text: string]; replaceAll: [text: string] }>();

const inputRef = ref<HTMLInputElement>();
const query = ref("");
const replaceText = ref("");
const showReplace = ref(false);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const countText = computed(() => {
  if (!query.value) return "0/0";
  if (!props.matchCount) return "无结果";
  return `${(props.currentMatch ?? 0) + 1}/${props.matchCount}`;
});

function onInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => emit("search", query.value), 150);
}

function onEnter(e: KeyboardEvent) {
  if (e.shiftKey) emit("prev");
  else emit("next");
}

function focus() {
  nextTick(() => inputRef.value?.focus());
}

function setQuery(text: string) {
  query.value = text;
}

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});

defineExpose({ focus, query, setQuery });
</script>

<style scoped>
.find-bar {
  position: absolute;
  top: 8px;
  right: 18px;
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 4px 6px;
  border-radius: 8px;
  background: rgba(var(--bg-panel), 0.95);
  backdrop-filter: blur(10px) saturate(130%);
  -webkit-backdrop-filter: blur(10px) saturate(130%);
  border: 1px solid rgba(var(--border), 0.12);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  animation: find-bar-in 0.15s ease-out;
  min-width: 320px;
}

.find-row {
  display: flex;
  align-items: center;
  gap: 4px;
}

@keyframes find-bar-in {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.find-input {
  width: 160px;
  padding: 4px 8px;
  border-radius: 5px;
  border: 1px solid rgba(var(--border), 0.15);
  background: rgba(var(--bg-primary), 0.5);
  color: #E6E6E6;
  font-size: 13px;
  font-family: 'JetBrains Mono', 'PingFang SC', 'Microsoft YaHei', sans-serif;
  outline: none;
}

.find-input::placeholder {
  color: rgba(255, 255, 255, 0.25);
}

.find-input:focus {
  border-color: rgba(var(--accent), 0.4);
}

.find-count {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  white-space: nowrap;
  min-width: 32px;
  text-align: center;
  user-select: none;
}

.find-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 5px;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.45);
  cursor: pointer;
  transition: all 0.15s;
}

.find-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
}

.find-btn-close:hover {
  background: rgba(255, 80, 80, 0.2);
  color: rgba(255, 120, 120, 0.9);
}

.find-btn-toggle {
  width: 20px;
  height: 20px;
  transition: transform 0.2s;
}

.find-btn-toggle.expanded {
  transform: rotate(180deg);
}

.find-btn-toggle-placeholder {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.find-btn-replace:hover {
  background: rgba(var(--accent), 0.2);
  color: rgba(var(--accent), 0.9);
}
</style>
