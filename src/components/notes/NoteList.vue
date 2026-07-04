<template>
  <div class="flex-1 overflow-y-auto px-2">
    <NoteCard
      v-for="note in notes"
      :key="note.id"
      :note="note"
      :isActive="activeId === note.id"
      @open="$emit('openNote', note)"
      @toggleFavorite="(id: string) => $emit('toggleFavorite', id)"
      @delete="(id: string) => $emit('deleteNote', id)"
      @dragStart="(e: PointerEvent, n: Note) => $emit('dragStart', e, n)"
    />
    <div v-if="notes.length === 0" class="flex flex-col items-center justify-center py-12 text-white/30 text-sm">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="mb-3">
        <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" />
      </svg><span>还没有笔记</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Note } from "@/types";
import NoteCard from "./NoteCard.vue";

defineProps<{ notes: Note[]; activeId: string | null }>();

defineEmits<{
  openNote: [note: Note]; toggleFavorite: [id: string]; deleteNote: [id: string];
  dragStart: [e: PointerEvent, note: Note];
}>();
</script>
