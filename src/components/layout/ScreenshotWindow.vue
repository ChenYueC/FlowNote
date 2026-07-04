<template>
  <div
    class="fixed inset-0 select-none overflow-hidden"
    style="background: rgba(0,0,0,0.25); cursor: crosshair;"
    @pointerdown="onDown"
    @pointermove="onMove"
    @pointerup="onUp"
  >
    <!-- Selection rectangle -->
    <div
      v-if="selecting && selection.width > 0 && selection.height > 0"
      class="absolute border-2 border-blue-400/80 bg-blue-400/10"
      :style="{
        left: selection.x + 'px',
        top: selection.y + 'px',
        width: selection.width + 'px',
        height: selection.height + 'px',
      }"
    >
      <!-- Size indicator -->
      <div
        class="absolute px-2 py-0.5 rounded-full text-xs font-mono text-white whitespace-nowrap bg-black/50"
        :style="sizeLabelStyle"
      >
        {{ Math.round(selection.width) }} × {{ Math.round(selection.height) }}
      </div>
    </div>

    <!-- Hint text -->
    <div
      v-if="!selecting"
      class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-white/50 text-sm pointer-events-none"
    >
      拖拽选择截图区域 · ESC 取消
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from "vue";
import { emit as tauriEmit } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const props = defineProps<{
  noteId: string;
}>();

const emit = defineEmits<{
  close: [];
  captured: [path: string];
}>();
void emit;

const selecting = ref(false);
const startPos = reactive({ x: 0, y: 0 });
const selection = reactive({ x: 0, y: 0, width: 0, height: 0 });
// Offset: viewport origin → screen origin (logical pixels)
const viewportOffset = reactive({ x: 0, y: 0 });

const sizeLabelStyle = computed(() => {
  const showBelow = selection.y < 36;
  return showBelow
    ? { top: "calc(100% + 4px)", left: "4px" }
    : { bottom: "calc(100% + 4px)", left: "4px" };
});

function onDown(e: PointerEvent) {
  selecting.value = true;
  startPos.x = e.clientX;
  startPos.y = e.clientY;
  selection.x = e.clientX;
  selection.y = e.clientY;
  selection.width = 0;
  selection.height = 0;
}

function onMove(e: PointerEvent) {
  if (!selecting.value) return;
  selection.x = Math.min(startPos.x, e.clientX);
  selection.y = Math.min(startPos.y, e.clientY);
  selection.width = Math.abs(e.clientX - startPos.x);
  selection.height = Math.abs(e.clientY - startPos.y);
}

async function onUp() {
  if (!selecting.value) return;
  selecting.value = false;

  if (selection.width < 10 || selection.height < 10) return;

  // Convert viewport coords → screen coords (logical pixels)
  const screenX = Math.round(selection.x + viewportOffset.x);
  const screenY = Math.round(selection.y + viewportOffset.y);
  const screenW = Math.round(selection.width);
  const screenH = Math.round(selection.height);

  doCapture(screenX, screenY, screenW, screenH);
}

function doCapture(x: number, y: number, w: number, h: number) {
  // Emit screen coords to EditorWindow, then close
  tauriEmit("screenshot:do-capture", {
    noteId: props.noteId,
    x, y, width: w, height: h,
  });
  close();
}

function close() {
  const win = getCurrentWebviewWindow();
  win.hide();
  win.destroy();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") close();
}

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  try {
    // Read the logical viewport offset passed by EditorWindow via URL params.
    // This avoids any DPI ambiguity from outerPosition().
    const params = new URLSearchParams(window.location.search);
    viewportOffset.x = Number(params.get("ox")) || 0;
    viewportOffset.y = Number(params.get("oy")) || 0;

    await win.setAlwaysOnTop(true);
    await win.setFocus();
  } catch (e) {
    console.error("ScreenshotWindow setup failed:", e);
  }
  document.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  document.removeEventListener("keydown", onKeyDown);
});
</script>
