<template>
  <div
    class="h-full flex flex-col rounded-3xl overflow-hidden transition-all duration-300"
    :style="{
      background: 'rgba(var(--bg-primary), 0.78)',
      backdropFilter: 'blur(10px)',
      opacity: opacity,
      border: '1px solid rgba(var(--border), 0.1)',
    }"
    data-window="floating"
  >
    <!-- Normal content -->
    <template v-if="!autoHide.isHidden.value">
      <!-- Title bar -->
      <div class="drag-region flex items-center justify-between px-3 pt-3 pb-1.5">
        <span class="text-xs font-medium text-white/60 truncate ml-1">
          {{ title }}
        </span>
        <div class="flex items-center gap-0.5">
          <!-- Auto-hide -->
          <button
            class="no-drag flex items-center justify-center w-6 h-6 rounded-full transition-all duration-200"
            :class="autoHide.autoHideEnabled.value ? 'text-blue-400 bg-white/10' : 'text-white/30 hover:text-white/60 hover:bg-white/8'"
            @click="autoHide.autoHideEnabled.value = !autoHide.autoHideEnabled.value"
            title="自动隐藏"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4" />
              <polyline points="16 17 21 12 16 7" />
              <line x1="21" y1="12" x2="9" y2="12" />
            </svg>
          </button>
          <!-- Snap -->
          <button
            class="no-drag flex items-center justify-center w-6 h-6 rounded-full transition-all duration-200"
            :class="autoHide.snapEnabled.value ? 'text-green-400 bg-white/10' : 'text-white/30 hover:text-white/60 hover:bg-white/8'"
            @click="autoHide.snapEnabled.value = !autoHide.snapEnabled.value"
            title="边缘吸附"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20 10V7a2 2 0 00-2-2H6a2 2 0 00-2 2v3" />
              <rect x="3" y="12" width="18" height="8" rx="2" />
            </svg>
          </button>
          <!-- Pin -->
          <button
            class="no-drag flex items-center justify-center w-6 h-6 rounded-full transition-all duration-200"
            :class="pinned ? 'text-yellow-400 bg-white/10' : 'text-white/30 hover:text-white/60 hover:bg-white/8'"
            @click="togglePin"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
              <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z" />
            </svg>
          </button>
          <!-- Close -->
          <button
            class="no-drag flex items-center justify-center w-6 h-6 rounded-full text-white/30 hover:text-red-400 hover:bg-red-400/10 transition-all duration-200"
            @click="handleClose"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto px-4 py-2 text-sm text-white/80 leading-relaxed whitespace-pre-wrap">
        {{ content }}
      </div>

      <!-- Bottom opacity slider -->
      <div class="px-4 pb-3 flex items-center gap-2">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-white/25 flex-shrink-0">
          <circle cx="12" cy="12" r="10" />
          <path d="M12 6v12M6 12h12" />
        </svg>
        <input
          type="range"
          min="0.3"
          max="1"
          step="0.05"
          :value="opacity"
          class="no-drag w-full h-1 appearance-none bg-white/10 rounded-full outline-none [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-white/60"
          @input="setOpacity"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useNotesStore } from "@/stores/notes";
import { useWindowsStore } from "@/stores/windows";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useAutoHide } from "@/composables/useAutoHide";

const props = defineProps<{
  noteId: string;
  windowId: string;
}>();

const notesStore = useNotesStore();
const windowsStore = useWindowsStore();
const autoHide = useAutoHide(props.windowId, '[data-window="floating"]');

const title = ref("");
const content = ref("");
const pinned = ref(true);
const opacity = ref(1);

async function togglePin() {
  pinned.value = !pinned.value;
  await windowsStore.updateWindowState({
    id: props.windowId,
    pinned: pinned.value ? 1 : 0,
  });

  const window = getCurrentWebviewWindow();
  await window.setAlwaysOnTop(pinned.value);
}

async function setOpacity(e: Event) {
  const val = parseFloat((e.target as HTMLInputElement).value);
  opacity.value = val;
  await windowsStore.updateWindowState({
    id: props.windowId,
    opacity: val,
  });
}

async function handleClose() {
  await windowsStore.closeFloating(props.windowId);
}

onMounted(async () => {
  const note = await notesStore.getNote(props.noteId);
  if (note) {
    title.value = note.title;
    content.value = note.content;
  }
});
</script>
