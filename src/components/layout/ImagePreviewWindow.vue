<template>
  <div
    class="h-screen w-screen flex items-center justify-center overflow-hidden select-none"
  style="background: rgba(0, 0, 0, 0.8);"
    @mousedown.self="startDrag"
    @wheel.prevent="onZoom"
    ref="containerRef"
  >
    <button
      class="absolute top-3 right-3 z-10 w-8 h-8 flex items-center justify-center rounded-full transition-all"
      :class="isLightBg ? 'text-black/50 hover:text-black hover:bg-black/10' : 'text-white/60 hover:text-white hover:bg-white/20'"
      @click="close"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
      </svg>
    </button>
    <img
      v-if="imageUrl"
      :src="imageUrl"
      :style="{ transform: `scale(${scale})`, transition: 'transform 0.1s ease-out, opacity 0.2s ease-in' }"
      class="w-full h-full object-contain animate-fade-in cursor-grab"
      draggable="false"
      @mousedown="startDrag"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import { join } from "@tauri-apps/api/path";
import { emit } from "@tauri-apps/api/event";

const imageUrl = ref("");
const scale = ref(1);
const isLightBg = ref(false);
const containerRef = ref<HTMLElement>();

function isLocalFilename(src: string): boolean {
  return !!src && !src.startsWith("http://") && !src.startsWith("https://") && !src.startsWith("data:") && !src.startsWith("asset.localhost");
}

async function resolveUrl(src: string): Promise<string> {
  if (!isLocalFilename(src)) return src;
  try {
    const assetsDir = await invoke<string>("get_assets_dir_cmd");
    const filePath = await join(assetsDir, src);
    const bytes = await readFile(filePath);
    const blob = new Blob([bytes]);
    return URL.createObjectURL(blob);
  } catch {
    return src;
  }
}

async function close() {
  const win = getCurrentWebviewWindow();
  await win.close();
}

function startDrag(e: MouseEvent) {
  e.preventDefault();
  getCurrentWebviewWindow().startDragging();
}

function onZoom(e: WheelEvent) {
  const delta = e.deltaY > 0 ? -0.1 : 0.1;
  scale.value = Math.max(0.1, Math.min(5, scale.value + delta));
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    close();
  }
}

function detectLightBg(url: string) {
  const img = new Image();
  img.onload = () => {
    const canvas = document.createElement("canvas");
    canvas.width = 1;
    canvas.height = 1;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    // Sample top-right corner
    ctx.drawImage(img, img.width - 1, 0, 1, 1, 0, 0, 1, 1);
    const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
    // Perceived brightness
    isLightBg.value = (r * 299 + g * 587 + b * 114) / 1000 > 128;
  };
  img.src = url;
}

onMounted(async () => {
  const params = new URLSearchParams(window.location.search);
  const label = params.get("label") || "";
  const stored = localStorage.getItem(`image-preview-${label}`) || "";
  const url = await resolveUrl(stored);
  imageUrl.value = url;
  detectLightBg(url);
  await nextTick();
  // Signal editor that preview is ready to be shown
  emit(`preview-ready:${label}`);
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<style scoped>
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
.animate-fade-in {
  animation: fadeIn 0.2s ease-in;
}
</style>
