<template>
  <div
    ref="windowRootRef"
    class="h-full flex flex-col rounded-3xl overflow-hidden relative"
    :style="{ background: `rgba(var(--bg-primary), var(--main-opacity, 0.80))` }"
    data-window="main"
    @mouseleave="requestHideCheck"
  >
    <!-- Hello Kitty sparkle decoration -->
    <div class="theme-hk-sparkle" />
    <!-- Title bar -->
    <div class="drag-region flex items-center justify-between px-4 pt-4 pb-2">
      <div class="flex items-center gap-2">
        <h1 class="text-base font-semibold select-none ml-1" style="color: rgba(var(--text-primary), 0.8)">
          FlowNote
        </h1>
        <!-- Hello Kitty bow decoration -->
        <svg class="theme-hk-bow w-5 h-5 flex-shrink-0" viewBox="0 0 32 32" fill="none">
          <path d="M10 8C6 4 2 3 1 5s1 6 5 9M22 8c4-4 8-5 9-3s-1 6-5 9" stroke="#ff8fab" stroke-width="2" fill="#ffb3c6" opacity="0.7"/>
          <circle cx="16" cy="14" r="2.5" fill="#ff6b91" opacity="0.6"/>
        </svg>
      </div>
      <div class="flex items-center gap-1">
        <button
          class="no-drag relative flex items-center justify-center w-7 h-7 rounded-full transition-all duration-200 hover:bg-white/8"
          :class="pendingCount > 0 ? 'text-white/80' : 'text-white/40 hover:text-white/70'"
          @click="showReminderList = !showReminderList"
          data-reminder-toggle
          title="提醒列表"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" />
            <path d="M13.73 21a2 2 0 01-3.46 0" />
          </svg>
          <span v-if="pendingCount > 0" class="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-red-400" />
        </button>
        <button
          class="no-drag flex items-center justify-center w-6 h-6 rounded-full text-white/30 hover:text-white/60 hover:bg-white/8 transition-all ml-1"
          @click="minimizeWindow"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
        <button
          class="no-drag flex items-center justify-center w-6 h-6 rounded-full text-white/30 hover:text-red-400 hover:bg-red-400/10 transition-all"
          @click="closeWindow"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Search -->
    <div class="px-3 mb-3">
      <NoteSearch @search="handleSearch" />
    </div>

    <!-- Workspace tabs row -->
    <div class="flex items-center gap-1 px-3 mb-2">
      <div ref="tabsRef" class="flex-1 flex gap-0.5 overflow-x-auto tabs-scrollbar pb-1.5" @wheel="onWheelTabs">
        <button
          v-for="ws in workspaceTabs"
          :key="ws"
          class="no-drag whitespace-nowrap px-2.5 py-1 rounded-full text-xs font-medium transition-all duration-200"
          :class="notesStore.currentWorkspace === ws ? 'bg-white/15 text-white' : 'text-white/40 hover:text-white/70'"
          @click="switchWorkspace(ws)"
          @contextmenu="openTabMenu($event, ws)"
        >
          {{ workspaceLabels[ws] || ws }}
        </button>
      </div>
      <button
        class="no-drag flex-shrink-0 px-2 py-1 rounded-full text-xs text-white/30 hover:text-white/60 transition-all"
        @click="showAddWorkspace = true"
      >
        +
      </button>
    </div>

    <!-- Add workspace input -->
    <div v-if="showAddWorkspace" class="px-3 mb-2 flex gap-2 animate-fade-in-up">
      <input v-model="newWorkspaceName" type="text" placeholder="请输入新标签名称" maxlength="15"
        class="no-drag flex-1 px-3 py-1.5 rounded-full text-xs bg-white/5 border border-white/6 text-white/90 outline-none placeholder:text-white/20 focus:bg-white/8 focus:border-white/10"
        @keyup.enter="addWorkspace" />
      <button class="no-drag px-3 py-1.5 rounded-full text-xs text-white/70 bg-white/10 hover:bg-white/15 transition-all" @click="addWorkspace">确定</button>
      <button class="no-drag px-3 py-1.5 rounded-full text-xs text-white/40 hover:text-white/70 transition-all" @click="showAddWorkspace = false">取消</button>
    </div>

    <!-- Filters -->
    <div class="flex gap-0.5 px-3 mb-3">
      <button v-for="tab in tabs" :key="tab.key"
        class="no-drag px-2.5 py-1 rounded-full text-xs font-medium transition-all duration-200"
        :class="activeTab === tab.key ? 'bg-white/15 text-white' : 'text-white/40 hover:text-white/70'"
        @click="activeTab = tab.key">{{ tab.label }}</button>
    </div>

    <!-- Reminder list overlay -->
    <Transition name="reminder-fade">
      <div v-if="showReminderList" data-reminder-panel
        class="absolute inset-0 z-50 flex flex-col rounded-3xl overflow-hidden"
        style="background: rgba(var(--bg-primary), 0.95);">
        <ReminderView @close="showReminderList = false" />
      </div>
    </Transition>

    <!-- Create reminder modal (standalone) -->
    <CreateReminderModal :visible="showCreateReminderModal" @close="showCreateReminderModal = false" />

    <!-- Note list (always visible) -->
    <NoteList :notes="filteredNotes" :activeId="currentNote?.id ?? null"
      @openNote="handleOpenNote" @toggleFavorite="handleToggleFavorite"
      @deleteNote="requestDeleteNote" @dragStart="handleDragStart" />

    <!-- Timeline slide-in overlay -->
    <Transition name="timeline-slide">
      <div v-if="showTimeline" data-timeline-panel
        class="absolute inset-0 z-50 flex flex-col rounded-3xl overflow-hidden"
        style="background: rgba(var(--bg-primary), 0.95);">
        <div class="drag-region flex items-center justify-between px-4 pt-4 pb-3">
          <button class="no-drag flex items-center gap-1 px-2 py-1 rounded-lg text-white/50 hover:text-white hover:bg-white/8 transition-all"
            @click="showTimeline = false">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M15 18l-6-6 6-6"/>
            </svg>
            <span class="text-xs">返回</span>
          </button>
          <h2 class="text-sm font-medium text-white/80 absolute left-1/2 -translate-x-1/2">时间轴</h2>
          <div class="w-16"></div>
        </div>
        <div ref="timelineScrollRef" class="flex-1 overflow-y-scroll px-1" @scroll="onTimelineScroll">
          <TimelineView :items="timelineItems" @open="handleOpenNote" />
        </div>
      </div>
    </Transition>

    <!-- Bottom bar: radial menu + timeline toggle -->
    <div class="relative px-3 py-3 flex gap-2 items-center">
      <!-- Radial menu background mask -->
      <div v-if="radialOpen" class="absolute inset-0 z-40 bg-black/20 rounded-3xl pointer-events-auto" @click="radialOpen = false" />

      <!-- Radial menu trigger (horizontal to the right) -->
      <div class="relative no-drag flex-shrink-0 transition-all duration-[180ms]"
        :class="showTimeline || showReminderList ? 'z-10' : 'z-50'"
        :style="{ width: radialOpen ? '200px' : '40px' }"
        @mouseenter="onRadialEnter"
        @mouseleave="onRadialLeave">
        <div v-for="(item, i) in radialItems" :key="item.id"
          class="absolute top-1/2 pointer-events-auto transition-all duration-[180ms]"
          :style="radialMenuItemStyle(i)">
          <div class="flex flex-col items-center gap-0.5 cursor-pointer group" @click="handleRadialAction(item.action)">
            <div class="w-9 h-9 rounded-full flex items-center justify-center transition-all"
              style="background:rgba(var(--bg-secondary), 0.92);backdrop-filter:blur(10px) saturate(130%);border:1px solid rgba(var(--border), 0.16);">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" class="text-white/80 group-hover:text-white">
                <line v-if="item.id==='new'" x1="12" y1="5" x2="12" y2="19" /><line v-if="item.id==='new'" x1="5" y1="12" x2="19" y2="12" />
                <path v-if="item.id==='remind'" d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9M13.73 21a2 2 0 01-3.46 0" />
                <circle v-if="item.id==='settings'" cx="12" cy="12" r="3" /><path v-if="item.id==='settings'" d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
              </svg>
            </div>
            <span class="text-[9px] text-white/50 whitespace-nowrap">{{ item.title }}</span>
          </div>
        </div>

        <div class="w-10 h-10 rounded-full flex items-center justify-center cursor-pointer z-10 radial-pulse-ring"
          :style="{ background: 'rgba(255,255,255,0.1)', border: '1px solid rgba(255,255,255,0.15)',
            boxShadow: '0 2px 12px rgba(0,0,0,0.2)' }">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-white/70">
            <circle cx="12" cy="5" r="2" /><circle cx="12" cy="12" r="2" /><circle cx="12" cy="19" r="2" />
          </svg>
        </div>
      </div>

      <button class="no-drag ml-auto py-2.5 px-3 glass-strong rounded-2xl text-sm text-white/50 hover:text-white/80 transition-all"
        :class="showTimeline ? 'text-white bg-white/15' : ''"
        data-timeline-toggle
        @click="showTimeline = !showTimeline" title="时间轴">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" />
        </svg>
      </button>
    </div>

    <!-- Delete confirmation (inside window, preserves round corners) -->
    <div v-if="deleteTarget" class="absolute inset-0 z-[9999] flex items-center justify-center rounded-3xl overflow-hidden"
      style="background: rgba(0, 0, 0, 0.60);"
      @click.self="deleteTarget = null">
      <div class="glass-strong rounded-2xl p-6 w-72 animate-scale-in">
        <p class="text-sm text-white/80 mb-2">确认删除这条笔记？</p>
        <p class="text-xs text-white/40 mb-4 truncate">{{ deleteTarget.title || "无标题" }}</p>
        <div class="flex gap-2 justify-end">
          <button class="no-drag px-4 py-1.5 rounded-full text-xs text-white/50 hover:text-white/80 hover:bg-white/8 transition-all" @click="deleteTarget = null">取消</button>
          <button class="no-drag px-4 py-1.5 rounded-full text-xs text-red-300 bg-red-400/10 hover:bg-red-400/20 transition-all" @click="confirmDelete">删除</button>
        </div>
      </div>
    </div>

    <!-- Right-click tab menu -->
    <div
      v-if="contextMenu.show"
      class="fixed z-[9998] rounded-xl overflow-hidden w-[100px] glass-strong"
      :style="{ left: menuX + 'px', top: menuY + 'px' }"
    >
      <button
        v-if="contextMenu.ws !== 'default'"
        class="ctx-item ctx-first"
        @click="startRenameWorkspace"
      >
        编辑标签
      </button>
      <button
        v-if="contextMenu.ws !== 'default'"
        class="ctx-item ctx-last"
        style="color: rgba(248, 113, 113, 0.8);"
        @click="requestDeleteWorkspace"
      >
        删除标签
      </button>
    </div>

    <!-- Rename workspace input -->
    <div v-if="renameTarget" class="absolute inset-0 z-[9999] flex items-center justify-center rounded-3xl overflow-hidden"
      style="background: rgba(0, 0, 0, 0.60);"
      @click.self="renameTarget = null">
      <div class="glass-strong rounded-2xl p-6 w-72 animate-scale-in">
        <p class="text-sm text-white/80 mb-3">编辑标签名称</p>
        <input v-model="renameValue" type="text" maxlength="15" class="no-drag w-full px-3 py-1.5 rounded-full text-xs bg-white/5 border border-white/6 text-white/80 outline-none placeholder:text-white/20 mb-4" @keyup.enter="confirmRenameWorkspace" />
        <div class="flex gap-2 justify-end">
          <button class="no-drag px-3 py-1 rounded-full text-xs text-white/50 hover:text-white/80 hover:bg-white/8 transition-all" @click="renameTarget = null">取消</button>
          <button class="no-drag px-3 py-1 rounded-full text-xs text-white bg-white/15 hover:bg-white/20 transition-all" @click="confirmRenameWorkspace">确定</button>
        </div>
      </div>
    </div>

    <!-- Delete workspace confirmation -->
    <div v-if="deleteWorkspaceTarget" class="absolute inset-0 z-[9999] flex items-center justify-center rounded-3xl overflow-hidden"
      style="background: rgba(0, 0, 0, 0.60);"
      @click.self="deleteWorkspaceTarget = null">
      <div class="glass-strong rounded-2xl p-6 w-72 animate-scale-in">
        <p class="text-sm text-white/80 mb-2">确认删除标签？</p>
        <p class="text-xs text-white/40 mb-4 line-clamp-2">「{{ deleteWorkspaceTarget }}」下的笔记将被移至默认工作区</p>
        <div class="flex gap-2 justify-end">
          <button class="no-drag px-3 py-1 rounded-full text-xs text-white/50 hover:text-white/80 hover:bg-white/8 transition-all" @click="deleteWorkspaceTarget = null">取消</button>
          <button class="no-drag px-3 py-1 rounded-full text-xs text-red-300 bg-red-400/10 hover:bg-red-400/20 transition-all" @click="confirmDeleteWorkspace">删除</button>
        </div>
      </div>
    </div>

    <!-- Close confirmation (inside window, preserves round corners) -->
    <div v-if="showCloseConfirm" class="absolute inset-0 z-[9999] flex items-center justify-center rounded-3xl overflow-hidden"
      style="background: rgba(0, 0, 0, 0.60);"
      @click.self="cancelClose">
      <div class="glass-strong rounded-2xl p-8 w-64 text-center animate-scale-in">
        <p class="text-sm text-white/80 mb-6">确认退出 FlowNote？</p>
        <div class="flex gap-3 justify-center">
          <button class="no-drag px-5 py-2 rounded-full text-xs text-white/60 hover:text-white/80 hover:bg-white/8 transition-all" @click="cancelClose">取消</button>
          <button class="no-drag px-5 py-2 rounded-full text-xs text-red-300 bg-red-400/10 hover:bg-red-400/20 transition-all" @click="confirmClose">确定</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onUnmounted } from "vue";
import { useNotesStore } from "@/stores/notes";
import { useRemindersStore } from "@/stores/reminders";
import { useWindowsStore } from "@/stores/windows";
import { useSettingsStore } from "@/stores/settings";
import { useDragDetach } from "@/composables/useDragDetach";
import { useEdgeDock } from "@/composables/useEdgeDock";
import { WebviewWindow, getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { PhysicalPosition, PhysicalSize, currentMonitor } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import NoteSearch from "@/components/notes/NoteSearch.vue";
import NoteList from "@/components/notes/NoteList.vue";
import TimelineView from "@/components/notes/TimelineView.vue";
import ReminderView from "@/components/layout/ReminderView.vue";
import CreateReminderModal from "@/components/layout/CreateReminderModal.vue";
import { useToast } from "@/composables/useToast";
import type { Note } from "@/types";

const notesStore = useNotesStore();
const remindersStore = useRemindersStore();
const windowsStore = useWindowsStore();
const settingsStore = useSettingsStore();
const { startDrag, updateDrag, endDrag } = useDragDetach();
const { windowState, requestHideCheck } = useEdgeDock();
const { toast } = useToast();

const MANIFEST_URL = "https://raw.githubusercontent.com/ChenYueC/FlowNote/main/latest.json";

async function checkUpdateOnStart() {
  // Check if user dismissed update within 3 days
  const lastDismissed = localStorage.getItem("update-dismissed-at");
  if (lastDismissed) {
    const elapsed = Date.now() - parseInt(lastDismissed, 10);
    if (elapsed < 3 * 24 * 60 * 60 * 1000) return;
  }

  try {
    const info = await invoke<{ need_update: boolean; version: string; download_url: string }>("check_update", {
      manifestUrl: MANIFEST_URL,
    });
    if (!info.need_update) return;

    // Open custom update dialog window
    const existing = await WebviewWindow.getByLabel("update-dialog");
    if (existing) { await existing.show(); existing.setFocus(); return; }

    new WebviewWindow("update-dialog", {
      url: `index.html?window=update-dialog&version=${info.version}&url=${encodeURIComponent(info.download_url)}`,
      title: "FlowNote 更新",
      width: 320, height: 260,
      decorations: false, transparent: true, shadow: false,
      center: true, visible: true,
      resizable: false,
      alwaysOnTop: true,
    });
  } catch (e) {
    // Silently fail — don't bother user on network errors
  }
}

const activeTab = ref("all");
const showTimeline = ref(false);
const showAddWorkspace = ref(false);
const showReminderList = ref(false);
const showCreateReminderModal = ref(false);

watch(showReminderList, (val) => {
  if (val) {
    remindersStore.loadReminders();
    radialOpen.value = false;
  }
});

watch(showTimeline, (val) => {
  if (val) {
    notesStore.loadTimeline();
    radialOpen.value = false;
  }
});
const newWorkspaceName = ref("");
const deleteTarget = ref<Note | null>(null);
const tabsRef = ref<HTMLElement | null>(null);
const timelineScrollRef = ref<HTMLElement | null>(null);

function onTimelineScroll() {
  if (!timelineScrollRef.value || !notesStore.timelineHasMore) return;
  const el = timelineScrollRef.value;
  if (el.scrollHeight - el.scrollTop - el.clientHeight < 100) {
    notesStore.loadMoreTimeline();
  }
}

const now = ref(Math.floor(Date.now() / 1000));
// Update now every 30 seconds to keep pendingCount reactive
let nowTimer: ReturnType<typeof setInterval> | null = null;

function onWheelTabs(e: WheelEvent) {
  if (!tabsRef.value) return;
  e.preventDefault();
  tabsRef.value.scrollLeft += e.deltaY;
}

const tabs = [{ key: "all", label: "全部" }, { key: "favorites", label: "收藏" }];

const workspaceLabels: Record<string, string> = {
  default: "默认",
};

const workspaceTabs = computed(() => {
  const ws = [...notesStore.workspaces];
  ws.sort((a, b) => {
    if (a === "default") return -1;
    if (b === "default") return 1;
    return a.localeCompare(b);
  });
  return ws;
});

const filteredNotes = computed(() => {
  let result = notesStore.notes;
  if (activeTab.value === "favorites") result = result.filter((n) => n.favorite === 1);
  return result;
});

const pendingCount = computed(() => remindersStore.reminders.filter((r) => !r.completed && !r.cancelled && !r.deleted && r.remind_at > now.value).length);
const timelineItems = computed(() => notesStore.timelineItems);
const currentNote = computed(() => notesStore.currentNote);

// Radial menu: horizontal to the right, labels below icons
const radialItems = [
  { id: "new", title: "新建", action: "new_note" },
  { id: "remind", title: "提醒", action: "add_reminder" },
  { id: "settings", title: "设置", action: "open_settings" },
];

function radialMenuItemStyle(index: number) {
  const gap = 43;
  const closedX = 4;
  const openX = 48 + index * (gap);

  return {
    left: radialOpen.value ? `${openX}px` : `${closedX}px`,
    top: '50%',
    opacity: radialOpen.value ? 1 : 0,
    transform: 'translateY(-50%)',
    transitionTimingFunction: 'cubic-bezier(.2,.8,.2,1)',
  };
}

async function minimizeWindow() {
  try {
    const pos = await getCurrentWebviewWindow().outerPosition();
    localStorage.setItem("main-window-pos", JSON.stringify({ x: pos.x, y: pos.y }));
  } catch { /* ignore */ }
  invoke("hide_main_window");
}

const showCloseConfirm = ref(false);
const deleteWorkspaceTarget = ref<string | null>(null);
const renameTarget = ref<string | null>(null);
const renameValue = ref("");
const contextMenu = reactive({ show: false, x: 0, y: 0, ws: "" });
const menuX = ref(0);
const menuY = ref(0);

function closeContextMenu() {
  contextMenu.show = false;
}

function openTabMenu(e: MouseEvent, ws: string) {
  if (ws === "default") return;
  const menuW = 100;
  const menuH = 64;
  let mx = e.clientX;
  let my = e.clientY;
  if (mx + menuW > window.innerWidth) mx = window.innerWidth - menuW - 8;
  if (my + menuH > window.innerHeight) my = window.innerHeight - menuH - 8;
  menuX.value = mx;
  menuY.value = my;
  contextMenu.show = true;
  contextMenu.ws = ws;
  // Click outside to close
  setTimeout(() => {
    document.addEventListener("click", closeContextMenu, { once: true });
  }, 0);
}

function startRenameWorkspace() {
  renameTarget.value = contextMenu.ws;
  renameValue.value = contextMenu.ws;
  contextMenu.show = false;
}

async function confirmRenameWorkspace() {
  if (!renameTarget.value || !renameValue.value.trim()) return;
  const oldName = renameTarget.value;
  const newName = renameValue.value.trim();
  if (oldName !== newName) {
    await notesStore.renameWorkspace(oldName, newName);
    if (notesStore.currentWorkspace === newName) {
      await notesStore.loadNotes(newName);
    }
  }
  renameTarget.value = null;
}

function requestDeleteWorkspace() {
  deleteWorkspaceTarget.value = contextMenu.ws;
  contextMenu.show = false;
}

async function confirmDeleteWorkspace() {
  if (!deleteWorkspaceTarget.value) return;
  const ws = deleteWorkspaceTarget.value;
  await notesStore.removeWorkspace(ws);
  if (notesStore.currentWorkspace === ws) {
    await notesStore.loadNotes("default");
  }
  deleteWorkspaceTarget.value = null;
}
const radialOpen = ref(false);
let radialLeaveTimer: ReturnType<typeof setTimeout> | null = null;

function onRadialEnter() {
  if (radialLeaveTimer) { clearTimeout(radialLeaveTimer); radialLeaveTimer = null; }
  radialOpen.value = true;
}

function onRadialLeave() {
  radialLeaveTimer = setTimeout(() => {
    radialOpen.value = false;
  }, 150);
}

async function handleRadialAction(action: string) {
  radialOpen.value = false;
  switch (action) {
    case "new_note": handleNewNote(); break;
    case "add_reminder": { showCreateReminderModal.value = true; break; }
    case "timeline": showTimeline.value = !showTimeline.value; break;
    case "open_settings": openSettings(); break;
  }
}

async function openSettings() {
  const label = "settings";
  const existing = await WebviewWindow.getByLabel(label);
  if (existing) { await existing.show(); existing.setFocus(); return; }
  new WebviewWindow(label, {
    url: "index.html?window=settings",
    title: "设置",
    width: 320, height: 520,
    decorations: false, transparent: true, shadow: false,
    center: true, visible: true,
  });
}

function closeWindow() { showCloseConfirm.value = true; }
async function confirmClose() {
  try {
    const pos = await getCurrentWebviewWindow().outerPosition();
    localStorage.setItem("main-window-pos", JSON.stringify({ x: pos.x, y: pos.y }));
  } catch { /* ignore */ }
  invoke("exit_app");
}
function cancelClose() { showCloseConfirm.value = false; }

async function switchWorkspace(ws: string) { await notesStore.loadNotes(ws); }

async function addWorkspace() {
  const name = newWorkspaceName.value.trim();
  if (name) await notesStore.addWorkspace(name);
  newWorkspaceName.value = "";
  showAddWorkspace.value = false;
}

async function handleSearch(query: string) { await notesStore.searchNotes(query); }

async function handleNewNote() {
  const note = await notesStore.createNote({ title: "新笔记", content: "" }, notesStore.currentWorkspace);
  if (note) { notesStore.setCurrentNote(note); openEditor(note); }
}

function handleOpenNote(note: Note) {
  const exists = notesStore.notes.find((n) => n.id === note.id);
  if (!exists) {
    toast("笔记已被删除！");
    return;
  }
  notesStore.setCurrentNote(note);
  openEditor(note);
}

async function openEditor(note: Note) {
  const label = `editor-${note.id}`;
  const existing = await WebviewWindow.getByLabel(label);
  if (existing) {
    try {
      await existing.outerSize();
      await existing.show();
      existing.setFocus();
      return;
    } catch {
      // Window is destroyed, continue to create new one
    }
  }

  // Limit to 3 simultaneous editor windows
  // Track open editors in localStorage
  let openEditors: string[] = [];
  try { openEditors = JSON.parse(localStorage.getItem("open-editors") || "[]"); } catch { openEditors = []; }
  // Clean up editors that no longer exist
  const validEditors = [];
  for (const editorLabel of openEditors) {
    try {
      const win = await WebviewWindow.getByLabel(editorLabel);
      if (win) {
        await win.outerSize();
        validEditors.push(editorLabel);
      }
    } catch {
      // Window is destroyed, remove from list
    }
  }
  // Update localStorage with valid editors
  localStorage.setItem("open-editors", JSON.stringify(validEditors));

  if (validEditors.length >= 3) {
    toast("最多同时打开 3 个编辑页");
    return;
  }

  // Restore saved dimensions or use defaults
  let width = 360, height = 680;
  try {
    const saved = JSON.parse(localStorage.getItem(`editor-size-${note.id}`) || "null");
    if (saved) { width = saved.width; height = saved.height; }
  } catch {}
  const webview = new WebviewWindow(label, { url: `index.html?window=editor&note_id=${note.id}`, title: note.title || "编辑",
    width, height, decorations: false, transparent: true, shadow: false, center: true, visible: true });
  webview.once('tauri://error', (e) => console.error('[openEditor] create failed:', label, e));

  // Track this editor window
  validEditors.push(label);
  localStorage.setItem("open-editors", JSON.stringify(validEditors));
}
async function handleToggleFavorite(id: string) { await notesStore.toggleFavorite(id); }

function requestDeleteNote(id: string) {
  const note = notesStore.notes.find((n) => n.id === id);
  if (note) deleteTarget.value = note;
}

async function confirmDelete() {
  if (!deleteTarget.value) return;
  const noteId = deleteTarget.value.id;
  // Prevent deleting if the note is being edited
  const editorLabel = `editor-${noteId}`;
  const editorWindow = await WebviewWindow.getByLabel(editorLabel);
  if (editorWindow) {
    toast("该笔记编辑中！");
    deleteTarget.value = null;
    return;
  }
  await notesStore.deleteNote(noteId);
  localStorage.removeItem(`editor-size-${noteId}`);
  deleteTarget.value = null;
}

function handleDragStart(e: PointerEvent, note: Note) {
  startDrag(e, note);
  const onMove = (ev: PointerEvent) => updateDrag(ev);
  const onUp = (ev: PointerEvent) => {
    document.removeEventListener("pointermove", onMove);
    document.removeEventListener("pointerup", onUp);
    const result = endDrag(ev);
    if (result) { windowsStore.createFloating({ note_id: result.note.id, x: result.x - 160, y: result.y - 20 }); }
  };
  document.addEventListener("pointermove", onMove);
  document.addEventListener("pointerup", onUp);
}

onMounted(async () => {
  await notesStore.loadWorkspaces();
  await notesStore.loadNotes();

  // Check for updates on startup (3-day reminder)
  if (settingsStore.checkUpdate) checkUpdateOnStart();

  // Update now every 30 seconds for pendingCount reactivity
  nowTimer = setInterval(() => { now.value = Math.floor(Date.now() / 1000); }, 30000);

  const unlisteners: UnlistenFn[] = [];

  // Refresh note list when an editor closes
  const unlisten1 = await listen("editor:closed", () => {
    notesStore.loadNotes();
  });
  unlisteners.push(unlisten1);

  // Refresh reminder list when a reminder is completed/snoozed
  const unlisten2 = await listen("reminder:updated", () => {
    remindersStore.loadReminders();
  });
  unlisteners.push(unlisten2);

  // Restore saved window position
  const savedPos = localStorage.getItem("main-window-pos");
  if (savedPos) {
    try {
      const { x, y } = JSON.parse(savedPos);
      const window = getCurrentWebviewWindow();
      await window.setPosition(new PhysicalPosition(x, y));
    } catch (e) {
      console.error("Failed to restore position:", e);
    }
  }

  // Restore saved window size
  const savedSize = localStorage.getItem("main-window-size");
  if (savedSize) {
    try {
      const { width, height } = JSON.parse(savedSize);
      const window = getCurrentWebviewWindow();
      await window.setSize(new PhysicalSize(width, height));
    } catch (e) {
      console.error("Failed to restore size:", e);
    }
  }

  // Save window position after stop moving for 1 second
  const appWindow = getCurrentWebviewWindow();
  let moveTimer: ReturnType<typeof setTimeout> | null = null;

  const unlistenMove = await appWindow.listen('tauri://move', async () => {
    if (moveTimer) clearTimeout(moveTimer);
    moveTimer = setTimeout(async () => {
      try {
        const pos = await appWindow.outerPosition();
        if (windowState.value === "visible" && pos.x >= 0 && pos.y >= 0) {
          localStorage.setItem("main-window-pos", JSON.stringify({ x: pos.x, y: pos.y }));
        }
        requestHideCheck();
      } catch { /* ignore */ }
    }, 1000);
  });

  // Save window size after stop resizing for 1 second
  let resizeTimer: ReturnType<typeof setTimeout> | null = null;

  const unlistenResize = await appWindow.listen('tauri://resize', async () => {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(async () => {
      try {
        const size = await appWindow.outerSize();
        if (size.width > 0 && size.height > 0) {
          localStorage.setItem("main-window-size", JSON.stringify({ width: size.width, height: size.height }));
        }
      } catch { /* ignore */ }
    }, 1000);
  });

  // Listen for due reminders and spawn notification popup
  const unlisten3 = await listen("reminder:due", async (event) => {
    const reminder = event.payload as any;
    if (!reminder?.id) return;
    const label = `reminder-${reminder.id}`;
    // Avoid duplicate windows
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) return;
    // Get current monitor for positioning (supports multi-display)
    let screenW = 1920;
    let monitorPosX = 0;
    try {
      const monitor = await currentMonitor();
      if (monitor) {
        screenW = monitor.size.width / monitor.scaleFactor;
        monitorPosX = monitor.position.x / monitor.scaleFactor;
      }
    } catch {}
    const winW = 340;
    const winH = 160;
    const x = Math.round(monitorPosX + screenW) - winW - 20;
    const y = 20;
    new WebviewWindow(label, {
      url: `index.html?window=reminder&reminder_id=${reminder.id}`,
      width: winW,
      height: winH,
      x,
      y,
      decorations: false,
      transparent: true,
      shadow: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      resizable: false,
      focus: false,
    });
  });

  // Register cleanup for all event listeners
  onUnmounted(() => {
    if (nowTimer) { clearInterval(nowTimer); nowTimer = null; }
    unlisteners.forEach(fn => fn());
    unlistenMove();
    unlistenResize();
    unlisten3();
  });

});
</script>

<style scoped>
.timeline-slide-enter-active,
.timeline-slide-leave-active {
  transition: all 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
}
.timeline-slide-enter-from {
  transform: translateX(100%);
  opacity: 0.6;
}
.timeline-slide-leave-to {
  transform: translateX(100%);
  opacity: 0.6;
}

.reminder-fade-enter-active,
.reminder-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.2, 0.8, 0.2, 1);
}
.reminder-fade-enter-from {
  transform: translateY(-100%);
  opacity: 0.8;
}
.reminder-fade-leave-to {
  transform: translateY(-100%);
  opacity: 0.8;
}
.ctx-item {
  display: flex;
  width: 100%;
  align-items: center;
  padding: 6px 12px;
  font-size: 12px;
  color: rgba(var(--text-primary), 0.7);
  border: none;
  background: none;
  cursor: pointer;
  transition: background-color 0.15s;
}
.ctx-item:hover {
  background: rgba(var(--interactive), 0.1);
}
.ctx-first {
  border-top-left-radius: 12px;
  border-top-right-radius: 12px;
}
.ctx-last {
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}
.radial-glow {
  position: relative;
  overflow: visible;
}
.radial-glow::before {
  content: "";
  position: absolute;
  inset: -4px;
  border-radius: 50%;
  background: rgba(var(--accent), 0.15);
  pointer-events: none;
  z-index: -1;
  transition: opacity 0.3s;
}

.tabs-scrollbar::-webkit-scrollbar {
  height: 2px;
}
.tabs-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.tabs-scrollbar::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 999px;
}
.tabs-scrollbar:has(button:hover)::-webkit-scrollbar-thumb {
  background: rgba(var(--scrollbar), 0.3);
}
</style>
