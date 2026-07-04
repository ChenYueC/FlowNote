import { ref } from "vue";
import type { Note } from "@/types";

export function useDragDetach() {
  const isDragging = ref(false);
  const dragNote = ref<Note | null>(null);
  const dragPos = ref({ x: 0, y: 0 });

  let ghostEl: HTMLElement | null = null;

  function startDrag(e: PointerEvent, note: Note) {
    isDragging.value = true;
    dragNote.value = note;
    dragPos.value = { x: e.clientX, y: e.clientY };

    // Create ghost element
    ghostEl = document.createElement("div");
    ghostEl.style.cssText = `
      position: fixed;
      pointer-events: none;
      z-index: 9999;
      padding: 12px 16px;
      background: rgba(var(--bg-primary), 0.85);
      backdrop-filter: blur(10px) saturate(130%);
      border: 1px solid rgba(var(--border), 0.15);
      border-radius: 16px;
      font-size: 13px;
      color: rgba(var(--text-primary), 0.9);
      box-shadow: 0 8px 32px rgba(0,0,0,0.4);
      transform: translate(-50%, -50%) scale(1.05);
      transition: transform 0.15s ease;
    `;
    ghostEl.textContent = note.title || "无标题";
    ghostEl.style.left = `${e.clientX}px`;
    ghostEl.style.top = `${e.clientY}px`;
    document.body.appendChild(ghostEl);
  }

  function updateDrag(e: PointerEvent) {
    if (!isDragging.value || !ghostEl) return;
    dragPos.value = { x: e.clientX, y: e.clientY };
    ghostEl.style.left = `${e.clientX}px`;
    ghostEl.style.top = `${e.clientY}px`;
  }

  function endDrag(e: PointerEvent) {
    if (ghostEl) {
      document.body.removeChild(ghostEl);
      ghostEl = null;
    }

    const note = dragNote.value;
    const pos = { x: e.clientX, y: e.clientY };

    isDragging.value = false;
    dragNote.value = null;

    if (note) {
      return { note, x: pos.x, y: pos.y };
    }

    return null;
  }

  return {
    isDragging,
    dragNote,
    dragPos,
    startDrag,
    updateDrag,
    endDrag,
  };
}
