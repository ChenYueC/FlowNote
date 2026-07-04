<template>
  <div class="fixed inset-0 pointer-events-none" data-window="radial">
    <!-- Main floating button (bottom-left) -->
    <div
      class="absolute left-8 bottom-8 pointer-events-auto"
      @mouseenter="expanded = true"
      @mouseleave="expanded = false"
    >
      <!-- Menu items (fan out to the right-up) -->
      <div
        v-for="(item, i) in items"
        :key="item.id"
        class="absolute left-1/2 top-1/2 pointer-events-auto"
        :style="menuItemStyle(i)"
      >
        <div
          class="flex flex-col items-center gap-1 cursor-pointer group"
          @click="handleAction(item.action)"
        >
          <div
            class="w-10 h-10 rounded-full flex items-center justify-center transition-all duration-[180ms]"
            :class="expanded ? 'glass-strong scale-100 opacity-100' : 'scale-0 opacity-0'"
            :style="{
              background: expanded ? 'rgba(255,255,255,0.12)' : 'transparent',
              backdropFilter: 'blur(10px)',
              border: '1px solid rgba(255,255,255,0.15)',
              transitionTimingFunction: 'cubic-bezier(.2,.8,.2,1)',
            }"
          >
            <!-- Icon -->
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" class="text-white/80 group-hover:text-white">
              <line v-if="item.id === 'new'" x1="12" y1="5" x2="12" y2="19" /><line v-if="item.id === 'new'" x1="5" y1="12" x2="19" y2="12" />
              <path v-if="item.id === 'remind'" d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9M13.73 21a2 2 0 01-3.46 0" />
              <rect v-if="item.id === 'shot'" x="2" y="3" width="20" height="16" rx="2" /><circle v-if="item.id === 'shot'" cx="12" cy="12" r="3" /><path v-if="item.id === 'shot'" d="M2 9h3l2-3h10l2 3h3" />
              <circle v-if="item.id === 'time'" cx="12" cy="12" r="10" /><polyline v-if="item.id === 'time'" points="12 6 12 12 16 14" />
            </svg>
          </div>
          <span
            class="text-[10px] text-white/60 whitespace-nowrap transition-all duration-[180ms]"
            :class="expanded ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-1'"
            :style="{ transitionTimingFunction: 'cubic-bezier(.2,.8,.2,1)' }"
          >
            {{ item.title }}
          </span>
        </div>
      </div>

      <!-- Center button -->
      <div
        class="relative w-12 h-12 rounded-full flex items-center justify-center cursor-pointer z-10"
        :class="expanded ? 'radial-glow-scoped' : 'radial-pulse-ring'"
        :style="{
          background: 'rgba(255,255,255,0.12)',
          backdropFilter: 'blur(10px)',
          border: '1px solid rgba(255,255,255,0.18)',
          boxShadow: '0 4px 16px rgba(0,0,0,0.3)',
        }"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-white/80">
          <circle cx="12" cy="12" r="3" v-if="!expanded" />
          <line x1="18" y1="6" x2="6" y2="18" v-if="expanded" /><line x1="6" y1="6" x2="18" y2="18" v-if="expanded" />
        </svg>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit } from "@tauri-apps/api/event";

const expanded = ref(false);

const items = [
  { id: "new", title: "新建笔记", angle: -75, radius: 64, action: "new_note" },
  { id: "remind", title: "添加提醒", angle: -35, radius: 64, action: "add_reminder" },
  { id: "shot", title: "截图", angle: 5, radius: 64, action: "screenshot" },
  { id: "time", title: "时间轴", angle: 45, radius: 64, action: "timeline" },
];

function menuItemStyle(index: number) {
  const item = items[index];
  const angleRad = (item.angle * Math.PI) / 180;
  const x = Math.cos(angleRad) * item.radius;
  const y = Math.sin(angleRad) * item.radius;
  return {
    transform: `translate(calc(-50% + ${x}px), calc(-50% + ${y}px))`,
  };
}

async function handleAction(action: string) {
  expanded.value = false;

  switch (action) {
    case "new_note": {
      // Create note via main window event
      const note = await invoke("create_note", {
        input: { title: "新笔记", content: "", workspace: "default" },
      });
      if (note) {
        const label = `editor-${(note as { id: string }).id}`;
        const url = `index.html?window=editor&note_id=${(note as { id: string }).id}`;
        new WebviewWindow(label, {
          url,
          title: "新笔记",
          width: 720,
          height: 560,
          decorations: false,
          transparent: true,
          center: true,
          visible: true,
        });
      }
      break;
    }
    case "add_reminder": {
      // Open main window and focus
      await emit("radial:action", { action: "add_reminder" });
      break;
    }
    case "screenshot": {
      const label = `screenshot-${Date.now()}`;
      const url = `index.html?window=screenshot&note_id=temp`;
      new WebviewWindow(label, {
        url,
        title: "截图",
        width: window.screen.width,
        height: window.screen.height,
        x: 0,
        y: 0,
        decorations: false,
        transparent: true,
        alwaysOnTop: true,
        resizable: false,
        visible: true,
      });
      break;
    }
    case "timeline": {
      await emit("radial:action", { action: "timeline" });
      break;
    }
  }
}
</script>

<style scoped>
.radial-glow-scoped {
  position: relative;
  overflow: visible;
}
.radial-glow-scoped::before {
  content: "";
  position: absolute;
  inset: -5px;
  border-radius: 50%;
  background: rgba(147, 197, 253, 0.18);
  pointer-events: none;
  z-index: -1;
  transition: opacity 0.3s;
}
</style>
