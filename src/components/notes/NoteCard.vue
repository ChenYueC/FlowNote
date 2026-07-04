<template>
  <div
    :data-note-id="note.id"
    class="group relative px-4 py-3 rounded-2xl cursor-pointer transition-colors duration-200 hover:bg-white/10 overflow-hidden"
    :class="{ 'bg-white/8': isActive, 'scale-[0.97] bg-white/15': pressing }"
    @click="onClick"
    @pointerdown="onPointerDown"
    @pointerup="onPointerUp"
    @pointerleave="onPointerUp"
  >
    <div class="absolute left-0 top-0 bottom-0 w-1.5 rounded-l-2xl opacity-0 group-hover:opacity-100 transition-opacity cursor-grab"
      :class="pressing ? 'opacity-100' : ''"
      :style="{ background: pressing ? 'rgba(var(--accent), 0.6)' : 'rgba(var(--interactive), 0.1)' }" />
    <div class="flex items-start justify-between gap-2 ml-1">
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-medium text-white/90 truncate">{{ note.title || "无标题" }}</h3>
        <p class="mt-1 text-xs text-white/50 truncate">{{ previewText }}</p>
      </div>
      <button class="no-drag flex-shrink-0 mt-0.5 transition-opacity"
        :class="note.favorite ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
        :style="note.favorite ? 'color: rgba(var(--favorite))' : 'color: rgba(var(--interactive), 0.3)'"
        @click.stop="handleToggleFavorite">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" /></svg>
      </button>
    </div>
    <div class="mt-2 flex items-center gap-2 text-[10px] text-white/30 ml-1">
      <span>{{ timeAgo }}</span>
      <button class="no-drag opacity-0 group-hover:opacity-100 transition-opacity hover:text-red-400"
        @click.stop="$emit('delete', note.id)">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" /></svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useNotesStore } from "@/stores/notes";
import { useToast } from "@/composables/useToast";
import type { Note } from "@/types";

const props = defineProps<{ note: Note; isActive: boolean }>();
const notesStore = useNotesStore();
const { toast } = useToast();

const emit = defineEmits<{
  open: [];
  toggleFavorite: [id: string];
  delete: [id: string];
  dragStart: [e: PointerEvent, note: Note];
  pointerDown: [noteId: string];
  pointerUp: [noteId: string];
}>();

const pressing = ref(false);
let longPressTimer: ReturnType<typeof setTimeout> | null = null;
let clickTimer: ReturnType<typeof setTimeout> | null = null;

function handleToggleFavorite() {
  notesStore.toggleFavorite(props.note.id);
  toast(props.note.favorite ? "已取消收藏" : "收藏成功");
}

function onClick() {
  if (clickTimer) {
    clearTimeout(clickTimer);
    clickTimer = null;
    emit("open");
  } else {
    clickTimer = setTimeout(() => {
      clickTimer = null;
    }, 400);
  }
}

function onPointerDown(_e: PointerEvent) {
  pressing.value = true;
  emit("pointerDown", props.note.id);
}

function onPointerUp() {
  if (longPressTimer) {
    clearTimeout(longPressTimer);
    longPressTimer = null;
  }
  pressing.value = false;
  emit("pointerUp", props.note.id);
}

const previewText = computed(() => {
  const text = props.note.content
    .replace(/<[^>]+>/g, "")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
    .replace(/[#*`>\-\[\]()!]/g, "")
    .replace(/[\r\n]+/g, " ")
    .trim();
  return text.substring(0, 60) || "空内容";
});

const timeAgo = computed(() => {
  const now = Date.now();
  const diff = now - props.note.updated_at * 1000;
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "刚刚";
  if (mins < 60) return `${mins} 分钟前`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours} 小时前`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days} 天前`;
  return new Date(props.note.updated_at * 1000).toLocaleDateString("zh-CN");
});
</script>
