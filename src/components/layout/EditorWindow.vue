<template>
  <div
    class="h-full flex flex-col rounded-3xl overflow-hidden"
    :style="{ background: `rgba(var(--bg-primary), var(--editor-opacity, 0.82))` }"
    data-window="editor"
  >
    <!-- Collapsed state (shown before hiding) — removed: now that the editor
         is fully hidden (window.hide()) when docked, the taskbar icon is gone
         and this collapsed view was only a taskbar-hover preview, no longer
         needed. Kept here commented for reference.
    <div v-if="autoHide.isCollapsed.value" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="text-white/40 text-5xl mb-5">📝</div>
        <div class="text-white/30 text-3xl font-bold mb-2">已收起</div>
        <div class="text-white/20 text-3xl truncate max-w-[300px] mx-auto">{{ title || '无标题' }}</div>
      </div>
    </div>
    -->

    <!-- Normal content (hidden when auto-hidden, use v-show to preserve state) -->
    <div v-show="!autoHide.isHidden.value" class="flex-1 flex flex-col overflow-hidden">
    <EditorToolbar
      class="relative z-10"
      :isPreview="isPreview"
      :pinned="pinned"
      @toggleMode="isPreview = !isPreview"
      @screenshot="handleScreenshot"
      @setReminder="showReminderPicker = !showReminderPicker"
      @pin="togglePin"
      @minimize="handleMinimize"
      @close="handleClose"
    />

    <!-- Create reminder modal -->
    <CreateReminderModal
      :visible="showReminderPicker"
      :initialTitle="title"
      @close="closeReminderModal"
    />

    <!-- Title (fixed height, transform:scale for visual transition without layout shift) -->
    <div class="px-4 h-9 flex items-center relative z-10" :class="titleCompact ? 'text-center' : ''">
      <input
        v-model="title"
        type="text"
        placeholder="标题"
        class="no-drag w-full bg-transparent font-semibold text-white/90 outline-none placeholder-white/20 transition-transform duration-200 text-xl"
        :class="titleCompact ? 'text-center' : ''"
        :style="{ fontFamily: 'JetBrains Mono, PingFang SC, Microsoft YaHei, sans-serif', transform: titleCompact ? 'scale(0.75) translateY(-10px)' : 'scale(1)', transformOrigin: 'center center' }"
        maxlength="15"
        @input="markDirty"
      />
    </div>

    <!-- Editor -->
    <div class="flex-1 overflow-hidden px-2 relative z-0" @contextmenu="onContextMenu">
      <MilkdownEditor ref="editorRef" v-model="content" :colorRanges="colorRanges" @update:colorRanges="onColorRangesChange" @floatMenuHover="(h) => autoHide.suppressAutoHide.value = h" :readonly="isPreview" :noteId="noteId" />
      <!-- Find bar -->
      <FindBar
        v-if="showFind"
        ref="findBarRef"
        :matchCount="findMatchCount"
        :currentMatch="findCurrentMatch"
        @search="onFindSearch"
        @next="onFindNext"
        @prev="onFindPrev"
        @close="onFindClose"
        @replace="onReplace"
        @replaceAll="onReplaceAll"
      />
      <!-- Scroll to top button -->
      <button
        v-show="titleCompact"
        tabindex="-1"
        class="absolute bottom-3 right-6 w-8 h-8 rounded-full flex items-center justify-center bg-white/10 hover:bg-white/20 text-white/50 hover:text-white/80 transition-all z-20"
        @mousedown.prevent="scrollToTop"
        title="回到顶部"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 15l-6-6-6 6"/></svg>
      </button>
    </div>

    <!-- Formatting toolbar + word count -->
    <div class="px-3 py-2 flex items-center gap-1.5">
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="粗体" @mousedown.prevent @click="format('bold')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4h8a4 4 0 014 4 4 4 0 01-4 4H6z"/><path d="M6 12h9a4 4 0 014 4 4 4 0 01-4 4H6z"/></svg>
      </button>
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="删除线" @mousedown.prevent @click="format('strikethrough')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H7"/><line x1="4" y1="12" x2="20" y2="12"/></svg>
      </button>
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="下划线" @mousedown.prevent @click="format('underline')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4v6a6 6 0 0012 0V4"/><line x1="4" y1="20" x2="20" y2="20"/></svg>
      </button>
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="标题" @mousedown.prevent @click="format('heading')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 5v14M18 5v14M6 12h12"/></svg>
      </button>
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="无序列表" @mousedown.prevent @click="format('bulletList')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="9" y1="6" x2="20" y2="6"/><line x1="9" y1="12" x2="20" y2="12"/><line x1="9" y1="18" x2="20" y2="18"/><circle cx="4" cy="6" r="1.5" fill="currentColor"/><circle cx="4" cy="12" r="1.5" fill="currentColor"/><circle cx="4" cy="18" r="1.5" fill="currentColor"/></svg>
      </button>
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="有序列表" @mousedown.prevent @click="format('orderedList')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="10" y1="6" x2="20" y2="6"/><line x1="10" y1="12" x2="20" y2="12"/><line x1="10" y1="18" x2="20" y2="18"/><text x="3" y="8" font-size="7" fill="currentColor" stroke="none" font-family="sans-serif">1</text><text x="3" y="14" font-size="7" fill="currentColor" stroke="none" font-family="sans-serif">2</text><text x="3" y="20" font-size="7" fill="currentColor" stroke="none" font-family="sans-serif">3</text></svg>
      </button>
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="待办事项" @mousedown.prevent @click="format('taskList')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="5" width="14" height="14" rx="2"/><path d="M7 12l2 2 4-4"/></svg>
      </button>
      <button tabindex="-1" class="no-drag p-1.5 rounded text-white/40 hover:text-white/80 hover:bg-white/10 transition-all" :class="isPreview ? 'invisible' : ''" title="字体颜色" @mousedown.prevent @click="format('fontColor')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m5 15 7-11 7 11M7 11h10"/><path d="M3 20h18" stroke-width="2.5"/></svg>
      </button>
      <div class="flex-1" />
      <span class="text-[10px] text-white/25 translate-y-[2px]">{{ displayWordCount }}</span>
    </div>

    <!-- Close confirmation modal -->
    <div
      v-if="showCloseConfirm"
      class="absolute inset-0 z-[9999] flex items-center justify-center bg-black/40 rounded-3xl"
      @click.self="cancelClose"
    >
      <div class="glass-strong rounded-2xl p-8 w-64 text-center animate-scale-in">
        <p class="text-sm text-white/80 mb-6">确认关闭编辑窗口？</p>
        <div class="flex gap-3 justify-center">
          <button
            class="no-drag px-5 py-2 rounded-full text-xs text-white/60 hover:text-white/80 hover:bg-white/8 transition-all"
            @click="cancelClose"
          >
            取消
          </button>
          <button
            class="no-drag px-5 py-2 rounded-full text-xs text-red-300 bg-red-400/10 hover:bg-red-400/20 transition-all"
            @click="confirmClose"
          >
            确定
          </button>
        </div>
      </div>
    </div>
    </div>
  </div>

  <!-- Context menu (outside main container to avoid layout interference) -->
  <div
    ref="contextMenuRef"
    v-if="contextMenu.visible"
    class="fixed z-[10000] rounded-xl overflow-hidden"
    :class="contextMenu.onImage ? 'w-[130px]' : 'w-[195px]'"
    style="background: rgba(var(--bg-panel), 0.95); backdrop-filter: blur(10px); border: 1px solid rgba(var(--border), 0.08);"
    :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    @pointerenter="onContextMenuEnter"
    @pointerleave="onContextMenuLeave"
  >
    <!-- Image context menu -->
    <template v-if="contextMenu.onImage">
      <button class="ctx-item ctx-first" @click="contextAction('copy')"><span>复制</span><span class="text-white/30 ml-4">Ctrl+C</span></button>
      <button class="ctx-item ctx-last" @click="contextAction('deleteNode')"><span>删除</span><span class="text-white/30 ml-4">Del</span></button>
    </template>
    <!-- Text context menu -->
    <template v-else>
      <button class="ctx-item ctx-first" @click="contextAction('copy')"><span>复制</span><span class="text-white/30 ml-4">Ctrl+C</span></button>
      <button class="ctx-item" @click="contextAction('paste')"><span>粘贴</span><span class="text-white/30 ml-4">Ctrl+V</span></button>
      <button class="ctx-item" @click="contextAction('pastePlain')"><span>粘贴纯文本</span><span class="text-white/30 ml-4">Ctrl+Shift+C</span></button>
      <div class="my-1 border-t border-white/8" />
      <button class="ctx-item" @click="contextAction('bold')"><span>加粗</span><span class="text-white/30 ml-4">Ctrl+B</span></button>
      <button class="ctx-item" @click="contextAction('strikethrough')"><span>删除线</span><span class="text-white/30 ml-4">Ctrl+D</span></button>
      <button class="ctx-item" @click="contextAction('underline')"><span>下划线</span><span class="text-white/30 ml-4">Ctrl+E</span></button>
      <div class="my-1 border-t border-white/8" />
      <button class="ctx-item" @click="contextAction('orderedList')"><span>有序列表</span><span class="text-white/30 ml-4">Ctrl+Shift+O</span></button>
      <button class="ctx-item" @click="contextAction('bulletList')"><span>无序列表</span><span class="text-white/30 ml-4">Ctrl+Shift+U</span></button>
      <button class="ctx-item ctx-last" @click="contextAction('taskList')"><span>待办事项</span><span class="text-white/30 ml-4">Ctrl+Shift+T</span></button>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { useNotesStore } from "@/stores/notes";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen, emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "@/composables/useToast";
import { useSettingsStore } from "@/stores/settings";
import EditorToolbar from "@/components/editor/EditorToolbar.vue";
import MilkdownEditor, { type ColorRange } from "@/components/editor/MilkdownEditor.vue";
import FindBar from "@/components/editor/FindBar.vue";
import CreateReminderModal from "@/components/layout/CreateReminderModal.vue";
import { useAutoSave } from "@/composables/useAutoSave";
import { useWindowClose } from "@/composables/useWindowClose";
import { useWordCount } from "@/composables/useWordCount";
import { useAutoHide } from "@/composables/useAutoHide";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";

const props = defineProps<{
  noteId: string;
}>();

const notesStore = useNotesStore();
const settingsStore = useSettingsStore();
const { toast } = useToast();
const editorRef = ref<InstanceType<typeof MilkdownEditor> | null>(null);

const title = ref("");
const content = ref("");
const colorRanges = ref<ColorRange[]>([]);
const isPreview = ref(false);
const showReminderPicker = ref(false);
function closeReminderModal() {
  showReminderPicker.value = false;
  nextTick(() => editorRef.value?.focus());
}
const pinned = ref(false);
const titleCompact = ref(false);

// Find bar
const showFind = ref(false);
const findBarRef = ref<InstanceType<typeof FindBar> | null>(null);
const findMatchCount = ref(0);
const findCurrentMatch = ref(0);

function scrollToTop() {
  const milkdownEl = document.querySelector(".milkdown-container");
  if (milkdownEl) milkdownEl.scrollTo({ top: 0, behavior: "smooth" });
}

// Context menu
const contextMenuRef = ref<HTMLElement>();
const contextMenu = ref({ visible: false, x: 0, y: 0, onImage: false });
let deleteImageSrc = "";
let contextMenuCloseHandler: (() => void) | null = null;

function onContextMenu(e: MouseEvent) {
  if (isPreview.value) return;
  // Don't show context menu on scrollbar
  const target = e.currentTarget as HTMLElement;
  if (target && e.offsetX >= target.clientWidth) return;
  // Detect if right-clicking on an image
  const clickTarget = e.target as HTMLElement;
  const isImage = clickTarget.tagName === "IMG";
  deleteImageSrc = isImage ? (clickTarget as HTMLImageElement).getAttribute("src") || "" : "";
  const menuWidth = isImage ? 130 : 195;
  // If menu goes beyond right edge, expand to the left
  const x = e.clientX + menuWidth > window.innerWidth ? e.clientX - menuWidth : e.clientX;
  // Initial position: show below cursor
  contextMenu.value = { visible: true, x: Math.max(0, x), y: e.clientY, onImage: isImage };
  // Adjust position after render if menu overflows
  nextTick(() => {
    const menuEl = contextMenuRef.value;
    if (menuEl) {
      const rect = menuEl.getBoundingClientRect();
      // Overflow bottom → show above cursor
      if (rect.bottom > window.innerHeight) {
        contextMenu.value.y = Math.max(0, e.clientY - rect.height);
      }
    }
  });
  // Remove previous close handler if any
  if (contextMenuCloseHandler) {
    document.removeEventListener("click", contextMenuCloseHandler);
    contextMenuCloseHandler = null;
  }
  const close = () => {
    contextMenu.value.visible = false;
    document.removeEventListener("click", close);
    contextMenuCloseHandler = null;
  };
  contextMenuCloseHandler = close;
  setTimeout(() => document.addEventListener("click", close), 0);
}

async function copyImageToClipboard(src: string) {
  try {
    await editorRef.value?.copyImage(src);
  } catch (err) {
    console.error("Failed to copy image:", err);
    toast("图片复制失败");
  }
}

async function contextAction(action: string) {
  contextMenu.value.visible = false;
  if (action === "copy") {
    // Check if we're copying an image
    if (contextMenu.value.onImage && deleteImageSrc) {
      await copyImageToClipboard(deleteImageSrc);
      return;
    }
    writeText(window.getSelection()?.toString() || "").catch(() => {});
    return;
  }
  if (action === "deleteNode") { editorRef.value?.execCommand("deleteImage", deleteImageSrc); return; }
  if (action === "paste" || action === "pastePlain") {
    readText().then((text) => {
      if (!text || !editorRef.value) return;
      editorRef.value.execCommand(action === "pastePlain" ? "pastePlain" : "paste");
    });
    return;
  }
  format(action);
}

// --- Composables ---
const { markDirty, saveImmediately } = useAutoSave({
  noteId: ref(props.noteId),
  title,
  content,
});

const { showCloseConfirm, handleClose, confirmClose, cancelClose } =
  useWindowClose({ saveImmediately });

const { displayWordCount } = useWordCount(content);
const autoHide = useAutoHide(props.noteId, '[data-window="editor"]');

// Register close callback for taskbar close when hidden
autoHide.setOnClose(async () => {
  await saveImmediately();
});

// Suppress auto-hide when hovering on context menu
function onContextMenuEnter() { autoHide.suppressAutoHide.value = true; }
function onContextMenuLeave() { autoHide.suppressAutoHide.value = false; }

// Re-enable alwaysOnTop after restore if editor was pinned
watch(autoHide.isHidden, async (hidden) => {
  if (!hidden && pinned.value) {
    await getCurrentWebviewWindow().setAlwaysOnTop(true);
  }
});

// Suppress auto-hide when pinned (always on top)
watch(pinned, (val) => {
  autoHide.autoHideEnabled.value = !val;
});

// --- Actions ---
function format(command: string) {
  editorRef.value?.execCommand(command);
}

function handleMinimize() {
  getCurrentWebviewWindow().minimize();
}

async function togglePin() {
  pinned.value = !pinned.value;
  await getCurrentWebviewWindow().setAlwaysOnTop(pinned.value);
}

async function handleScreenshot() {
  const config = settingsStore.screenshot;
  if (config.fixedArea && config.fixedWidth > 0 && config.fixedHeight > 0) {
    await captureFixedArea();
  } else {
    await createScreenshotOverlay();
  }
}

/** Capture a pre-configured fixed area (no overlay) */
async function captureFixedArea() {
  const config = settingsStore.screenshot;
  try {
    const path = await invoke<string>("capture_fixed_area", {
      noteId: props.noteId, x: Math.round(config.fixedX), y: Math.round(config.fixedY),
      width: Math.round(config.fixedWidth), height: Math.round(config.fixedHeight),
    });
    if (editorRef.value) editorRef.value.insertImageFromPath(path);
    toast("截图已保存");
  } catch (e) {
    console.error("Screenshot failed:", e);
    toast("截图失败");
  }
}

/** Create native Win32 overlay for screenshot selection.
 *  All coordinate handling happens in Rust — no webview DPI issues. */
async function createScreenshotOverlay() {
  try {
    const path = await invoke<string>("select_and_capture_screenshot", {
      noteId: props.noteId,
    });
    if (editorRef.value) editorRef.value.insertImageFromPath(path);
    toast("截图已保存");
  } catch (e) {
    // "Cancelled" is expected when user presses ESC
    if (e !== "Cancelled" && !String(e).includes("Cancelled")) {
      console.error("Screenshot failed:", e);
      toast("截图失败");
    }
  }
}

let unlistenScreenshot: (() => void) | null = null;
let unlistenScreenshotCapture: (() => void) | null = null;
let milkdownScrollHandler: (() => void) | null = null;
let unlistenResize: (() => void) | null = null;

// Screenshot shortcut
const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);
function normalizeKey(e: KeyboardEvent): string {
  if (MODIFIER_KEYS.has(e.key)) return "";
  if (e.shiftKey && (e.ctrlKey || e.metaKey)) return `shift-mod-${e.key.toLowerCase()}`;
  if (e.ctrlKey || e.metaKey) return `mod-${e.key.toLowerCase()}`;
  if (e.altKey) return `alt-${e.key.toLowerCase()}`;
  return "";
}

function handleScreenshotShortcut(e: KeyboardEvent) {
  const configured = settingsStore.screenshot.shortcut;
  if (!configured) return;
  if (normalizeKey(e) !== configured) return;
  e.preventDefault();
  handleScreenshot();
}

function handleFindShortcut(e: KeyboardEvent) {
  if (e.ctrlKey && !e.shiftKey && !e.altKey && e.key.toLowerCase() === "f") {
    e.preventDefault();
    // Hide float menu when opening find bar
    editorRef.value?.hideFloatMenu();
    // Get selected text from editor
    const selectedText = editorRef.value?.getSelectedText() || "";
    showFind.value = true;
    nextTick(() => {
      if (selectedText) {
        findBarRef.value?.setQuery(selectedText);
        onFindSearch(selectedText);
      }
      findBarRef.value?.focus();
    });
  }
}

function onFindSearch(query: string) {
  if (!query) {
    editorRef.value?.clearFind();
    findMatchCount.value = 0;
    findCurrentMatch.value = 0;
    return;
  }
  findMatchCount.value = editorRef.value?.find(query) ?? 0;
  findCurrentMatch.value = 0;
}

function onFindNext() {
  if (!findMatchCount.value) return;
  editorRef.value?.findNext();
  findCurrentMatch.value = (findCurrentMatch.value + 1) % findMatchCount.value;
}

function onFindPrev() {
  if (!findMatchCount.value) return;
  editorRef.value?.findPrev();
  findCurrentMatch.value = (findCurrentMatch.value - 1 + findMatchCount.value) % findMatchCount.value;
}

function onFindClose() {
  showFind.value = false;
  editorRef.value?.clearFind();
  findMatchCount.value = 0;
  findCurrentMatch.value = 0;
  editorRef.value?.focus();
}

function onReplace(replaceText: string) {
  if (!findMatchCount.value) return;
  const result = editorRef.value?.replaceCurrent(replaceText);
  if (result) {
    findMatchCount.value = result.count;
    findCurrentMatch.value = result.index;
  }
}

function onReplaceAll(replaceText: string) {
  if (!findMatchCount.value) return;
  const count = editorRef.value?.replaceAll(replaceText) ?? 0;
  findMatchCount.value = 0;
  findCurrentMatch.value = 0;
  if (count > 0) {
    toast(`已替换 ${count} 处`);
  }
}

onMounted(async () => {
  try {
    const note = await notesStore.getNote(props.noteId);
    if (note) {
      title.value = note.title;
      content.value = note.content;
      try { colorRanges.value = JSON.parse(note.color_ranges || "[]"); } catch { colorRanges.value = []; }
    }
  } catch (err) {
    console.error("Failed to load note:", err);
    toast("笔记加载失败");
    getCurrentWebviewWindow().close();
  }

  // Listen for screenshot events targeted at this note
  unlistenScreenshot = await listen<{ noteId: string; path: string }>("screenshot:captured", (event) => {
    if (event.payload.noteId === props.noteId && editorRef.value) {
      editorRef.value.insertImageFromPath(event.payload.path);
      toast("截图已保存");
    }
  });

  // Listen for capture requests from ScreenshotWindow (overlay closed, now capture)
  unlistenScreenshotCapture = await listen<{ noteId: string; x: number; y: number; width: number; height: number }>(
    "screenshot:do-capture",
    async (event) => {
      if (event.payload.noteId !== props.noteId) return;
      try {
        const path = await invoke<string>("capture_screenshot", {
          noteId: props.noteId,
          x: Math.round(event.payload.x),
          y: Math.round(event.payload.y),
          width: Math.round(event.payload.width),
          height: Math.round(event.payload.height),
        });
        if (editorRef.value) editorRef.value.insertImageFromPath(path);
        toast("截图已保存");
      } catch (e) {
        console.error("Screenshot capture failed:", e);
      }
    }
  );

  // Screenshot shortcut
  document.addEventListener("keydown", handleScreenshotShortcut);
  // Find shortcut (Ctrl+F)
  document.addEventListener("keydown", handleFindShortcut);

  // Save window dimensions on resize (logical pixels)
  let resizeTimer: ReturnType<typeof setTimeout> | null = null;
  let firstResize = true;
  unlistenResize = await getCurrentWebviewWindow().listen('tauri://resize', async () => {
    if (firstResize) { firstResize = false; return; }
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(async () => {
      try {
        const win = getCurrentWebviewWindow();
        const size = await win.outerSize();
        const factor = await win.scaleFactor();
        const w = Math.round(size.width / factor);
        const h = Math.round(size.height / factor);
        localStorage.setItem(`editor-size-${props.noteId}`, JSON.stringify({ width: w, height: h }));
      } catch {}
    }, 500);
  });

  // Monitor editor scroll for title compact mode
  nextTick(() => {
    const milkdownEl = document.querySelector(".milkdown-container");
    if (milkdownEl) {
      const onScroll = () => {
        titleCompact.value = milkdownEl.scrollTop > 10;
      };
      milkdownEl.addEventListener("scroll", onScroll);
      milkdownScrollHandler = () => milkdownEl.removeEventListener("scroll", onScroll);
    }
  });

});

onUnmounted(() => {
  if (unlistenScreenshot) { unlistenScreenshot(); unlistenScreenshot = null; }
  if (unlistenScreenshotCapture) { unlistenScreenshotCapture(); unlistenScreenshotCapture = null; }
  if (milkdownScrollHandler) { milkdownScrollHandler(); milkdownScrollHandler = null; }
  if (unlistenResize) { unlistenResize(); unlistenResize = null; }
  document.removeEventListener("keydown", handleScreenshotShortcut);
  document.removeEventListener("keydown", handleFindShortcut);
});

// Persist color_ranges on change (debounced)
let colorSaveTimer: ReturnType<typeof setTimeout> | null = null;
function onColorRangesChange(ranges: ColorRange[]) {
  colorRanges.value = ranges;
  if (colorSaveTimer) clearTimeout(colorSaveTimer);
  colorSaveTimer = setTimeout(() => {
    notesStore.updateNote({ id: props.noteId, color_ranges: JSON.stringify(ranges) });
  }, 500);
}
</script>

<style>
.ctx-item {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
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
</style>
