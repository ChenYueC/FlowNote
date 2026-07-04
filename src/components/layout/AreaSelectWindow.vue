<template>
  <div
    class="fixed inset-0 cursor-crosshair"
    style="background: rgba(0,0,0,0.25);"
    @pointerdown="onDown"
    @pointermove="onMove"
    @pointerup="onUp"
    @keydown.esc="close"
    tabindex="0"
    ref="rootRef"
  >
    <div v-if="rect.w > 0 && rect.h > 0"
      class="absolute border-2 border-blue-400 bg-blue-400/10"
      :style="{ left: rect.x + 'px', top: rect.y + 'px', width: rect.w + 'px', height: rect.h + 'px' }"
    />
    <div v-if="rect.w > 10 && rect.h > 10"
      class="absolute text-xs text-white bg-black/60 px-2 py-0.5 rounded whitespace-nowrap"
      :style="{ left: (rect.x + rect.w / 2 - 30) + 'px', top: (rect.y + rect.h + 8) + 'px' }"
    >
      {{ Math.round(rect.w) }} × {{ Math.round(rect.h) }}
    </div>
    <div class="absolute top-6 left-1/2 -translate-x-1/2 text-xs text-white/50 select-none pointer-events-none">
      拖拽选取区域 · ESC 取消
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit as tauriEmit } from "@tauri-apps/api/event";

const rootRef = ref<HTMLElement>();
const rect = reactive({ x: 0, y: 0, w: 0, h: 0 });
let startPos = { x: 0, y: 0 };
let done = false;
let dragging = false;

onMounted(() => {
  rootRef.value?.focus();
});

function onDown(e: PointerEvent) {
  if (done) return;
  dragging = true;
  startPos = { x: e.clientX, y: e.clientY };
  rect.x = e.clientX;
  rect.y = e.clientY;
  rect.w = 0;
  rect.h = 0;
}

function onMove(e: PointerEvent) {
  if (done || !dragging) return;
  rect.x = Math.min(startPos.x, e.clientX);
  rect.y = Math.min(startPos.y, e.clientY);
  rect.w = Math.abs(e.clientX - startPos.x);
  rect.h = Math.abs(e.clientY - startPos.y);
}

function onUp() {
  if (done || !dragging) return;
  dragging = false;
  done = true;
  finish();
}

function close() {
  if (done) return;
  done = true;
  finish();
}

async function finish() {
  if (rect.w > 50 && rect.h > 50) {
    await tauriEmit("area:selected", { x: rect.x, y: rect.y, width: Math.round(rect.w), height: Math.round(rect.h) });
  }
  const win = getCurrentWebviewWindow();
  setTimeout(() => { win.destroy(); }, 50);
}
</script>
