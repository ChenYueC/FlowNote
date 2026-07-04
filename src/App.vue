<template>
  <MainWindow v-if="windowType === 'main'" />
  <EditorWindow
    v-else-if="windowType === 'editor' && noteId"
    :noteId="noteId"
  />
  <FloatingNoteWindow
    v-else-if="windowType === 'floating' && noteId && windowId"
    :noteId="noteId"
    :windowId="windowId"
  />
  <ScreenshotWindow
    v-else-if="windowType === 'screenshot' && noteId"
    :noteId="noteId"
    @close="handleScreenshotClose"
    @captured="handleScreenshotCaptured"
  />
  <ReminderBubble
    v-else-if="windowType === 'reminder' && reminderId"
    :reminderId="reminderId"
  />
  <EdgeDock
    v-else-if="windowType === 'edgedock'"
    :edge="edge"
  />
  <EditorDock
    v-else-if="windowType === 'editor-dock'"
    :edge="edge"
    :dockId="dockId"
  />
  <ImagePreviewWindow
    v-else-if="windowType === 'image-preview'"
  />
  <TrayMenu
    v-else-if="windowType === 'tray-menu'"
  />
  <SettingsWindow
    v-else-if="windowType === 'settings'"
  />
  <UpdateDialog
    v-else-if="windowType === 'update-dialog'"
  />
  <AreaSelectWindow
    v-else-if="windowType === 'area-select'"
  />
  <MainWindow v-else />
  <Toast :message="toastMessage" :visible="toastVisible" />
</template>

<script setup lang="ts">
import MainWindow from "@/components/layout/MainWindow.vue";
import EditorWindow from "@/components/layout/EditorWindow.vue";
import FloatingNoteWindow from "@/components/layout/FloatingNoteWindow.vue";
import ScreenshotWindow from "@/components/layout/ScreenshotWindow.vue";
import ReminderBubble from "@/components/layout/ReminderBubble.vue";
import EdgeDock from "@/components/layout/EdgeDock.vue";
import EditorDock from "@/components/layout/EditorDock.vue";
import ImagePreviewWindow from "@/components/layout/ImagePreviewWindow.vue";
import TrayMenu from "@/components/layout/TrayMenu.vue";
import SettingsWindow from "@/components/layout/SettingsWindow.vue";
import UpdateDialog from "@/components/layout/UpdateDialog.vue";
import AreaSelectWindow from "@/components/layout/AreaSelectWindow.vue";
import Toast from "@/components/ui/Toast.vue";
import { useToast } from "@/composables/useToast";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const { message: toastMessage, visible: toastVisible } = useToast();

const params = new URLSearchParams(window.location.search);
const windowType = params.get("window") || "main";
const noteId = params.get("note_id") || "";
const windowId = params.get("window_id") || "";
const reminderId = params.get("reminder_id") || "";
const edge = (params.get("edge") || "top") as "left" | "right" | "top";
const dockId = params.get("dock_id") || "";

async function handleScreenshotClose() {
  const window = getCurrentWebviewWindow();
  await window.close();
}

async function handleScreenshotCaptured(_path: string) {
  // Screenshot saved, window auto-closes after delay
}
</script>
