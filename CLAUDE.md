# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Prerequisites

- **Node.js** (LTS) + **pnpm** (package manager)
- **Rust** toolchain (for Tauri backend)
- **Windows** — the app targets Windows exclusively (`winreg` for auto-start, NSIS installer, Windows registry paths)

## Build & Run

```bash
pnpm install                    # Install dependencies (requires Node.js + Rust)
pnpm tauri dev                  # Start dev mode (Vite HMR + Rust hot-reload)
pnpm tauri build                # Production build + NSIS installer
cargo check                     # Rust type-check only (faster than full build)
npx vite build                  # Frontend-only build (skip TypeScript check)
```

NSIS installer configured in `tauri.conf.json` under `bundle.windows.nsis`: Chinese language (`languages: ["SimpChinese"]`), custom installer icon (`installerIcon: "icons/icon.ico"`). Build script in `package.json` uses `vite build` (skips TypeScript check for faster builds).

## Architecture

Tauri v2 multi-window desktop app. Window types routed via URL query params (`?window=main|editor|floating|screenshot|reminder|image-preview|settings`). `App.vue` reads `window.location.search` and renders the matching layout component.

**Rust side** (`src-tauri/src/`):
- `lib.rs` — App setup, plugin registration, tray (left-click to show, no menu), global shortcut (Alt+Space), background scheduler start, command registration
- `db/mod.rs` — SQLite via `sqlx`, auto-migration on startup. Tables: `notes`, `workspaces`, `images`, `floating_windows`, `reminders`. DB path uses `app.path().app_local_data_dir()` (Local)
- `commands/` — All `#[tauri::command]` functions exposed to frontend via `invoke()`. Modules: `app` (`exit_app`, `hide_main_window`, `open_file`, `load_settings`, `save_settings`, `get_autostart`, `set_autostart`), `floating`, `images`, `notes`, `reminders`, `screenshot`. `get_autostart`/`set_autostart` use `winreg` crate to read/write Windows Registry for auto-start on boot.
- `models/` — Serde structs mapping to DB rows. `timeline.rs` has `TimelineItem` for unified note+reminder timeline
- `services/scheduler.rs` — Background task polling reminders every 2s, emits `reminder:due` events (does NOT auto-complete; frontend popup handles user action)
- **Tauri plugins**: `opener`, `dialog`, `fs`, `shell`, `global-shortcut`, `notification`, `clipboard-manager`

**Vue side** (`src/`):
- `App.vue` — Window router based on URL params, hosts global `<Toast>` component
- `stores/notes.ts` — Pinia store: note CRUD, workspace switching (DB-backed), timeline with pagination (`list_timeline_items` returns unified note+reminder data), soft-delete (archived=1), favorite toggle
- `stores/windows.ts` — Pinia store: floating window CRUD + state
- `stores/reminders.ts` — Pinia store: reminder CRUD with backend filtering/sorting. `loadReminders(filter)` passes tab filter to backend, returns `ReminderListResponse` with filtered list + counts for all tabs. Actions: `createReminder`, `completeReminder`, `cancelReminder`, `snoozeReminder`, `deleteReminder` (soft delete)
- `stores/settings.ts` — Pinia store: user preferences persisted via Rust commands (`load_settings`/`save_settings`) to `settings.json` in app data dir. Every save writes the entire settings object. Cross-window sync via Tauri event (`settings:updated`). Sections: `shortcuts` (commandId→key map), `appearance` (mainOpacity/editorOpacity 60–100), `screenshot` (fixedWidth/fixedHeight/fixedArea/shortcut), `autostart` (boolean, via `get_autostart`/`set_autostart` commands). CSS vars `--main-opacity`/`--editor-opacity` applied on load and change.
- `composables/useToast.ts` — Shared toast state (module-level refs), `toast(msg, duration)` function, shake animation
- `composables/useDragDetach.ts` — Long-press (1s) drag ghost → floating window
- `composables/useAutoHide.ts` — Edge snap + auto-shrink for floating windows. `getMonitorInfo()` caches monitor data permanently
- `composables/useEdgeDock.ts` — Main window edge dock: state machine (`visible`/`hiding`/`docked`/`restoring`), pointer tracking, hide to screen edge with 350ms animation, dock icon, restore on hover. `getMonitorInfo()` caches permanently. On focus-lost: pauses CSS animations (`.window-unfocused` class), clears 200ms hover-check interval. On focus-gained: resumes interval. Pointer staleness detection: if no `pointermove` for 500ms, treats pointer as outside window.
- `composables/useAutoSave.ts` — Editor save logic: 500ms debounce, dirty state tracking, 3s safety interval, `saveImmediately()` for close-before-save
- `composables/useWindowClose.ts` — Editor close flow: confirmation modal, save-before-close, emits `editor:closed` event, cleans up `image-preview-*` localStorage entries, destroys `image-preview-*` windows, window destroy
- `composables/useWordCount.ts` — Character count: strips image Markdown and syntax, returns `"{count} 字"`
- `composables/useCommandRegistry.ts` — Singleton command handler registry. `registerCommand(id, handler)`, `executeCommand(id)`. Stable references, never rebuilt. Used by editor shortcuts and toolbar.
- `composables/useEditorShortcuts.ts` — Three-layer command pattern: `KeyEvent → normalize → commandId → handlerRegistry`. Reads shortcut bindings from settings store, watches for `settings:updated` Tauri event to rebuild shortcutMap. Non-configurable keys (Tab/Shift+Tab/Enter) handled directly. Keymap plugin inserted at beginning of plugin array (before Milkdown's built-in keymaps) to ensure custom handlers take priority.
- `composables/useEditorSync.ts` — Bidirectional modelValue ↔ Milkdown sync with focus protection and feedback loop prevention
- `composables/useImagePaste.ts` — (legacy, logic moved to MilkdownEditor capture-phase paste handler)
- `components/layout/` — MainWindow, EditorWindow, FloatingNoteWindow, ScreenshotWindow, ReminderBubble, ReminderView, CreateReminderModal, EdgeDock, ImagePreviewWindow, SettingsWindow, AreaSelectWindow
- `components/notes/` — NoteList, NoteCard, NoteSearch, TimelineView, ReminderCard
- `components/editor/` — MilkdownEditor (with image path resolution, link detection, keyboard shortcuts), EditorToolbar
- `components/ui/` — GlassContainer, IconButton, Toast

## Multi-Window Pattern

Windows created with `new WebviewWindow(label, { url: 'index.html?window=xxx&...' })`. URL params determine component in `App.vue`. Window types: `main`, `editor`, `floating`, `screenshot`, `reminder`, `edgedock`, `image-preview`, `settings`, `area-select`. New window label patterns must be added to `capabilities/default.json` `"windows"` array. All windows use `decorations: false, transparent: true, shadow: false`.

Main window uses `skipTaskbar: true` (appears only in system tray, not taskbar). Window position and size are saved to `localStorage` on move/resize (1s debounce) and restored on startup. Editor window dimensions are saved per-note to `localStorage` key `editor-size-{noteId}` (logical pixels, converted from physical via `scaleFactor`). Restored on open, removed on note deletion.

## ACL / Permissions (Critical)

Tauri v2 uses an ACL system. `core:default` does NOT include many window/webview operations. The following are explicitly added in `capabilities/default.json`:

- `core:window:allow-close` / `allow-destroy` / `allow-hide` / `allow-show` / `allow-minimize`
- `core:window:allow-set-always-on-top` / `allow-set-position` / `allow-set-size` / `allow-set-focus` / `allow-start-dragging`
- `core:webview:allow-create-webview-window` / `allow-create-webview`
- `fs:default`, `fs:allow-temp-write`, `fs:allow-read-file` — filesystem access
- Scoped: `fs:allow-read-file` with `{ "path": "$APPLOCALDATA/**" }` — reads from Local app data dir
- `clipboard-manager:default`, `clipboard-manager:allow-read-text`, `clipboard-manager:allow-write-image`, `clipboard-manager:allow-read-image` — clipboard support (text + image)

Custom Rust `#[tauri::command]` functions bypass ACL — they are always allowed.

## Global Context Menu

Browser native right-click menu is disabled globally in `main.ts` via `document.addEventListener("contextmenu", (e) => e.preventDefault())`. Browser shortcuts are also blocked: all function keys (F1-F11, F12 kept for DevTools), Ctrl+R/P/S/D/F/H/J/U/L, Alt+Left/Right for history navigation. All windows inherit this. Components that need custom right-click behavior (EditorWindow, MainWindow workspace tabs) use their own `@contextmenu` handlers without `.prevent` (global already handles prevention).

## Key Interactions

- **Double-click note** → opens EditorWindow (360×680). Uses `@click` with 400ms timer, NOT native `@dblclick` (suppressed by `user-select: none` in Chromium)
- **Long press (1s)** on note → drag ghost → release creates FloatingNoteWindow
- **Minimize main window** → saves position to localStorage, `invoke("hide_main_window")` hides to system tray
- **Close main window** → confirmation dialog → `invoke("exit_app")` exits process
- **Bell icon** (top bar) → toggles reminder list panel (red dot = pending reminders). `pendingCount` uses `now` ref (updated every 30s) for reactivity. Data loaded on every open via `loadReminders(filter)`
- **Timeline** → toggled by button in bottom bar. Calls `list_timeline_items` (single API, returns notes+reminders). Auto-scroll pagination (30/page)
- **Radial menu** → hover on three-dots button, expands horizontally right with 150ms leave delay. Items: 新建 (New) / 提醒 (Reminder, opens CreateReminderModal) / 设置 (Settings, opens SettingsWindow)
- **Favorite toggle** → NoteCard calls `notesStore.toggleFavorite()` directly
- **Edge Dock** → Drag main window to screen edge (top/left/right, 30px threshold) → window hides with 350ms animation → dock icon appears at edge → hover icon 250ms to restore. Dock hotzone is on the `dock-handle` element (26×72px), NOT the full-screen wrapper.
- **Delete prevention** → Cannot delete a note while its editor is open (checks `editor-{noteId}` window label). Toast: "该笔记编辑中！"
- **Editor close** → Saves immediately, destroys all `image-preview-*` windows, emits `editor:closed` event, then destroys window. MainWindow listens for `editor:closed` to refresh note list via `loadNotes()`
- **Editor title compact mode** — When editor scrolls past 10px, title switches to compact mode via `transform: scale(0.75) translateY(-10px)` (NOT font-size change — causes scroll jumps). Title container has fixed height (`h-9`) to prevent layout shift. A floating "scroll to top" button appears (`v-show`, not `v-if` — avoids DOM mutation).
- **Editor container padding** — `.milkdown-container` uses `padding: 0 15px 15px 15px` (no top padding) to reduce gap between title and content.
- **Image copy** → Ctrl+C or right-click "复制" on selected image copies to system clipboard via Rust `copy_image_to_clipboard` command (uses `image` crate for decoding + `arboard` for clipboard). Supports all common formats (PNG/JPEG/GIF/WEBP/BMP). Works with external apps (WeChat, Word, etc.).
- **Clipboard read** → Uses Tauri `readText()` from `@tauri-apps/plugin-clipboard-manager` instead of browser `navigator.clipboard.readText()` to avoid permission prompts. Requires `clipboard-manager:allow-read-text` permission in `capabilities/default.json`.

## Editor (MilkdownEditor.vue)

Uses `@milkdown/core` v7.21 with `commonmark`, `gfm`, `nord`, `history`, `clipboard`, `listener` plugins.

**Composable architecture**: Editor logic is split into focused composables:
- `useEditorShortcuts` — all keyboard shortcuts via ProseMirror keymap plugin
- `useEditorSync` — bidirectional modelValue sync with focus protection

**File drag-drop**: Tauri native `onDragDropEvent` handles file drops on the editor window. Image files are skipped (handled by paste handler). All other file types are converted to `[filename](file:///path)` markdown links via `insertFileLink()`. Paths are backslash-normalized and URI-encoded. Double-click opens the file with system default app via `invoke("open_file")`. Before opening, checks file existence via `exists()` from `@tauri-apps/plugin-fs`; shows toast "文件不存在！" if missing. On open failure: "文件打开失败，已拒绝或已打开！".

**Paste handling**: Single capture-phase paste handler in MilkdownEditor handles all paste scenarios:
- Image files (screenshot, etc.) → `saveAndInsertImage()`: writes to temp dir, invokes Rust `save_image`, inserts `![filename](filename)` markdown
- HTML with `file:`/`blob:` image URLs (Windows clipboard history) → strips invalid images, inserts cleaned HTML
- Normal text/HTML → lets Milkdown clipboard plugin handle it

**Image drag-drop**: Supports dragging images into the editor from external sources:
- **Tauri native drag-drop**: File explorer → `onDragDropEvent` → reads file bytes → `saveAndInsertImage()`
- **HTML5 drag-drop**: WeChat, web pages, etc. → `drop` event → `e.dataTransfer.files` → `saveAndInsertImage()`
- Both paths save the image to assets folder and insert markdown

**Image handling**: Images are stored as filenames in Markdown (`![uuid.png](uuid.png)`), NOT base64 data URLs. A MutationObserver in `setupImageResolution()` detects `<img>` tags with local filenames and resolves them to blob URLs by reading from `{appLocalDataDir}/assets/`. The `imageUrlCache` Map prevents re-reading. `isLocalFilename()` checks `!src.includes("://")` to exclude `file:`, `blob:`, `http:`, `data:` URLs. **Important**: `setupImageResolution()` is called BEFORE `Editor.create()` so the observer is attached before Milkdown renders — this prevents broken image icons from flashing. A `resolvedSrcs` Set deduplicates to prevent observer re-entry on self-triggered `src` changes. CSS hides unresolved images: `img:not([src*="://"]) { visibility: hidden; height: 0; }`.

**Image drag prevention**: Prevents images from being dragged out of the editor:
- CSS: `-webkit-user-drag: none`, `user-select: none`
- JS: Container-level `dragstart` prevention
- **Important**: Do NOT use `-webkit-app-region: no-drag` on images — it causes scroll stuttering and prevents title bar dragging when images fill the viewport

**Image selection style**: ProseMirror's `ProseMirror-selectednode` class only works for block nodes. Images are inline nodes, so a custom ProseMirror plugin (`setupImageSelection()`) detects image selection and adds `image-selected` class. Uses `box-shadow` instead of `outline` because global CSS has `* { outline: none !important; }`.

**Link detection**: ProseMirror `Decoration.inline` plugin matches `https?://` and `file://` URLs in text nodes, and also inspects `link` nodes for `file://` hrefs. Renders as blue underlined text. Double-click on web URLs opens in browser via `shell.open`. Double-click on `file://` links calls `invoke("open_file", { path })` (Rust `open::that`) to open with system default app; shows toast "该文件已打开！" on failure.

**Keyboard shortcuts**: Configurable via settings store (three-layer command pattern). Defaults: Ctrl+B (bold), Ctrl+I (italic), Ctrl+D (strikethrough), Ctrl+E (underline), Ctrl+H (heading), Ctrl+Shift+U/O (bullet/ordered list), Ctrl+Shift+T (task list), Ctrl+Shift+C (paste plain text). Non-configurable: Tab/Shift+Tab (sink/lift list or indent), Enter (split list item or exit empty list). Bold/strikethrough/underline/fontColor require text selection (no paragraph-level fallback). Screenshot shortcut configurable (default Alt+Q), EditorWindow only. Text formatting shortcuts (bold/italic/strikethrough/underline/heading/fontColor) are blocked when an image is in the selection — `hasImageInSelection()` checks the ProseMirror doc and shows toast "请框选文本后操作！". This guard runs in the keydown handler before the command registry, ensuring it catches Milkdown's built-in keymaps too.

**Formatting toolbar**: Bottom-left of EditorWindow. Buttons for bold, strikethrough, underline, heading, bullet list, ordered list, task list, font color. Exposes `execCommand(name, payload?)` via `defineExpose`. In preview mode, buttons are invisible (space preserved via `invisible` class).

**Font color**: Uses ProseMirror `Decoration.inline` plugin (NOT schema marks). `colorRanges` stored as JSON in `notes.color_ranges` column. Plugin state manages `DecorationSet`, decorations mapped through `tr.mapping` for undo/redo safety. Toggle via `tr.setMeta(COLOR_META, {ranges})`. Parent watches `update:colorRanges` event, debounces 500ms before saving to DB.

**Context menu**: Right-click in editor shows custom styled menu. Two variants: text menu (195px, items: 复制/粘贴/粘贴纯文本/加粗/删除线/下划线/有序/无序/待办) and image menu (130px, items: 复制/删除). Image detected by `e.target.tagName === "IMG"`. Boundary detection adjusts by actual menu size. Disabled in preview mode. First/last items have matching container border-radius. Image delete uses `imageUrlCache` reverse-lookup to match blob URL to filename in ProseMirror doc. **Image copy**: Uses `tauri-plugin-clipboard-manager` with `Image.fromBytes()` to copy image to system clipboard. Resolves blob URL → fetch bytes → `TauriImage.fromBytes(uint8)` → `writeImage(img)`. Both right-click menu "复制" and Ctrl+C support copying images.

**Task list**: GFM preset extends list_item with `checked` attribute. CSS renders checkbox via `::before`/`::after` pseudo-elements. Capture-phase `mousedown` handler blocks ProseMirror from handling checkbox clicks (prevents cursor placement), then uses `state.doc.descendants` + `view.nodeDOM(pos)` to find the list_item and `setNodeMarkup` to toggle checked state. Toolbar/context menu creates task list via `wrapInBulletListCommand` + `setNodeMarkup` (sets `checked: false`). Toggle: if already in task list → `liftListItemCommand.run()` exits to paragraph. Keyboard: Ctrl+Shift+T.

**Image preview**: Double-click image opens `ImagePreviewWindow` (WebviewWindow). Window created with `visible: false`, emits `preview-ready:{label}` event after image renders, editor then calls `win.show()` — no blank flash. Window size adapts to image dimensions (min 760×530, max 65% screen). Supports scroll zoom, window drag via `startDragging()` (on both container and image), ESC/close button. Uses localStorage to pass blob URL between windows. Same image reuses existing window via `getByLabel`. Closing editor destroys all `image-preview-*` windows.

**Content sync**: Uses `parserCtx` to replace ProseMirror document when external `modelValue` changes. `lastEmittedMarkdown` prevents update loops. Skips updates when editor has focus AND has content.

**Float selection menu**: Selecting text in the editor shows a floating toolbar (Teleported to `body`) with formatting buttons: bold, italic, strikethrough, underline, heading, font color. Positioned above selection center via `view.coordsAtPos()`. Only appears on `mouseup` (not during drag). Calls `execCommand()` on click, then collapses selection and hides menu. Hidden when selection is cleared, click outside editor, or when an image node is selected. Disabled in preview mode (`props.readonly` check). Does not show when selection is whitespace only.

**Find bar** (`FindBar.vue`): Ctrl+F opens find bar at top-right of editor. Pre-fills with selected text if any. Searches text in ProseMirror doc via `doc.descendants()`, highlights all matches with `Decoration.inline` (yellow). Current match has stronger highlight. Enter/Shift+Enter or buttons navigate matches. ESC closes. 150ms debounce on input. Displays "0/0" when no query. Exposes `find(query)`, `findNext()`, `findPrev()`, `clearFind()`, `replaceCurrent(text)`, `replaceAll(text)`, `getSelectedText()`, `hideFloatMenu()`, `setQuery(text)` via `defineExpose`.

**Find replace**: Expand arrow toggles replace input below find input. "替换" replaces current match and auto-advances to next (skips replaced region to avoid re-matching). "全部替换" replaces all matches at once, shows count in toast. `replaceCurrent()` returns `{ count, index }` for UI sync.

**Ctrl+A in blocks**: When cursor is inside a table cell or code block, Ctrl+A selects only that block's content (not the entire document). Uses `$from.depth` traversal to find `table_cell`/`table_header`/`code_block`/`codeblock`.

**Float menu stats**: When selected text contains countable patterns (e.g. `140g*2`, `60kg×3`), a 7th "统计" (Stats) button appears. `hasCountableContent()` regex-checks for weight/count patterns (supports g, 克, kg, 千克, 斤, 两, ml, 毫升, L, 升). Clicking it opens a stats popup (positioned below menu, opens upward if near window bottom) showing total weight, per-line breakdown, copy button, and item count. `calcSelectionStats()` normalizes all units to grams for totals.

**`suppressFloatMenu` guard**: Module-level boolean prevents the float menu from reappearing at wrong times. Set `true` in `floatAction()` (reset after 200ms) and `showStats()` (reset on popup close). Checked in both `updateFloatMenu()` and the ProseMirror plugin `update` hook. Also suppressed when selection matches a find highlight range (checked via `findHighlightKey` state).

## Reminder System

Reminders stored in `reminders` table with `completed`, `snoozed`, `deleted`, `cancelled` fields.

**States:**
- Active: `completed=0, cancelled=0, deleted=0` — shows Cancel + Snooze buttons
- Overdue: active + `remind_at < now - 65s` — shows "已过期" label, no buttons
- Completed: `completed=1` — shows "已完成" label
- Cancelled: `cancelled=1` — shows "已取消" label

**Backend filtering/sorting** (`list_reminders` command):
- Accepts `filter` param: `all`, `today`, `overdue`, `cancelled`, `completed`
- Returns `ReminderListResponse` with filtered list + `ReminderCounts` for all tabs
- "all" tab: active first (by `remind_at ASC`), then overdue, cancelled, completed
- "today"/"overdue" tabs: `remind_at DESC`
- Duplicate check: same minute + non-deleted + non-cancelled

**Notification popup** (`ReminderBubble.vue`): Slides in from right at screen top-right. Shows title ("提醒 · {标题}"), content, 60s countdown ring. Buttons: 完成 / 稍后提醒 (5min). Auto-snooze on timeout. Hover pauses countdown. Uses fire-and-forget pattern: `invoke("complete_reminder"/"snooze_reminder").catch(() => {})` without await, emits `reminder:updated` event, then immediately closes window. MainWindow listens for `reminder:updated` to refresh its reminder list.

**Cross-window sync**: `reminder:updated` event is emitted by CreateReminderModal (on create), ReminderView (on complete/cancel/delete/snooze), and ReminderBubble (on complete/snooze). MainWindow listens and calls `remindersStore.loadReminders()` to refresh the list. Bell icon red dot counts pending reminders excluding `completed`, `cancelled`, and `deleted`.

**CreateReminderModal.vue**: Shared component used by MainWindow (radial menu) and ReminderView. Uses `absolute` positioning (NOT `fixed`) with `rounded-3xl overflow-hidden` to preserve window rounded corners. Number input time picker (hour 0-23, minute 0-59, spin buttons hidden via CSS), custom dropdown date picker (year/month/day with past-date filtering). Title required (15 char max). Default time is current + 1 hour (date rolls over if crossing midnight). `resetForm()` recalculates on every close. Time validation: `clampHour()`/`clampMinute()` on blur constrain to valid ranges; `handleCreate` checks `dt.getTime() <= Date.now()` and shows toast "提醒时间需大于当前时间！".

## Edge Dock Architecture

Global singleton in `useEdgeDock.ts`. State machine: `visible` → `hiding` → `docked` → `restoring` → `visible`. `getMonitorInfo()` caches monitor data permanently (called once, reused forever). `initialized` global lock ensures single initialization; `onUnmounted` resets it.

**Race condition protection**: `hideRequestId` counter invalidates stale async hide tasks. Every `hideWindow()` increments and captures the ID; async continuations bail out if the ID changed.

**Pointer tracking**: Dual tracking with intentional 50ms debounce:
- `pointermove` on root element: tracks pointer presence, distinguishes drag-region vs content area
- `pointerleave` as backup: 50ms delay before setting `pointerInContent = false` (intentional debounce to prevent fast mouse-in/out from triggering hide)
- Staleness check in 200ms interval: no `pointermove` for 500ms → treats pointer as outside → triggers hide
- `DRAG_REGION_TIMEOUT = 2000`: prevents hide while user interacts with drag-region

**Hide timing safety**: `suppressHideUntil` (300ms after `showWindow()`) prevents hide during state instability. Minimum path from show to hide: 300ms (cooldown) + 500ms (staleness) + 200ms (interval) + 100ms (delay) ≈ 1.1s — cooldown always expires before hide can trigger.

**Focus management**: On focus-lost, adds `window-unfocused` CSS class to `<html>` which pauses all CSS animations (`animation-play-state: paused !important`), clears the 200ms hover-check interval. On focus-gained, removes the class and restarts the interval. This prevents unnecessary GPU usage when the window is minimized/hidden.

**Dock icon** (`EdgeDock.vue`): 26×72px (left/right) or 72×24px (top). Right edge measures invisible border via `outerSize()/innerSize()` at dock creation time to position handle flush with screen edge. Hover/click restores window. Created with `visible: false` then shown after 85ms delay (Tauri transparent window rendering bug). Position uses physical pixels. Main window and editor use same delay.

**Editor Dock** (`useAutoHide.ts`): Editor windows use a similar edge dock mechanism. Key differences:
- Serial execution: dock window created first, then editor slides off-screen (avoids animation conflicts)
- `isCollapsed` state removed — now uses `window.hide()` directly (taskbar icon gone, no preview needed)
- `restoringFromDock` flag — prevents `onMoved` listener from showing taskbar icon during restore animation. Set `true` at start of `restoreFromEdge()`, `false` after animation completes. User manually moving window away from edge still shows taskbar icon normally.
- `animatePosition` uses independent `localAnimFrame` per call (prevents conflicts when multiple animations run)
- Uses cached `monitorCache` for all position calculations (no async calls during hide/show)
- Dock fade-out uses per-editor events (`editor-dock:fade-out-${dockId}`) to avoid affecting other editors
- Close callback (`setOnClose`) for taskbar close when editor is hidden
- Taskbar close cleanup: `onCloseRequested` handler destroys `image-preview-*` windows and cleans their localStorage entries (same as in-app close button)
- Pinned suppression: `watch(pinned)` sets `autoHideEnabled = false` when pinned, prevents edge dock while window is always-on-top

**Editor window tracking**: `localStorage["open-editors"]` tracks open editor labels. On open: cleans up destroyed windows, checks count (< 3). On close: removes label from tracking. Uses `outerSize()` to verify window existence (handles destroyed windows that still appear in `getAll()`).

## Workspace System

Workspaces stored in dedicated `workspaces` table (not derived from notes). `list_workspaces` queries this table. `addWorkspace(name)` persists to DB via `create_workspace` command. `removeWorkspace(name)` moves notes to `default` then deletes the workspace row. Default workspace is insert-ignored on migration.

## Toast System

`useToast.ts` uses module-level refs for shared state. Any component can call `useToast()` → `toast("message")`. The `<Toast>` component is rendered once in `App.vue` with a CSS shake animation. 2500ms auto-dismiss.

## Data Directory

All app data uses `app.path().app_local_data_dir()` → `AppData\Local\com.flownote.app\`:
- `flownote.db` — SQLite database (via `db::get_db_path`)
- `assets/` — pasted images (via `commands::images::get_assets_dir`)
- `settings.json` — user preferences (via `load_settings`/`save_settings` commands)

App icon: `src-tauri/icons/icon.png` (source) + `src-tauri/icons/icon.ico` (generated multi-size: 16/32/48/64/128/256). Use Python PIL to generate ICO from PNG. NSIS installer configured for Chinese (`languages: ["SimpChinese"]`).

On startup, `lib.rs` migrates old directory `AppData\Local\FlowNote\` → `AppData\Local\com.flownote.app\` if it exists.

## Delete Model

Notes use **soft delete**: `DELETE` is actually `UPDATE notes SET archived = 1`. `list_notes` filters `WHERE archived = 0`. Deleted notes in timeline show `line-through` + `opacity-50`. Cannot delete a note while its editor window is open (checks `editor-{noteId}` window label). Deleting a note also destroys its open editor window.

- **`WebviewWindow.getByLabel(label)` is async** — returns `Promise<WebviewWindow | null>`, must be awaited
- `WebviewWindow` methods: `show()`, `hide()`, `setFocus()`, `destroy()` (force-close), `setAlwaysOnTop()`, `close()` (emits closeRequested)
- `getCurrentWindow()` from `@tauri-apps/api/window` for window operations
- `getCurrentWebviewWindow()` from `@tauri-apps/api/webviewWindow` for the full webview window
- `xcap::Monitor` is not `Send` — wrap in `tokio::task::spawn_blocking()`
- Screenshot uses Win32 API (`windows` crate) for native overlay: `UpdateLayeredWindow`, `SetCapture`, `MonitorFromPoint`, etc.
- `xcap::Monitor::capture_image()` captures monitor content; `image` crate for cropping/resizing
- `open::that(path)` (Rust crate `open`) opens files with system default app, used by `open_file` command
- `sqlx::SqliteConnectOptions` with `.create_if_missing(true)` for fresh DB creation
- `app.emit()` needs `use tauri::Emitter;` in scope
- `app.state::<AppState>()` needs state to be managed first (order in setup matters)
- `convertFileSrc()` from `@tauri-apps/api/core` for converting local file paths to asset URLs
- `writeFile` from `@tauri-apps/plugin-fs` for writing binary files
- `open` from `@tauri-apps/plugin-shell` for opening URLs/files in default app
- `writeImage` from `@tauri-apps/plugin-clipboard-manager` for copying images to clipboard (requires `Image` from `@tauri-apps/api/image` — import as `TauriImage` to avoid shadowing browser's `Image`)
- Cross-window events: `emit(event, payload)` / `listen(event, handler)` from `@tauri-apps/api/event` for inter-window communication (e.g. `screenshot:captured`, `screenshot:do-capture`, `editor:closed`, `reminder:updated`, `settings:updated`)

## Tray Menu

System tray uses custom styled popup (NOT native menu) for consistent UI. Right-click tray icon → creates/shows `tray-menu` webview window at mouse position (physical pixels). Left-click shows main window. Menu closes on `mouseleave` (200ms delay). `tray-menu` window label must be in `capabilities/default.json`.

## Settings System

`SettingsWindow.vue` (320×520) opened from radial menu. Five tabs: 通用 / 快捷键 / 外观 / 截图 / 关于. Settings persisted via Rust commands (`load_settings`/`save_settings`) to `settings.json`. Cross-window sync via Tauri `emit`/`listen` (`settings:updated` event) — NOT `window.addEventListener("storage")`. Every save writes the entire settings object (shortcuts + appearance + screenshot) as a single JSON blob.

**General tab**: Auto-start toggle. Uses `get_autostart`/`set_autostart` Rust commands to read/write Windows Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`). Uses `winreg` crate.

**About tab**: Displays app icon (`@/assets/icon.png`), app name "FlowNote", tagline, and copyright.

**Shortcut rebind flow**: click button → enter rebind mode → next keydown normalizes → check conflicts (toast if duplicate, NOT allowed to override) → save to store → rebuild shortcutMap in `useEditorShortcuts`. ESC cancels. Modifier-only keys (Ctrl/Shift/Alt/Meta) are ignored.

**Appearance**: sliders preview via CSS vars on `input` and commit on `change`. CSS variables `--main-opacity` and `--editor-opacity` set on `document.documentElement`. MainWindow uses `var(--main-opacity, 0.80)`, EditorWindow uses `var(--editor-opacity, 0.82)`. Min 60%, max 100%.

**Theme system**: 6 themes defined in `THEMES` array (`settings.ts`): default, hello-kitty, sakura-cream, sweet-lavender, cotton-lavender, galaxy-lavender. Theme applied via `data-theme` attribute on `<html>`. CSS variables (RGB triplets) defined in `global.css` for each theme: `--bg-primary`, `--bg-secondary`, `--bg-panel`, `--accent`, `--accent-soft`, `--interactive`, `--border`, `--glass-bg`, `--scrollbar`, `--favorite`, etc. All windows use `rgba(var(--bg-primary), ...)` for backgrounds. Decorative effects (sparkle animation, bow icon) display on pink themes via `.theme-hk-sparkle` and `.theme-hk-bow` classes.

**Screenshot config**: auto-saved on area selection and toggle change (no explicit save button). `AreaSelectWindow` for drag-selecting area, saves x/y/width/height via Tauri event `area:selected`. Fixed area toggle controls whether screenshots use saved position/dimensions or open ScreenshotWindow for manual selection.

**Command registry** (`useCommandRegistry.ts`): Singleton `Map<string, () => void>`. Handlers registered once when `useEditorShortcuts` initializes. `executeCommand(id)` called by both keyboard shortcuts and toolbar. Toolbar also has its own `execCommand` with paragraph-level logic for bold/strikethrough/underline.

**Screenshot pipeline**: Screenshot only in EditorWindow (not MainWindow). Uses native Win32 overlay for multi-monitor + DPI support:

- **Interactive selection** (`select_and_capture_screenshot`): Rust creates native Win32 overlay windows (one per monitor) using `UpdateLayeredWindow` for per-pixel alpha. User selects region → overlay returns physical pixel coordinates → `capture_physical_region()` finds correct monitor via `MonitorFromPoint()`, captures with xcap, crops and resizes to logical pixels.

- **Fixed area** (`capture_fixed_area`): Uses physical pixel coordinates stored from previous `select_screen_area()` call. Same `capture_physical_region()` path.

- **Settings area selection** (`select_screen_area`): Same native overlay, returns coordinates for settings storage.

**Key Rust functions**:
- `show_native_overlay()`: Creates Win32 overlay windows on all monitors, handles mouse events, returns physical pixel coordinates
- `capture_physical_region()`: Physical pixels → find monitor → capture → crop → resize to logical pixels
- `capture_region()`: Logical pixels → iterate all monitors → crop each → paste onto canvas (for multi-monitor spanning)

**Coordinate handling**: All coordinates from native overlay are physical pixels. `capture_physical_region()` uses `MonitorFromPoint()` to find the correct monitor, then maps physical offsets to image pixel coordinates using scale factor. Output is resized to logical pixels.

`MilkdownEditor.insertImageFromPath()` extracts filename, pre-caches blob URL, inserts `![filename](filename)\n` markdown at cursor (trailing newline positions cursor below image).

## Update System

Cold update via `latest.json` hosted on GitHub. `check_update` command fetches manifest, compares versions, applies `downloadProxy` logic (if value exists, prepend to URL; if empty/false, use URL directly). `download_and_install` streams download with progress events (`update-download-progress`), runs installer, waits 2s, exits app.

**Manifest format** (`latest.json`):
```json
{
  "version": "0.3.91",
  "downloadProxy": "",
  "url": "https://github.com/ChenYueC/FlowNote/raw/refs/heads/main/FlowNote_0.3.91_x64-setup.exe"
}
```

**Auto-check on startup** (`MainWindow.vue`): Checks `latest.json` on mount. If new version and user hasn't dismissed within 3 days, opens `UpdateDialog.vue` window (320×260, not resizable, always-on-top). Dialog shows version + progress bar on download. Dismiss saves `update-dismissed-at` to localStorage.

**Manual check** (`SettingsWindow.vue` "关于" tab): "检查更新" button, shows version + "下载更新" on new version.

## Dev Tooling

No tests, no linter, no CI/CD pipeline. Manual testing only.

## UI Style

Frosted glass (glassmorphism). `.glass`: `rgba(255,255,255,0.08)` + `blur(10px) saturate(130%)`. `.glass-strong`: `rgba(255,255,255,0.15)` + `blur(10px) saturate(130%)`. Used by Toast, context menus, and confirmation modals. All windows `decorations: false, transparent: true, shadow: false`. Window root divs need `rounded-3xl overflow-hidden`. Scrollbar: 5px capsule (`border-radius: 999px`, `background-clip: padding-box`). Workspace tabs use `.tabs-scrollbar` (2px WebKit pseudo-element, show on hover via `:has(button:hover)`). `user-select: none` on `html, body, #app` — this suppresses native `dblclick` events in Chromium.

**Font**: JetBrains Mono (woff2 in `src/assets/fonts/`), loaded via `global.css`. Tailwind custom color palette: `fn-*` colors defined in `tailwind.config.ts`.

**GPU optimization**: MainWindow and EditorWindow root elements have NO `backdrop-filter` (pure semi-transparent background). Overlay panels (reminder list, timeline) use `rgba(30,30,46,0.95)` without blur. Cards use fixed backgrounds instead of `.glass`. `backdrop-filter` is reserved for small elements (context menus, float menus, dock handle, toast) where the blur area is small. Global `* { outline: none !important; }` removes focus rings on all elements (including buttons, which Tailwind preflight styles otherwise).

**Radial pulse animation**: Uses `transform: scale()` + `opacity` on a `::before` pseudo-element (GPU composited, NOT `box-shadow` which triggers repaint). `.radial-pulse-ring` class in `global.css`. Duration 3s. `.window-unfocused` class pauses all animations when window loses focus.

Modal overlays use `style="background: rgba(0, 0, 0, 0.60);"` (NOT `backdrop-blur`) to avoid text halo effect.

**Editor text**: Fixed white `rgba(255, 255, 255, 0.9)` — does NOT follow theme's `--text-primary`. Bold: `font-weight: 800`. Strikethrough: `text-decoration-thickness: 2px`, `opacity: 0.65`.

**Toolbar buttons**: All toolbar buttons (top and bottom) must have `tabindex="-1"` to prevent Tab from stealing editor focus. Bottom toolbar buttons use `@mousedown.prevent` to prevent focus loss on click. All bottom SVG icons use `stroke-linecap="round"` + `stroke-linejoin="round"` for consistent visual weight.

**Image preview close button**: Adapts to image background brightness — samples top-right corner pixel, if brightness > 128 → black button, else white.
