<template>
  <div
    class="h-full flex flex-col rounded-3xl overflow-hidden"
    style="background: rgba(var(--bg-primary), 0.92); backdrop-filter: blur(10px);"
  >
    <!-- Title bar -->
    <div class="drag-region flex items-center justify-between px-4 pt-4 pb-2">
      <h1 class="text-sm font-semibold text-white/80">设置</h1>
      <div class="flex items-center gap-1">
        <button
          class="no-drag flex items-center justify-center w-6 h-6 rounded-full text-white/30 hover:text-red-400 hover:bg-red-400/10 transition-all"
          @click="getCurrentWebviewWindow().close()"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex gap-0.5 px-4 mb-3">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        tabindex="-1"
        class="no-drag px-3 py-1 rounded-full text-xs font-medium transition-all duration-200"
        :class="activeTab === tab.key ? 'bg-white/15 text-white' : 'text-white/40 hover:text-white/70'"
        @click="activeTab = tab.key"
      >{{ tab.label }}</button>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-4 pb-4">

      <!-- ===== GENERAL ===== -->
      <div v-if="activeTab === 'general'" class="space-y-3 pt-2">
        <div class="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-white/5 transition-colors">
          <span class="text-xs text-white/70">开机自启</span>
          <button
            class="no-drag relative w-10 h-5 rounded-full transition-colors duration-200"
            :class="settingsStore.autostart ? 'bg-blue-500/60' : 'bg-white/10'"
            @click="toggleAutostart"
          >
            <div
              class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform duration-200"
              :class="settingsStore.autostart ? 'translate-x-5' : 'translate-x-0.5'"
            />
          </button>
        </div>
        <div class="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-white/5 transition-colors">
          <span class="text-xs text-white/70">检查更新</span>
          <div class="flex gap-1">
            <button
              class="no-drag px-3 py-1 rounded-md text-[11px] transition-all"
              :class="settingsStore.checkUpdate ? 'bg-blue-500/40 text-white/90' : 'bg-white/8 text-white/40 hover:text-white/60'"
              @click="settingsStore.saveCheckUpdate(true)"
            >是</button>
            <button
              class="no-drag px-3 py-1 rounded-md text-[11px] transition-all"
              :class="!settingsStore.checkUpdate ? 'bg-blue-500/40 text-white/90' : 'bg-white/8 text-white/40 hover:text-white/60'"
              @click="settingsStore.saveCheckUpdate(false)"
            >否</button>
          </div>
        </div>
      </div>

      <!-- ===== SHORTCUTS ===== -->
      <div v-if="activeTab === 'shortcuts'" class="space-y-0.5">
        <template v-for="(item, idx) in shortcutList" :key="item.id">
          <div class="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-white/5 transition-colors">
            <span class="text-xs text-white/70">{{ item.label }}</span>
            <button
              class="no-drag px-2.5 py-1 rounded-md text-[11px] transition-all min-w-[80px] text-center outline-none focus:outline-none"
              :class="rebindTarget === item.id
                ? 'text-white/60 border border-white/10'
                : 'bg-white/8 text-white/60 hover:bg-white/12 hover:text-white/80 border border-white/10'"
              @click="startRebind(item.id)"
            >
              {{ rebindTarget === item.id ? '按下新快捷键...' : formatKey(draftShortcuts[item.id]) }}
            </button>
          </div>
          <!-- Insert screenshot shortcut after heading (index 2) -->
          <div v-if="idx === 2" class="flex items-center justify-between py-1.5 px-2 rounded-lg hover:bg-white/5 transition-colors">
            <span class="text-xs text-white/70">截图</span>
            <button
              class="no-drag px-2.5 py-1 rounded-md text-[11px] transition-all min-w-[80px] text-center outline-none focus:outline-none"
              :class="rebindTarget === 'screenshot'
                ? 'text-white/60 border border-white/10'
                : 'bg-white/8 text-white/60 hover:bg-white/12 hover:text-white/80 border border-white/10'"
              @click="startRebind('screenshot')"
            >
              {{ rebindTarget === 'screenshot' ? '按下新快捷键...' : (formatKey(draftScreenshot.shortcut) || '未设置') }}
            </button>
          </div>
        </template>
        <div v-if="rebindTarget" class="flex items-center gap-2 mt-1 px-2">
          <span class="text-[10px] text-white/30">按 ESC 取消</span>
          <span v-if="conflictWarning" class="text-[10px] text-amber-400">{{ conflictWarning }}</span>
        </div>
      </div>

      <!-- ===== APPEARANCE ===== -->
      <div v-if="activeTab === 'appearance'" class="space-y-4 pt-0">
        <!-- Theme selector -->
        <div>
          <span class="text-xs text-white/70 block mb-1.5">主题皮肤</span>
          <div class="grid grid-cols-2 gap-1.5">
            <button
              v-for="theme in THEMES"
              :key="theme.id"
              class="no-drag relative rounded-xl p-2.5 transition-all duration-200 border overflow-hidden"
              :class="draftAppearance.theme === theme.id
                ? 'border-white/25 bg-white/10'
                : 'border-white/6 bg-white/3 hover:bg-white/6 hover:border-white/10'"
              @click="selectTheme(theme.id)"
            >
              <!-- Theme preview -->
              <div class="flex gap-1 mb-2">
                <div
                  v-for="(color, ci) in theme.colors"
                  :key="ci"
                  class="w-5 h-5 rounded-full border border-white/10"
                  :style="{ background: color }"
                />
              </div>
              <div class="text-left">
                <div class="text-[11px] font-medium" :class="draftAppearance.theme === theme.id ? 'text-white/90' : 'text-white/60'">{{ theme.label }}</div>
                <div class="text-[9px] text-white/30">{{ theme.desc }}</div>
              </div>
              <!-- Active indicator -->
              <div v-if="draftAppearance.theme === theme.id" class="absolute top-2 right-2 w-2 h-2 rounded-full" style="background: rgb(var(--accent))" />
              <!-- Hello Kitty bow decoration -->
              <svg v-if="theme.id === 'hello-kitty'" class="absolute top-1.5 right-1.5 w-4 h-4 opacity-40" viewBox="0 0 24 24" fill="none">
                <path d="M12 4C8 4 4 7 4 12s4 8 8 8 8-3 8-8-4-8-8-8z" fill="#ff8fab" opacity="0.3"/>
                <path d="M8 6c-2-3-5-4-6-2s0 5 2 7M16 6c2-3 5-4 6-2s0 5-2 7" stroke="#ff8fab" stroke-width="1.5" fill="none"/>
                <circle cx="9" cy="12" r="1" fill="#ff6b91"/>
                <circle cx="15" cy="12" r="1" fill="#ff6b91"/>
                <ellipse cx="12" cy="14" rx="1.5" ry="1" fill="#ff6b91" opacity="0.6"/>
              </svg>
            </button>
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-white/70">笔记页 · 透明度</span>
            <span class="text-xs text-white/40">{{ draftAppearance.mainOpacity }}%</span>
          </div>
          <input
            type="range" min="60" max="100" step="1"
            class="settings-slider w-full"
            v-model.number="draftAppearance.mainOpacity"
            @input="previewAppearance"
            @change="commitAppearance"
          />
        </div>
        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-white/70">编辑页 · 透明度</span>
            <span class="text-xs text-white/40">{{ draftAppearance.editorOpacity }}%</span>
          </div>
          <input
            type="range" min="60" max="100" step="1"
            class="settings-slider w-full"
            v-model.number="draftAppearance.editorOpacity"
            @input="previewAppearance"
            @change="commitAppearance"
          />
        </div>
      </div>

      <!-- ===== SCREENSHOT ===== -->
      <div v-if="activeTab === 'screenshot'" class="space-y-4 pt-2">
        <!-- Width / Height -->
        <div>
          <span class="text-xs text-white/70 block mb-2">固定截图尺寸</span>
          <div class="flex gap-2">
            <div class="flex-1">
              <input
                :value="draftScreenshot.fixedWidth"
                type="number" placeholder="宽度"
                readonly
                class="no-drag w-full px-3 py-1.5 rounded-lg text-xs outline-none text-center"
                :class="!draftScreenshot.fixedArea ? 'text-white/30 cursor-not-allowed' : 'text-white/90 cursor-default'"
                style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08);"
              />
            </div>
            <div class="flex-1">
              <input
                :value="draftScreenshot.fixedHeight"
                type="number" placeholder="高度"
                readonly
                class="no-drag w-full px-3 py-1.5 rounded-lg text-xs outline-none text-center"
                :class="!draftScreenshot.fixedArea ? 'text-white/30 cursor-not-allowed' : 'text-white/90 cursor-default'"
                style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08);"
              />
            </div>
          </div>
        </div>

        <!-- Fixed area toggle -->
        <div class="flex items-center justify-between">
          <span class="text-xs text-white/70">固定区域截图</span>
          <button
            class="no-drag relative w-10 h-5 rounded-full transition-colors duration-200"
            :class="draftScreenshot.fixedArea ? 'bg-blue-500/60' : 'bg-white/10'"
            @click="toggleFixedArea"
          >
            <div
              class="absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform duration-200"
              :class="draftScreenshot.fixedArea ? 'translate-x-5' : 'translate-x-0.5'"
            />
          </button>
        </div>

        <!-- Select area button -->
        <button
          class="no-drag w-full py-2 rounded-lg text-xs transition-all"
          :class="!draftScreenshot.fixedArea ? 'text-white/20 cursor-not-allowed' : 'text-white/60 hover:text-white/80'"
          style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08);"
          :disabled="!draftScreenshot.fixedArea"
          @click="selectArea"
        >
          选取截图区域
        </button>
      </div>

      <!-- ===== ABOUT ===== -->
      <div v-if="activeTab === 'about'" class="flex flex-col h-full items-center">
        <div class="flex-1 flex flex-col items-center justify-center -mt-8">
          <img src="@/assets/icon.png" alt="FlowNote" class="w-16 h-16 rounded-2xl mb-4" />
          <h2 class="text-2xl font-bold text-white/90 mb-1">FlowNote</h2>
          <p class="text-xs text-white/40 mb-4">玲儿响叮当</p>
          <p class="text-xs text-white/30 mb-4">v{{ currentVersion }}</p>
          <!-- Update section -->
          <div v-if="updateInfo" class="text-center mb-4">
            <p class="text-xs text-green-400/80 mb-2">发现新版本 v{{ updateInfo.version }}</p>
            <button
              class="no-drag px-4 py-1.5 rounded-full text-xs text-white/70 bg-white/10 hover:bg-white/15 transition-all"
              @click="handleDownload"
            >
              {{ downloading ? '下载中...' : '下载更新' }}
            </button>
          </div>
          <button
            v-else
            class="no-drag px-4 py-1.5 rounded-full text-xs text-white/40 hover:text-white/60 hover:bg-white/8 transition-all"
            :class="checking ? 'cursor-not-allowed opacity-50' : ''"
            :disabled="checking"
            @click="checkForUpdate"
          >
            {{ checking ? '检查中...' : '检查更新' }}
          </button>
        </div>
        <p class="text-[10px] text-white/25 pb-3">Copyright © 2026 <span @click="openWebsite" class="cursor-pointer hover:text-white/50 transition-colors">ChenYue</span>. All Rights Reserved.</p>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { useSettingsStore, SHORTCUT_LABELS, THEMES, type ThemeId } from "@/stores/settings";
import { useToast } from "@/composables/useToast";

const settingsStore = useSettingsStore();
const { toast } = useToast();

const activeTab = ref<"general" | "shortcuts" | "appearance" | "screenshot" | "about">("general");
const tabs = [
  { key: "general" as const, label: "通用" },
  { key: "shortcuts" as const, label: "快捷键" },
  { key: "appearance" as const, label: "外观" },
  { key: "screenshot" as const, label: "截图" },
  { key: "about" as const, label: "关于" },
];

// --- Shortcuts ---
const shortcutList = computed(() =>
  Object.keys(settingsStore.shortcuts).map((id) => ({
    id,
    label: SHORTCUT_LABELS[id] || id,
  }))
);

const draftShortcuts = reactive<Record<string, string>>({ ...settingsStore.shortcuts });
const rebindTarget = ref<string | null>(null);
const conflictWarning = ref("");

function startRebind(id: string) {
  rebindTarget.value = id;
  conflictWarning.value = "";
}

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

function normalizeKey(e: KeyboardEvent): string {
  if (e.key === "Escape") return "";
  if (MODIFIER_KEYS.has(e.key)) return "";
  let normalized = "";
  if (e.shiftKey && (e.ctrlKey || e.metaKey)) {
    normalized = `shift-mod-${e.key.toLowerCase()}`;
  } else if (e.altKey && e.shiftKey) {
    normalized = `shift-alt-${e.key.toLowerCase()}`;
  } else if (e.ctrlKey || e.metaKey) {
    normalized = `mod-${e.key.toLowerCase()}`;
  } else if (e.altKey) {
    normalized = `alt-${e.key.toLowerCase()}`;
  } else {
    return "";
  }
  return normalized;
}

function formatKey(key: string): string {
  if (!key) return "";
  return key
    .replace("shift-mod-", "Ctrl+Shift+")
    .replace("shift-alt-", "Shift+Alt+")
    .replace("mod-", "Ctrl+")
    .replace("alt-", "Alt+")
    .split("-")
    .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
    .join("+");
}

function onSettingsKeydown(e: KeyboardEvent) {
  if (!rebindTarget.value) return;
  e.preventDefault();
  e.stopPropagation();

  // ESC cancels rebind
  if (e.key === "Escape") {
    rebindTarget.value = null;
    conflictWarning.value = "";
    return;
  }

  // Ignore modifier-only keys
  if (MODIFIER_KEYS.has(e.key)) return;

  const normalized = normalizeKey(e);
  if (!normalized) return;

  const target = rebindTarget.value;

  // Check conflict — reject, not allowed to override
  const conflict = settingsStore.findConflict(target, normalized);
  if (conflict) {
    toast(`与「${SHORTCUT_LABELS[conflict] || conflict}」快捷键冲突`);
    return;
  }

  if (target === "screenshot") {
    draftScreenshot.shortcut = normalized;
    rebindTarget.value = null;
    conflictWarning.value = "";
    settingsStore.saveScreenshot({ shortcut: normalized });
  } else {
    draftShortcuts[target] = normalized;
    rebindTarget.value = null;
    conflictWarning.value = "";
    settingsStore.saveShortcuts({ ...draftShortcuts });
  }
}

// --- Appearance ---
const draftAppearance = reactive({
  mainOpacity: settingsStore.appearance.mainOpacity,
  editorOpacity: settingsStore.appearance.editorOpacity,
  theme: settingsStore.appearance.theme as ThemeId,
});

function previewAppearance() {
  // Live preview via CSS vars
  document.documentElement.style.setProperty("--main-opacity", String(draftAppearance.mainOpacity / 100));
  document.documentElement.style.setProperty("--editor-opacity", String(draftAppearance.editorOpacity / 100));
  // Preview theme
  if (draftAppearance.theme && draftAppearance.theme !== "default") {
    document.documentElement.setAttribute("data-theme", draftAppearance.theme);
  } else {
    document.documentElement.removeAttribute("data-theme");
  }
}

function commitAppearance() {
  settingsStore.saveAppearance({
    mainOpacity: draftAppearance.mainOpacity,
    editorOpacity: draftAppearance.editorOpacity,
    theme: draftAppearance.theme,
  });
}

function selectTheme(themeId: ThemeId) {
  draftAppearance.theme = themeId;
  previewAppearance();
  commitAppearance();
}

// --- Autostart ---
async function toggleAutostart() {
  try {
    await settingsStore.saveAutostart(!settingsStore.autostart);
  } catch (e: any) {
    toast(e?.message || "设置失败");
  }
}

// --- Screenshot ---
const draftScreenshot = reactive({
  fixedX: settingsStore.screenshot.fixedX,
  fixedY: settingsStore.screenshot.fixedY,
  fixedWidth: settingsStore.screenshot.fixedWidth,
  fixedHeight: settingsStore.screenshot.fixedHeight,
  fixedArea: settingsStore.screenshot.fixedArea,
  shortcut: settingsStore.screenshot.shortcut,
});

function toggleFixedArea() {
  draftScreenshot.fixedArea = !draftScreenshot.fixedArea;
  settingsStore.saveScreenshot({
    fixedX: draftScreenshot.fixedX,
    fixedY: draftScreenshot.fixedY,
    fixedWidth: draftScreenshot.fixedWidth,
    fixedHeight: draftScreenshot.fixedHeight,
    fixedArea: draftScreenshot.fixedArea,
    shortcut: draftScreenshot.shortcut,
  });
}


// --- Area selection (native Win32 overlay) ---
async function selectArea() {
  try {
    const [x, y, width, height] = await invoke<[number, number, number, number]>("select_screen_area");
    draftScreenshot.fixedX = Math.round(x);
    draftScreenshot.fixedY = Math.round(y);
    draftScreenshot.fixedWidth = Math.round(width);
    draftScreenshot.fixedHeight = Math.round(height);
    settingsStore.saveScreenshot({
      fixedX: Math.round(x), fixedY: Math.round(y),
      fixedWidth: Math.round(width), fixedHeight: Math.round(height),
      fixedArea: draftScreenshot.fixedArea, shortcut: draftScreenshot.shortcut,
    });
    toast(`已选取 ${width} × ${height}`);
  } catch (e) {
    if (e !== "Cancelled" && !String(e).includes("Cancelled") && !String(e).includes("too small")) {
      console.error("Area selection failed:", e);
      toast("选取区域失败");
    }
  }
}

// --- Website ---
async function openWebsite() {
  try {
    await open("https://www.cyue.top/");
  } catch (e) {
    console.error("Failed to open website:", e);
    toast("打开网站失败");
  }
}

// --- Update ---
import { getName, getVersion } from "@tauri-apps/api/app";
const currentVersion = ref("");
(async () => { currentVersion.value = await getVersion(); })();
const checking = ref(false);
const downloading = ref(false);
const updateInfo = ref<{ version: string; download_url: string } | null>(null);

const MANIFEST_URL = "https://raw.githubusercontent.com/ChenYueC/FlowNote/main/latest.json";

async function checkForUpdate() {
  checking.value = true;
  try {
    const info = await invoke<{ need_update: boolean; version: string; download_url: string }>("check_update", {
      manifestUrl: MANIFEST_URL,
    });
    if (info.need_update) {
      updateInfo.value = { version: info.version, download_url: info.download_url };
    } else {
      toast("当前已是最新版本");
    }
  } catch (e) {
    console.error("Check update failed:", e);
    toast("检查更新失败");
  } finally {
    checking.value = false;
  }
}

async function handleDownload() {
  if (!updateInfo.value || downloading.value) return;
  downloading.value = true;
  try {
    await invoke("download_and_install", { url: updateInfo.value.download_url });
  } catch (e) {
    console.error("Download failed:", e);
    toast("下载失败");
    downloading.value = false;
  }
}

// --- Lifecycle ---
onMounted(async () => {
  document.addEventListener("keydown", onSettingsKeydown, true);
  // 等待 store 加载完成后同步配置到 draft 变量
  await settingsStore.load();
  syncDraftFromStore();
});

// 同步 store 配置到 draft 变量
function syncDraftFromStore() {
  // 同步快捷键
  Object.assign(draftShortcuts, settingsStore.shortcuts);

  // 同步外观
  draftAppearance.mainOpacity = settingsStore.appearance.mainOpacity;
  draftAppearance.editorOpacity = settingsStore.appearance.editorOpacity;
  draftAppearance.theme = settingsStore.appearance.theme;

  // 同步截图配置
  Object.assign(draftScreenshot, settingsStore.screenshot);
}

onUnmounted(() => {
  document.removeEventListener("keydown", onSettingsKeydown, true);
});
</script>

<style scoped>
.settings-slider {
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.1);
  outline: none;
}
.settings-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: background 0.15s;
}
.settings-slider::-webkit-slider-thumb:hover {
  background: rgba(255, 255, 255, 0.9);
}
input[type="number"]::-webkit-inner-spin-button,
input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
input[type="number"] {
  -moz-appearance: textfield;
}
</style>
