<template>
  <div class="drag-region flex items-center justify-between px-4 pt-3 pb-1">
    <div class="flex items-center gap-2">
      <IconButton tabindex="-1" @click="$emit('toggleMode')">
        <svg v-if="isPreview" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
          <path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </IconButton>
      <div class="w-px h-5 bg-white/10" />
      <IconButton tabindex="-1" @click="$emit('screenshot')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="3" width="20" height="16" rx="2" />
          <circle cx="12" cy="12" r="3" />
          <path d="M2 9h3l2-3h10l2 3h3" />
        </svg>
      </IconButton>
      <IconButton tabindex="-1" @click="$emit('setReminder')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" />
          <path d="M13.73 21a2 2 0 01-3.46 0" />
        </svg>
      </IconButton>
    </div>
    <span class="text-xs text-white/40">
      {{ isPreview ? "预览模式" : "编辑模式" }}
    </span>
    <div class="flex items-center gap-0.5">
      <button
        tabindex="-1"
        class="no-drag flex items-center justify-center w-6 h-6 rounded-full transition-all duration-200 hover:bg-white/8"
        :class="pinned ? 'text-red-400' : 'text-white/30 hover:text-white/60'"
        @click="$emit('pin')"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="17" x2="12" y2="22" />
          <path d="M5 17h14v-1.76a2 2 0 00-1.11-1.79l-1.78-.9A2 2 0 0115 10.76V6h1a2 2 0 000-4H8a2 2 0 000 4h1v4.76a2 2 0 01-1.11 1.79l-1.78.9A2 2 0 005 15.24V17z" />
        </svg>
      </button>
      <button
        tabindex="-1"
        class="no-drag flex items-center justify-center w-6 h-6 rounded-full text-white/30 hover:text-white/60 hover:bg-white/8 transition-all"
        @click="$emit('minimize')"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
      <button
        tabindex="-1"
        class="no-drag flex items-center justify-center w-6 h-6 rounded-full text-white/30 hover:text-red-400 hover:bg-red-400/10 transition-all"
        @click="$emit('close')"
      >
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import IconButton from "@/components/ui/IconButton.vue";

defineProps<{
  isPreview: boolean;
  pinned: boolean;
}>();

defineEmits<{
  toggleMode: [];
  screenshot: [];
  setReminder: [];
  pin: [];
  minimize: [];
  close: [];
}>();
</script>
