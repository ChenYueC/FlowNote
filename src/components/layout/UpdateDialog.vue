<template>
  <div class="h-full flex flex-col items-center justify-center rounded-3xl overflow-hidden"
    style="background: rgba(var(--bg-primary), 0.92); backdrop-filter: blur(10px) saturate(130%);">
    <!-- Icon -->
    <div class="w-12 h-12 rounded-2xl flex items-center justify-center mb-4"
      style="background: rgba(var(--accent), 0.15);">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="rgb(var(--accent))" stroke-width="2"
        stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
        <polyline points="7 10 12 15 17 10" />
        <line x1="12" y1="15" x2="12" y2="3" />
      </svg>
    </div>

    <!-- Title -->
    <h2 class="text-lg font-bold text-white/90 mb-1">发现新版本</h2>
    <p class="text-xs text-white/40 mb-6">v{{ version }}</p>

    <!-- Progress or buttons -->
    <div v-if="downloading" class="w-56">
      <!-- Progress bar -->
      <div class="w-full h-2 rounded-full overflow-hidden mb-2"
        style="background: rgba(255,255,255,0.1);">
        <div class="h-full rounded-full transition-all duration-100"
          :style="{ width: progress + '%', background: 'rgb(var(--accent))' }">
        </div>
      </div>
      <p class="text-xs text-white/40 text-center">正在下载… {{ progress }}%</p>
    </div>

    <div v-else class="flex gap-3">
      <button
        class="no-drag px-5 py-2 rounded-full text-xs text-white/50 hover:text-white/70 hover:bg-white/8 transition-all"
        @click="handleCancel"
      >
        稍后提醒
      </button>
      <button
        class="no-drag px-5 py-2 rounded-full text-xs text-white/90 transition-all"
        style="background: rgba(var(--accent), 0.25);"
        @click="handleConfirm"
      >
        立即更新
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";

const params = new URLSearchParams(window.location.search);
const version = params.get("version") || "";
const downloadUrl = params.get("url") || "";

const downloading = ref(false);
const progress = ref(0);

let unlistenProgress: UnlistenFn | null = null;

onMounted(async () => {
  // Listen for download progress events
  unlistenProgress = await listen<{ percent: number }>("update-download-progress", (event) => {
    progress.value = event.payload.percent;
  });
});

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
});

async function handleConfirm() {
  downloading.value = true;
  try {
    await invoke("download_and_install", { url: downloadUrl });
  } catch (e) {
    downloading.value = false;
  }
}

function handleCancel() {
  // Save dismiss timestamp
  localStorage.setItem("update-dismissed-at", String(Date.now()));
  getCurrentWebviewWindow().close();
}
</script>
