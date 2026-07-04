<template>
  <div class="tray-menu">
    <button class="menu-item" @click="handleExit">退出</button>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted } from "vue";

let unlisten: (() => void) | null = null;

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  unlisten = await win.onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      win.hide();
    }
  });
});

onUnmounted(() => {
  unlisten?.();
});

async function handleExit() {
  await invoke("exit_app");
}
</script>

<style scoped>
.tray-menu {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: stretch;
  background: rgba(var(--bg-primary), 0.92);
  backdrop-filter: blur(10px) saturate(130%);
  -webkit-backdrop-filter: blur(10px) saturate(130%);
  border: 1px solid rgba(var(--border), 0.1);
  border-radius: 10px;
  overflow: hidden;
}

.menu-item {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: rgba(var(--text-primary), 0.85);
  background: transparent;
  border: none;
  border-radius: inherit;
  cursor: pointer;
  transition: background 0.15s;
}

.menu-item:hover {
  background: rgba(var(--interactive), 0.1);
}
</style>
