<template>
  <div class="dock-wrapper" :class="edge">
    <div class="dock-handle" :class="[edge, { fading }]" @mouseenter="handleMouseEnter" @mouseleave="handleMouseLeave" @click="handleClick">
      <svg
        class="dock-icon"
        width="16" height="16" viewBox="0 0 24 24"
        fill="none" stroke="white" stroke-width="2"
        stroke-linecap="round" stroke-linejoin="round"
      >
        <path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/>
        <path d="m15 5 4 4"/>
      </svg>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { cursorPosition } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";

const props = defineProps<{
  edge: "left" | "right" | "top";
  dockId: string;
}>();

let pollInterval: ReturnType<typeof setInterval> | null = null;
let lastShowEmit = 0;
let ignoring = false;
const fading = ref(false);
let unlistenFade: UnlistenFn | null = null;
let stateChangeTimer: ReturnType<typeof setTimeout> | null = null;

function handleMouseEnter() {
  if (!ignoring) return;
  if (Date.now() - lastShowEmit < 1000) return;
  emit(`editor-dock:show-${props.dockId}`);
  lastShowEmit = Date.now();
}

function handleMouseLeave() {
  // No-op: show is now immediate via polling
}

async function handleClick() {
  await emit(`editor-dock:show-${props.dockId}`);
  lastShowEmit = Date.now();
}

onMounted(async () => {
  const win = getCurrentWebviewWindow();

  try {
    await win.setIgnoreCursorEvents(true);
    ignoring = true;
  } catch {}

  unlistenFade = await listen(`editor-dock:fade-out-${props.dockId}`, () => {
    fading.value = true;
  });

  // Poll cursor position to toggle OS-level click-through
  pollInterval = setInterval(async () => {
    try {
      const cursor = await cursorPosition();
      const outerPos = await win.outerPosition();
      const outerSize = await win.outerSize();
      const innerSize = await win.innerSize();

      const borderLR = Math.max(0, Math.round((outerSize.width - innerSize.width) / 2));
      const borderTB = Math.max(0, Math.round((outerSize.height - innerSize.height) / 2));

      const innerLeft = outerPos.x + borderLR;
      const innerRight = innerLeft + innerSize.width;
      const innerTop = outerPos.y + borderTB;
      const innerBottom = innerTop + innerSize.height;

      const HANDLE_W = 40;
      const HANDLE_H = 36;
      let handleLeft = innerLeft, handleRight = innerRight;
      let handleTop = innerTop, handleBottom = innerBottom;

      if (props.edge === "right") {
        handleLeft = innerRight - HANDLE_W;
      } else if (props.edge === "left") {
        handleRight = innerLeft + HANDLE_W;
      } else if (props.edge === "top") {
        handleBottom = innerTop + HANDLE_H;
      }

      const cx = cursor.x;
      const cy = cursor.y;
      const inHandle =
        cx >= handleLeft && cx <= handleRight &&
        cy >= handleTop && cy <= handleBottom;

      const margin = ignoring ? 15 : 0;
      const nearHandle =
        cx >= handleLeft - margin && cx <= handleRight + margin &&
        cy >= handleTop - margin && cy <= handleBottom + margin;

      if (!ignoring && !nearHandle) {
        // Debounce exit: delay 20ms before enabling click-through
        if (stateChangeTimer) { clearTimeout(stateChangeTimer); stateChangeTimer = null; }
        stateChangeTimer = setTimeout(async () => {
          stateChangeTimer = null;
          if (!ignoring) return;
          ignoring = true;
          await win.setIgnoreCursorEvents(true);
        }, 20);
      } else if (ignoring && inHandle) {
        // Debounce enter: delay 20ms before disabling click-through
        if (stateChangeTimer) { clearTimeout(stateChangeTimer); stateChangeTimer = null; }
        stateChangeTimer = setTimeout(async () => {
          stateChangeTimer = null;
          if (!ignoring) return;
          ignoring = false;
          await win.setIgnoreCursorEvents(false);

          // Trigger dock show immediately
          if (Date.now() - lastShowEmit > 1000) {
            await emit(`editor-dock:show-${props.dockId}`);
            lastShowEmit = Date.now();
          }
        }, 20);
      }
    } catch {}
  }, 130);
});

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval);
  if (stateChangeTimer) clearTimeout(stateChangeTimer);
  if (unlistenFade) unlistenFade();
});
</script>

<style scoped>
.dock-wrapper {
  width: 100vw;
  height: 100vh;
  display: flex;
  background: transparent;
  user-select: none;
  -webkit-user-select: none;
  pointer-events: none;
}

.dock-wrapper.left  { justify-content: flex-start; align-items: center; }
.dock-wrapper.right { justify-content: flex-end; align-items: center; }
.dock-wrapper.top   { justify-content: center; align-items: flex-start; }

.dock-handle {
  display: flex;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  pointer-events: auto;
  background: rgba(var(--bg-primary), 0.75);
  backdrop-filter: blur(10px) saturate(130%);
  -webkit-backdrop-filter: blur(10px) saturate(130%);
  border: 1px solid rgba(var(--border), 0.15);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.dock-handle.left {
  width: 26px;
  height: 72px;
  border-radius: 0 12px 12px 0;
  border-left: none;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.2);
}

.dock-handle.right {
  width: 26px;
  height: 72px;
  border-radius: 12px 0 0 12px;
  border-right: none;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.2);
}

.dock-handle.top {
  width: 72px;
  height: 24px;
  border-radius: 0 0 12px 12px;
  border-top: none;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.dock-handle:hover {
  background: rgba(var(--interactive), 0.12);
  border-color: rgba(var(--border), 0.25);
}

.dock-handle.fading {
  opacity: 0;
  transition: opacity 150ms ease-out;
}

.dock-handle.left:hover  { transform: translateX(2px); }
.dock-handle.right:hover { transform: translateX(-2px); }
.dock-handle.top:hover   { transform: translateY(2px); }
</style>
