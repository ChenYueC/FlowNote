<template>
  <div class="relative">
    <svg
      class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
    <input
      v-model="query"
      type="text"
      placeholder="搜索笔记..."
      class="no-drag w-full pl-9 pr-4 py-2.5 rounded-2xl text-sm outline-none transition-all duration-200 bg-white/5 text-white/90 placeholder:text-white/25 focus:bg-white/10"
      @input="onInput"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onUnmounted } from "vue";

const query = ref("");
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const emit = defineEmits<{
  search: [query: string];
}>();

function onInput() {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    emit("search", query.value);
  }, 300);
}

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
});
</script>
