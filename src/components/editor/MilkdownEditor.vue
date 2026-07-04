<template>
  <div class="milkdown-container no-drag" ref="editorRef" />
  <Teleport to="body">
    <div
      v-if="floatMenu.visible"
      ref="floatMenuRef"
      class="editor-float-menu"
      :style="{ left: floatMenu.x + 'px', top: floatMenu.y + 'px' }"
      @mousedown.prevent
      @pointerenter="emit('floatMenuHover', true)"
      @pointerleave="emit('floatMenuHover', false)"
    >
      <button @mousedown.prevent @click="floatAction('bold')" title="加粗">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4h8a4 4 0 014 4 4 4 0 01-4 4H6z"/><path d="M6 12h9a4 4 0 014 4 4 4 0 01-4 4H6z"/></svg>
      </button>
      <button @mousedown.prevent @click="floatAction('italic')" title="斜体">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="4" x2="10" y2="4"/><line x1="14" y1="20" x2="5" y2="20"/><line x1="15" y1="4" x2="9" y2="20"/></svg>
      </button>
      <button @mousedown.prevent @click="floatAction('strikethrough')" title="删除线">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 5H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H7"/><line x1="4" y1="12" x2="20" y2="12"/></svg>
      </button>
      <button @mousedown.prevent @click="floatAction('underline')" title="下划线">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4v6a6 6 0 0012 0V4"/><line x1="4" y1="20" x2="20" y2="20"/></svg>
      </button>
      <button @mousedown.prevent @click="floatAction('heading')" title="标题">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 5v14M18 5v14M6 12h12"/></svg>
      </button>
      <button @mousedown.prevent @click="floatAction('fontColor')" title="字体颜色">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m5 15 7-11 7 11M7 11h10"/><path d="M3 20h18" stroke-width="2.5"/></svg>
      </button>
      <template v-if="floatMenu.hasStats">
        <div class="float-menu-divider"></div>
        <button @mousedown.prevent @click="showStats" title="统计">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 3v18h18"/><path d="M7 16l4-8 4 4 4-6"/></svg>
        </button>
      </template>
    </div>
    <!-- Stats popup -->
    <div
      v-if="statsPopup.visible"
      ref="statsPopupRef"
      class="stats-popup"
      :style="{ left: statsPopup.x + 'px', top: statsPopup.y + 'px' }"
    >
      <div v-if="statsPopup.totalGrams > 0" class="stats-total">
        合计：{{ statsPopup.totalGrams }}g（{{ (statsPopup.totalGrams / 1000).toFixed(statsPopup.totalGrams % 1000 === 0 ? 0 : 2) }}kg）
      </div>
      <div v-if="statsPopup.details.length" class="stats-details">
        <div v-for="(d, i) in statsPopup.details" :key="i" class="stats-detail-item">{{ d }}</div>
      </div>
      <div class="stats-bottom">
        <div class="stats-bottom-btns">
          <button class="stats-copy-btn" @mousedown.prevent @click="copyStats">复制</button>
        </div>
        <span v-if="statsPopup.totalItems > 0" class="stats-count">共 {{ statsPopup.totalItems }} 件</span>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, type Ref } from "vue";
import { Editor, rootCtx, defaultValueCtx, editorViewCtx, parserCtx } from "@milkdown/core";
import { commonmark } from "@milkdown/preset-commonmark";
import { gfm } from "@milkdown/preset-gfm";
import { nord } from "@milkdown/theme-nord";
import { history } from "@milkdown/plugin-history";
import { clipboard } from "@milkdown/plugin-clipboard";
import { listener } from "@milkdown/plugin-listener";
import { Plugin, PluginKey, TextSelection, NodeSelection } from "@milkdown/prose/state";
import { DecorationSet, Decoration } from "@milkdown/prose/view";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { currentMonitor } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-shell";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { readFile, writeFile, exists } from "@tauri-apps/plugin-fs";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { join, tempDir } from "@tauri-apps/api/path";
import { toggleStrikethroughCommand } from "@milkdown/preset-gfm";
import { toggleStrongCommand, toggleEmphasisCommand, toggleInlineCodeCommand, wrapInHeadingCommand, wrapInBulletListCommand, wrapInOrderedListCommand, liftListItemCommand } from "@milkdown/preset-commonmark";
import { commandsCtx } from "@milkdown/core";
import { useEditorShortcuts, underlineAttr, underlineSchema, toggleUnderlineCommand } from "@/composables/useEditorShortcuts";
import { createCodeHighlightPlugin } from "@/composables/useCodeHighlight";
import { useEditorSync } from "@/composables/useEditorSync";
import { useToast } from "@/composables/useToast";

export interface ColorRange {
  from: number;
  to: number;
  color: string;
}

const props = withDefaults(
  defineProps<{
    modelValue: string;
    readonly?: boolean;
    noteId?: string;
    colorRanges?: ColorRange[];
  }>(),
  { readonly: false, noteId: "", colorRanges: () => [] },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  "update:colorRanges": [value: ColorRange[]];
  "floatMenuHover": [hovering: boolean];
}>();

const editorRef = ref<HTMLElement>();
const floatMenuRef = ref<HTMLElement>();
const statsPopupRef = ref<HTMLElement>();
const floatMenu = ref({ visible: false, x: 0, y: 0, hasStats: false });
const statsPopup = ref({ visible: false, x: 0, y: 0, totalGrams: 0, totalItems: 0, details: [] as string[], copyText: "" });
let editor: Editor | null = null;
let fileDropUnlisten: (() => void) | null = null;
const cleanupFns: (() => void)[] = [];

const URL_REGEX = /(?:https?|file):\/\/[^\s<>)\]]+/g;

// --- Color decoration plugin ---
const colorDecoKey = new PluginKey("colorDeco");
const COLOR_META = "colorDeco:update";

function buildColorDecorations(doc: any, ranges: ColorRange[]): DecorationSet {
  const decos = ranges
    .filter((r) => r.from >= 0 && r.to <= doc.content.size)
    .map((r) => Decoration.inline(r.from, r.to, { style: `color: ${r.color}` }));
  return DecorationSet.create(doc, decos);
}

function createColorPlugin(initialRanges: ColorRange[]) {
  return new Plugin({
    key: colorDecoKey,
    state: {
      init(_, state) { return buildColorDecorations(state.doc, initialRanges); },
      apply(tr, oldDecos: DecorationSet): DecorationSet {
        let decos = oldDecos.map(tr.mapping, tr.doc);
        const meta = tr.getMeta(COLOR_META);
        if (meta) {
          decos = buildColorDecorations(tr.doc, meta.ranges);
        }
        return decos;
      },
    },
    props: {
      decorations(state) { return colorDecoKey.getState(state); },
    },
  });
}

// --- Trailing paragraph plugin ---
// When the doc ends with a non-text block (code_block / table / image), the
// empty area below it has no placeable node, so clicks land inside the last
// block and the cursor stays trapped. We keep an empty trailing paragraph so
// there is always a text block at the very end to receive the cursor.
function createTrailingParagraphPlugin() {
  const key = new PluginKey("trailing-paragraph");
  // Node types whose presence as the last top-level child mandates a trailing
  // paragraph (these are the blocks you can't place a text cursor "below").
  function needsTrailing(node: any): boolean {
    const name = node.type.name;
    return name === "code_block" || name === "codeblock" || name === "table" || name === "image";
  }
  return new Plugin({
    key,
    appendTransaction: (trs, _oldState, newState) => {
      // Only consider appending when at least one transaction actually changed
      // the document — ignore pure selection/metadata transactions so we never
      // react to focus, scroll, or decoration updates (which could otherwise
      // cause the editor to jump to the end).
      const docChanged = trs.some((tr) => tr.docChanged);
      if (!docChanged) return null;
      const lastChild = newState.doc.lastChild;
      if (!lastChild || !needsTrailing(lastChild)) return null;
      // Append an empty paragraph at the end of the doc. Mark it so ProseMirror
      // does NOT auto-scroll the new paragraph into view.
      const paragraphType = newState.schema.nodes.paragraph;
      const tr = newState.tr.insert(newState.doc.content.size, paragraphType.create());
      tr.setMeta("addToHistory", false);
      (tr as any).scrollIntoView = false;
      return tr;
    },
  });
}

// --- Find highlight plugin ---
const FIND_META = "find-update";
const findHighlightKey = new PluginKey("find-highlight");

function createFindHighlightPlugin() {
  return new Plugin({
    key: findHighlightKey,
    state: {
      init() {
        return { ranges: [] as { from: number; to: number }[], currentIndex: -1, query: "" };
      },
      apply(tr, old) {
        const meta = tr.getMeta(FIND_META);
        if (meta) return meta;
        if (tr.docChanged && old.query) {
          // Re-search after doc change
          const ranges = searchDoc(tr.doc, old.query);
          let currentIndex = old.currentIndex;
          if (currentIndex >= ranges.length) currentIndex = ranges.length - 1;
          return { ranges, currentIndex, query: old.query };
        }
        return old;
      },
    },
    props: {
      decorations(state) {
        const { ranges, currentIndex } = findHighlightKey.getState(state) || { ranges: [], currentIndex: -1 };
        if (!ranges.length) return DecorationSet.empty;
        const decos: Decoration[] = [];
        ranges.forEach((r: { from: number; to: number }, i: number) => {
          const cls = i === currentIndex ? "find-highlight-active" : "find-highlight";
          decos.push(Decoration.inline(r.from, r.to, { class: cls }));
        });
        return DecorationSet.create(state.doc, decos);
      },
    },
  });
}

function searchDoc(doc: any, query: string): { from: number; to: number }[] {
  const ranges: { from: number; to: number }[] = [];
  if (!query) return ranges;
  const lower = query.toLowerCase();
  doc.descendants((node: any, pos: number) => {
    if (!node.isText) return;
    const text = node.text?.toLowerCase();
    if (!text) return;
    let idx = 0;
    while ((idx = text.indexOf(lower, idx)) !== -1) {
      ranges.push({ from: pos + idx, to: pos + idx + query.length });
      idx += 1;
    }
  });
  return ranges;
}

// --- Link decoration plugin (unchanged) ---
function createLinkPlugin() {
  const key = new PluginKey("link-decorator");
  return new Plugin({
    key,
    state: {
      init(_, state) {
        return buildDecorations(state.doc);
      },
      apply(tr, old) {
        if (tr.docChanged) return buildDecorations(tr.doc);
        return old;
      },
    },
    props: {
      decorations(state) {
        return key.getState(state);
      },
      handleDOMEvents: {
        dblclick(_view, event) {
          const target = event.target as HTMLElement;
          // Check for detected-link decoration or <a> tag with href
          const link = target.closest(".detected-link") as HTMLElement | null;
          const url = link?.dataset.url || (target.closest("a") as HTMLAnchorElement)?.href;
          if (url) {
            if (url.startsWith("file://")) {
              const path = decodeURI(url.replace("file:///", "").replace(/\//g, "\\"));
              exists(path).then((fileExists) => {
                if (!fileExists) {
                  toast("文件不存在！");
                } else {
                  invoke("open_file", { path }).catch(() => {
                    toast("文件打开失败，已拒绝或已打开！");
                  });
                }
              });
            } else {
              open(url);
            }
            event.preventDefault();
            return true;
          }
          return false;
        },
      },
    },
  });
}

function buildDecorations(doc: any): DecorationSet {
  const decos: Decoration[] = [];
  doc.descendants((node: any, pos: number) => {
    if (!node.isText) {
      // Check link nodes for file:// href
      if (node.type?.name === "link") {
        const href = node.attrs?.href || "";
        if (href.startsWith("file://")) {
          const from = pos + 1;
          const to = pos + node.nodeSize - 1;
          decos.push(
            Decoration.inline(from, to, {
              class: "detected-link",
              "data-url": href,
            }),
          );
        }
      }
      return;
    }
    const text = node.text;
    if (!text) return;
    let match: RegExpExecArray | null;
    URL_REGEX.lastIndex = 0;
    while ((match = URL_REGEX.exec(text)) !== null) {
      const from = pos + match.index;
      const to = from + match[0].length;
      decos.push(
        Decoration.inline(from, to, {
          class: "detected-link",
          "data-url": match[0],
        }),
      );
    }
  });
  return DecorationSet.create(doc, decos);
}

// --- Task list checkbox click ---
function setupCheckboxClick() {
  const container = editorRef.value;
  if (!container) return;

  function handleCheckboxToggle(e: MouseEvent) {
    if (!editor) return;
    const target = e.target as HTMLElement;
    const li = target.closest('li[data-item-type="task"]') as HTMLElement | null;
    if (!li) return;

    const liRect = li.getBoundingClientRect();
    if (e.clientX > liRect.left + 20) return;

    // Fully block ProseMirror from seeing this event
    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();

    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state } = view;

      // Find the list_item node in ProseMirror doc by iterating all nodes
      let nodePos = -1;
      state.doc.descendants((node, pos) => {
        if (nodePos >= 0) return false; // already found
        if (node.type.name === "list_item" && node.attrs.checked != null) {
          // Check if this node corresponds to the clicked DOM element
          const dom = view.nodeDOM(pos);
          if (dom && (dom === li || dom.contains(li) || li.contains(dom))) {
            nodePos = pos;
          }
        }
      });

      if (nodePos < 0) return;
      const node = state.doc.nodeAt(nodePos);
      if (!node) return;
      const tr = state.tr.setNodeMarkup(nodePos, undefined, {
        ...node.attrs,
        checked: !node.attrs.checked,
      });
      view.dispatch(tr);
    });
  }

  // Capture phase — runs before ProseMirror
  container.addEventListener("mousedown", handleCheckboxToggle, true);
  cleanupFns.push(() => container.removeEventListener("mousedown", handleCheckboxToggle, true));
}

// --- Float selection menu ---
let suppressFloatMenu = false;

function hasCountableContent(text: string): boolean {
  return /(\d+(?:\.\d+)?)\s*(g|克|kg|千克|斤|两|ml|毫升|L|升)\s*[×xX*]\s*\d+/i.test(text)
    || /[×xX*]\s*\d+/.test(text);
}

function calcSelectionStats(text: string): { totalGrams: number; totalItems: number; details: string[]; copyText: string } {
  const weightRegex = /(\d+(?:\.\d+)?)\s*(g|克|kg|千克|斤|两|ml|毫升|L|升)\s*[×xX*]\s*(\d+)/gi;
  let totalGrams = 0;
  let match;
  const details: string[] = [];

  while ((match = weightRegex.exec(text)) !== null) {
    const num = parseFloat(match[1]);
    const unit = match[2].toLowerCase();
    const count = parseInt(match[3]);
    let grams = num;
    if (unit === "kg" || unit === "千克") grams = num * 1000;
    else if (unit === "斤") grams = num * 500;
    else if (unit === "两") grams = num * 50;
    else if (unit === "ml" || unit === "毫升") grams = num;
    else if (unit === "L" || unit === "升") grams = num * 1000;
    const subtotal = grams * count;
    totalGrams += subtotal;
    const subStr = subtotal >= 1000
      ? `${(subtotal / 1000).toFixed(subtotal % 1000 === 0 ? 0 : 1)}kg`
      : `${subtotal}g`;
    details.push(`${details.length + 1}. ${num}${unit} × ${count} = ${subStr}`);
  }

  const countRegex = /[×xX*]\s*(\d+)/g;
  let totalItems = 0;
  while ((match = countRegex.exec(text)) !== null) {
    totalItems += parseInt(match[1]);
  }

  const parts: string[] = [];
  if (totalGrams > 0) {
    const kg = totalGrams / 1000;
    const kgStr = kg.toFixed(kg % 1 === 0 ? 0 : 2);
    parts.push(`合计：${totalGrams}g（${kgStr}kg）`);
  }
  if (totalItems > 0) {
    parts.push(`共 ${totalItems} 件`);
  }

  return { totalGrams, totalItems, details, copyText: parts.join("，") };
}

function floatAction(action: string) {
  suppressFloatMenu = true;
  execCommand(action);
  // Collapse selection to cursor at end
  editor?.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const { state } = view;
    const { to } = state.selection;
    const tr = state.tr.setSelection(TextSelection.create(state.doc, to));
    view.dispatch(tr);
  });
  floatMenu.value.visible = false;
  setTimeout(() => { suppressFloatMenu = false; }, 200);
}

function showStats() {
  if (!editor) return;
  let result = { totalGrams: 0, totalItems: 0, copyText: "" };
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const { state } = view;
    const { from, to } = state.selection;
    const text = state.doc.textBetween(from, to, " ");
    result = calcSelectionStats(text);
  });
  if (!result.copyText) return;

  // Collapse selection
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const { state } = view;
    const { to } = state.selection;
    view.dispatch(state.tr.setSelection(TextSelection.create(state.doc, to)));
  });

  const menuEl = floatMenuRef.value;
  if (menuEl) {
    const rect = menuEl.getBoundingClientRect();
    const popupMaxH = 260;
    let x = rect.left;
    let y = rect.bottom + 6;
    if (y + popupMaxH > window.innerHeight - 8) {
      y = rect.top - popupMaxH - 6;
      if (y < 4) y = 4;
    }
    statsPopup.value = { visible: true, x, y, totalGrams: result.totalGrams, totalItems: result.totalItems, details: result.details, copyText: result.copyText };
  }
  suppressFloatMenu = true;
  floatMenu.value.visible = false;
}

function copyStats() {
  navigator.clipboard.writeText(statsPopup.value.copyText);
  statsPopup.value.visible = false;
  suppressFloatMenu = false;
}

function setupFloatMenu() {
  const container = editorRef.value;
  if (!container) return;

  let mouseDown = false;

  function updateFloatMenu() {
    if (suppressFloatMenu) return;
    if (!editor || props.readonly) { floatMenu.value.visible = false; return; }
    try { editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state } = view;
      const { from, to, empty } = state.selection;

      if (empty || from === to) {
        floatMenu.value.visible = false;
        return;
      }

      // Don't show menu when image is selected
      let hasImage = false;
      state.doc.nodesBetween(from, to, (node: any) => {
        if (node.type.name === "image") hasImage = true;
      });
      if (hasImage) {
        floatMenu.value.visible = false;
        return;
      }

      // Don't show menu when selection is a find match
      const findState = findHighlightKey.getState(state);
      if (findState && findState.ranges.length) {
        const isFindMatch = findState.ranges.some((r: { from: number; to: number }) => r.from === from && r.to === to);
        if (isFindMatch) {
          floatMenu.value.visible = false;
          return;
        }
      }

      // Get selection coordinates
      const startCoords = view.coordsAtPos(from);
      const endCoords = view.coordsAtPos(to);

      // Position above the selection center
      const centerX = (startCoords.left + endCoords.left) / 2;
      const topY = Math.min(startCoords.top, endCoords.top);

      const menuWidth = floatMenuRef.value?.offsetWidth || 216;
      let x = centerX - menuWidth / 2;
      let y = topY - 42;

      // Clamp to viewport
      x = Math.max(4, Math.min(x, window.innerWidth - menuWidth - 4));
      y = Math.max(4, y);

      const selectedText = state.doc.textBetween(from, to, " ");
      // Don't show menu if selection is only whitespace
      if (!selectedText.trim()) {
        floatMenu.value.visible = false;
        return;
      }
      const hasStats = hasCountableContent(selectedText);

      floatMenu.value = { visible: true, x, y, hasStats };
    }); } catch {}
  }

  // Track mouse state — don't show menu while selecting
  const onMouseDown = (e: Event) => {
    mouseDown = true;
    const anchor = (e.target as HTMLElement).closest("a");
    if (anchor) e.preventDefault();
  };
  const onMouseUp = () => {
    mouseDown = false;
    requestAnimationFrame(updateFloatMenu);
  };
  container.addEventListener("mousedown", onMouseDown);
  document.addEventListener("mouseup", onMouseUp);

  // Listen for selection changes via ProseMirror plugin
  if (!editor) return;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const floatMenuPlugin = new Plugin({
      key: new PluginKey("float-menu"),
      view: () => ({
        update: (view) => {
          if (suppressFloatMenu) return;
          const { state } = view;
          const { empty } = state.selection;
          if (empty) {
            floatMenu.value.visible = false;
            return;
          }
          // Only update position when not actively selecting
          if (!mouseDown) requestAnimationFrame(updateFloatMenu);
        },
      }),
    });
    const newState = view.state.reconfigure({
      plugins: [...view.state.plugins, floatMenuPlugin],
    });
    view.updateState(newState);
  });

  // Hide on click outside
  const onDocumentMouseDown = (e: Event) => {
    // Close stats popup on outside click
    if (statsPopup.value.visible && statsPopupRef.value && !statsPopupRef.value.contains(e.target as Node)) {
      statsPopup.value.visible = false;
      suppressFloatMenu = false;
    }
    if (!floatMenu.value.visible) return;
    const menu = floatMenuRef.value;
    if (menu && menu.contains(e.target as Node)) return;
    if (container.contains(e.target as Node)) return;
    floatMenu.value.visible = false;
  };
  document.addEventListener("mousedown", onDocumentMouseDown);

  // Register cleanup
  cleanupFns.push(() => {
    container.removeEventListener("mousedown", onMouseDown);
    document.removeEventListener("mouseup", onMouseUp);
    document.removeEventListener("mousedown", onDocumentMouseDown);
  });
}

// --- Image double-click preview ---
function hashUrl(url: string): string {
  let h = 0;
  for (let i = 0; i < url.length; i++) {
    h = ((h << 5) - h + url.charCodeAt(i)) | 0;
  }
  return `image-preview-${Math.abs(h)}`;
}

function setupImageSelection() {
  if (!editor) return;

  let selectedImg: HTMLElement | null = null;
  let lastFrom = -1;
  let lastTo = -1;

  function clearSelection() {
    if (selectedImg) {
      selectedImg.classList.remove("image-selected");
      selectedImg = null;
    }
  }

  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const imageSelectPlugin = new Plugin({
      key: new PluginKey("image-select"),
      view: () => ({
        update: (view) => {
          const { state } = view;
          const { from, to, empty } = state.selection;

          // Skip if selection hasn't changed (e.g. during scroll)
          if (from === lastFrom && to === lastTo) return;
          lastFrom = from;
          lastTo = to;

          // Clear previous selection
          clearSelection();

          if (empty) return;

          // Check if selection contains an image
          let imgPos = -1;
          state.doc.nodesBetween(from, to, (node: any, pos: number) => {
            if (node.type.name === "image" && imgPos < 0) imgPos = pos;
          });

          // Also check NodeSelection (click on image creates NodeSelection)
          if (imgPos < 0 && state.selection instanceof NodeSelection) {
            const node = (state.selection as any).node;
            if (node?.type?.name === "image") {
              imgPos = from;
            }
          }

          if (imgPos >= 0) {
            // Find the img DOM element
            const dom = view.nodeDOM(imgPos);
            if (dom) {
              if ((dom as HTMLElement).tagName === "IMG") {
                selectedImg = dom as HTMLElement;
              } else {
                const img = (dom as HTMLElement).querySelector?.("img");
                if (img) selectedImg = img as HTMLElement;
              }
              if (selectedImg) selectedImg.classList.add("image-selected");
            }
            // Fallback: search via domAtPos
            if (!selectedImg) {
              const pos = view.domAtPos(imgPos);
              if (pos.node) {
                const el = pos.node instanceof HTMLElement ? pos.node : pos.node.parentElement;
                const img = el?.querySelector("img") || el?.closest("img");
                if (img) {
                  selectedImg = img as HTMLElement;
                  selectedImg.classList.add("image-selected");
                }
              }
            }
          }
        },
      }),
    });

    const newState = view.state.reconfigure({
      plugins: [...view.state.plugins, imageSelectPlugin],
    });
    view.updateState(newState);
  });
}

function setupImagePreview() {
  const container = editorRef.value;
  if (!container) return;

  const onDblClick = async (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.tagName !== "IMG") return;
    const src = (target as HTMLImageElement).src;
    const label = hashUrl(src);

    // If already open for this image, focus it
    const existing = await WebviewWindow.getByLabel(label);
    if (existing) {
      existing.setFocus();
      return;
    }

    // Get image dimensions for window sizing
    const img = new Image();
    img.src = src;
    await new Promise((resolve) => { img.onload = resolve; });

    const minW = 760, minH = 530;
    // Use currentMonitor for multi-display support (window.screen only returns primary monitor)
    let monitorW = 1920, monitorH = 1080;
    try {
      const mon = await currentMonitor();
      if (mon) {
        monitorW = Math.round(mon.size.width / mon.scaleFactor);
        monitorH = Math.round(mon.size.height / mon.scaleFactor);
      }
    } catch {}
    const maxW = Math.floor(monitorW * 0.65);
    const maxH = Math.floor(monitorH * 0.65);
    const w = Math.max(minW, Math.min(maxW, img.naturalWidth));
    const h = Math.max(minH, Math.min(maxH, img.naturalHeight));

    localStorage.setItem(`image-preview-${label}`, src);
    const win = new WebviewWindow(label, {
      url: `index.html?window=image-preview&label=${label}`,
      title: "图片预览",
      width: w,
      height: h,
      decorations: false,
      transparent: false,
      resizable: true,
      center: true,
      visible: false,
    });
    // Show once the preview window signals it's ready
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen(`preview-ready:${label}`, () => {
      win.show();
      unlisten();
    });
  };
  container.addEventListener("dblclick", onDblClick);
  cleanupFns.push(() => container.removeEventListener("dblclick", onDblClick));
}

// --- Image path resolution ---
const imageUrlCache = new Map<string, string>();
let cachedAssetsDir: string | null = null;

async function getAssetsDir(): Promise<string> {
  if (!cachedAssetsDir) {
    cachedAssetsDir = await invoke<string>("get_assets_dir_cmd");
  }
  return cachedAssetsDir;
}

function isLocalFilename(src: string): boolean {
  return !!src && !src.includes("://");
}

function isInvalidImageSrc(src: string): boolean {
  return src.startsWith("file:");
}

async function resolveImageUrl(filename: string): Promise<string> {
  if (imageUrlCache.has(filename)) return imageUrlCache.get(filename)!;
  try {
    const assetsDir = await getAssetsDir();
    const filePath = await join(assetsDir, filename);
    const bytes = await readFile(filePath);
    const blob = new Blob([bytes]);
    const url = URL.createObjectURL(blob);
    imageUrlCache.set(filename, url);
    return url;
  } catch (err) {
    console.error("Failed to resolve image:", filename, err);
    return filename;
  }
}

function setupImageResolution() {
  const container = editorRef.value;
  if (!container) return;

  // Track already-resolved srcs to prevent observer re-entry
  const resolvedSrcs = new Set<string>();

  // Prevent native browser image dragging (from editor)
  const onDragStart = (e: Event) => {
    if ((e.target as HTMLElement).tagName === "IMG") e.preventDefault();
  };
  container.addEventListener("dragstart", onDragStart);

  // Handle HTML5 drop for images from external apps (WeChat, web pages, etc.)
  const onDrop = async (e: DragEvent) => {
    const files = e.dataTransfer?.files;
    if (!files?.length) return;

    for (const file of Array.from(files)) {
      if (file.type.startsWith("image/")) {
        e.preventDefault();
        e.stopPropagation();
        await saveAndInsertImage(file);
        return;
      }
    }
  };
  container.addEventListener("drop", onDrop);

  const onDragOver = (e: DragEvent) => {
    e.preventDefault();
  };
  container.addEventListener("dragover", onDragOver);

  // Process a single image element - only resolve src, no-drag handled by CSS
  function processImage(imgEl: HTMLImageElement) {
    const src = imgEl.getAttribute("src");
    if (src && isLocalFilename(src) && !resolvedSrcs.has(src)) {
      resolvedSrcs.add(src);
      resolveImageUrl(src).then((url) => { if (url !== src) imgEl.setAttribute("src", url); });
    }
  }

  // Initial resolve - only process images that need it
  container.querySelectorAll("img").forEach((img) => processImage(img as HTMLImageElement));

  // Watch for new or updated images
  const observer = new MutationObserver((mutations) => {
    for (const m of mutations) {
      if (m.type === "attributes" && m.target instanceof HTMLImageElement) {
        const imgEl = m.target;
        const src = imgEl.getAttribute("src");
        if (!src) continue;
        if (isInvalidImageSrc(src)) {
          imgEl.remove();
          continue;
        }
        if (!resolvedSrcs.has(src)) {
          processImage(imgEl);
        }
      }
      for (const node of m.addedNodes) {
        if (node instanceof HTMLElement) {
          if (node.tagName === "IMG") {
            const imgEl = node as HTMLImageElement;
            const src = imgEl.getAttribute("src");
            if (src && isInvalidImageSrc(src)) { imgEl.remove(); continue; }
            processImage(imgEl);
          } else {
            node.querySelectorAll("img").forEach((img) => {
              const imgEl = img as HTMLImageElement;
              const src = imgEl.getAttribute("src");
              if (src && isInvalidImageSrc(src)) { imgEl.remove(); return; }
              processImage(imgEl);
            });
          }
        }
      }
    }
  });
  observer.observe(container, { childList: true, subtree: true, attributes: true, attributeFilter: ["src"] });

  return () => {
    observer.disconnect();
    container.removeEventListener("dragstart", onDragStart);
    container.removeEventListener("drop", onDrop);
    container.removeEventListener("dragover", onDragOver);
  };
}

// --- Image paste ---
const { toast } = useToast();

async function saveAndInsertImage(file: File) {
  try {
    const arrayBuffer = await file.arrayBuffer();
    const uint8 = new Uint8Array(arrayBuffer);
    const ext = file.type.split("/")[1] || "png";
    const tempName = `paste_${Date.now()}.${ext}`;

    const sysTempDir = await tempDir();
    const tempPath = await join(sysTempDir, tempName);

    await writeFile(tempPath, uint8);

    const image = await invoke<{ path: string }>("save_image", {
      noteId: props.noteId || null,
      sourcePath: tempPath,
    });

    if (image?.path) {
      const filename = image.path.split(/[/\\]/).pop() || tempName;
      // Pre-cache the blob URL so the image shows immediately
      // Reuse the existing uint8 data we already have in memory — avoid redundant readFile
      try {
        const blob = new Blob([uint8]);
        const blobUrl = URL.createObjectURL(blob);
        imageUrlCache.set(filename, blobUrl);
      } catch {}
      const markdown = `![${filename}](${filename})\n`;
      if (editor) {
        editor.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          const parser = ctx.get(parserCtx);
          if (!parser) return;
          const doc = parser(markdown);
          if (!doc) return;
          const { state } = view;
          const { selection } = state;
          const tr = state.tr.replaceWith(selection.from, selection.to, doc.content);
          const endPos = selection.from + doc.content.size;
          tr.setSelection(TextSelection.create(tr.doc, endPos));
          view.dispatch(tr);
          view.focus();
        });
      }
    }
  } catch (err) {
    console.error("Failed to save pasted image:", err);
    toast("图片保存失败");
  }
}

// --- Composables ---
const editorRefForSync = ref<Editor | null>(null) as unknown as Ref<Editor | null>;

const { addKeymapPlugin, cleanup: cleanupShortcuts } = useEditorShortcuts(editorRefForSync);
const { setupListener } = useEditorSync({
  editor: editorRefForSync,
  getModelValue: () => props.modelValue,
  emit,
});

// --- Readonly ---
function updateReadonly() {
  if (!editor) return;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    view.setProps({
      editable: () => !props.readonly,
    });
    if (!props.readonly) {
      view.focus();
    }
  });
}

watch(() => props.readonly, updateReadonly);

// --- Expose for toolbar ---
let headingLevel = 1;

function isInList(typeName: string): boolean {
  if (!editor) return false;
  let result = false;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const { $from } = view.state.selection;
    for (let d = $from.depth; d >= 0; d--) {
      if ($from.node(d).type.name === typeName) { result = true; break; }
    }
  });
  return result;
}

function toggleList(listType: "bullet" | "ordered") {
  if (!editor) return;
  const targetType = listType === "bullet" ? "bullet_list" : "ordered_list";
  const inTarget = isInList(targetType);
  const inOther = isInList(listType === "bullet" ? "ordered_list" : "bullet_list");

  if (inTarget) {
    liftListItemCommand.run();
  } else {
    if (inOther) liftListItemCommand.run();
    if (listType === "bullet") wrapInBulletListCommand.run();
    else wrapInOrderedListCommand.run();
  }
}

function execCommand(command: string, payload?: string) {
  if (!editor) return;

  // Block formatting on image selection
  const textCommands = new Set(["bold", "italic", "strikethrough", "underline", "heading", "fontColor"]);
  const listCommands = new Set(["bulletList", "orderedList", "taskList"]);
  if (textCommands.has(command) || listCommands.has(command)) {
    let hasImage = false;
    let blockedCtx: "table" | "codeblock" | "blockquote" | null = null;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state } = view;
      const { from, to } = state.selection;
      state.doc.nodesBetween(from, to, (node: any) => {
        if (node.type.name === "image") hasImage = true;
      });
      const { $from } = state.selection;
      for (let d = $from.depth; d >= 0; d--) {
        const name = $from.node(d).type.name;
        if (name === "table") { blockedCtx = "table"; break; }
        if (name === "code_block" || name === "codeblock") { blockedCtx = "codeblock"; break; }
        if (name === "blockquote") { blockedCtx = "blockquote"; break; }
      }
    });
    if (textCommands.has(command) && hasImage) { toast("请框选文本后操作！"); return; }
    if (blockedCtx) {
      const msg = blockedCtx === "table" ? "表格内不支持此操作！" : blockedCtx === "codeblock" ? "代码块内不支持此操作！" : "引用块内不支持此操作！";
      toast(msg); return;
    }
  }

  const commands: Record<string, () => void> = {
    bold: () => {
      editor!.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        const { empty } = state.selection;
        if (empty) return;
        toggleStrongCommand.run();
      });
    },
    italic: () => toggleEmphasisCommand.run(),
    strikethrough: () => {
      editor!.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        const { empty } = state.selection;
        if (empty) return;
        toggleStrikethroughCommand.run();
      });
    },
    underline: () => {
      editor!.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        const { empty } = state.selection;
        if (empty) return;
        const commands = ctx.get(commandsCtx);
        commands.call(toggleUnderlineCommand.key);
      });
    },
    code: () => toggleInlineCodeCommand.run(),
    heading: () => { headingLevel = (headingLevel + 1) % 4; wrapInHeadingCommand.run(headingLevel || 0); },
    bulletList: () => toggleList("bullet"),
    orderedList: () => toggleList("ordered"),
    taskList: () => {
      let inTask = false;
      editor!.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const { $from } = view.state.selection;
        for (let d = $from.depth; d >= 0; d--) {
          if ($from.node(d).type.name === "list_item" && $from.node(d).attrs.checked != null) {
            inTask = true; break;
          }
        }
      });
      if (inTask) { liftListItemCommand.run(); return; }
      if (!isInList("list_item")) wrapInBulletListCommand.run();
      editor!.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        const { $from } = state.selection;
        for (let d = $from.depth; d >= 0; d--) {
          if ($from.node(d).type.name === "list_item" && $from.node(d).attrs.checked == null) {
            const pos = $from.before(d);
            const node = $from.node(d);
            const tr = state.tr.setNodeMarkup(pos, undefined, { ...node.attrs, checked: false });
            view.dispatch(tr); break;
          }
        }
      });
    },
    fontColor: () => {
      editor!.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        const { $from, empty } = state.selection;
        if (empty) return;
        for (let d = $from.depth; d >= 0; d--) {
          const node = $from.node(d);
          if (node.isBlock) {
            const from = $from.before(d);
            const to = $from.after(d);
            const decos = colorDecoKey.getState(state) as DecorationSet;
            const existing = decos?.find(from, to) ?? [];
            const hasColor = existing.length > 0;
            const allFound = decos?.find(0, state.doc.content.size) ?? [];
            let ranges: ColorRange[] = allFound.map((d: any) => ({
              from: d.from, to: d.to, color: "#ef4444",
            }));
            if (hasColor) ranges = ranges.filter((r) => !(r.from === from && r.to === to));
            else ranges.push({ from, to, color: "#ef4444" });
            const tr = state.tr.setMeta(COLOR_META, { ranges });
            view.dispatch(tr);
            emit("update:colorRanges", ranges); break;
          }
        }
      });
    },
    deleteImage: () => {
      if (!payload) return;
      editor!.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const { state } = view;
        // payload is the blob URL from the <img> element
        // Find the filename by reverse-looking up the imageUrlCache
        let targetFilename = "";
        for (const [filename, blobUrl] of imageUrlCache.entries()) {
          if (blobUrl === payload) { targetFilename = filename; break; }
        }
        if (!targetFilename) {
          // Fallback: try to extract filename from payload
          targetFilename = payload.split(/[/\\]/).pop() || payload;
        }
        state.doc.descendants((node: any, pos: number) => {
          if (node.type.name === "image") {
            const src = node.attrs.src || "";
            if (src === targetFilename) {
              const tr = state.tr.delete(pos, pos + node.nodeSize);
              // Place cursor at the deletion point, collapse selection
              tr.setSelection(TextSelection.create(tr.doc, Math.min(pos, tr.doc.content.size)));
              view.dispatch(tr);
              return false;
            }
          }
        });
      });
    },
    paste: () => {
      readText().then((text) => {
        if (!text) return;
        editor!.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          const { state } = view;
          const tr = state.tr.insertText(text, state.selection.from, state.selection.to);
          view.dispatch(tr);
        });
      });
    },
    pastePlain: () => {
      readText().then((text) => {
        if (!text) return;
        const plain = text.replace(/<[^>]+>/g, "").replace(/[\r\n]+/g, " ");
        editor!.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          const { state } = view;
          const tr = state.tr.insertText(plain, state.selection.from, state.selection.to);
          view.dispatch(tr);
        });
      });
    },
  };
  commands[command]?.();
}

async function insertImageFromPath(fullPath: string) {
  try {
    const filename = fullPath.split(/[/\\]/).pop() || fullPath;
    try {
      const assetsDir = await getAssetsDir();
      const filePath = await join(assetsDir, filename);
      const bytes = await readFile(filePath);
      const blob = new Blob([bytes]);
      const blobUrl = URL.createObjectURL(blob);
      imageUrlCache.set(filename, blobUrl);
    } catch {}
    const markdown = `![${filename}](${filename})\n`;
    if (editor) {
      editor.action((ctx) => {
        const view = ctx.get(editorViewCtx);
        const parser = ctx.get(parserCtx);
        if (!parser) return;
        const doc = parser(markdown);
        if (!doc) return;
        const { state } = view;
        const { selection } = state;
        const tr = state.tr.replaceWith(selection.from, selection.to, doc.content);
        const endPos = selection.from + doc.content.size;
        tr.setSelection(TextSelection.create(tr.doc, endPos));
        view.dispatch(tr);
        view.focus();
      });
    }
  } catch (err) {
    console.error("Failed to insert image from path:", err);
    toast("图片插入失败");
  }
}

function focus() {
  if (!editor) return;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    view.focus();
  });
}

function insertFileLink(name: string, path: string) {
  if (!editor) return;
  const normalizedPath = path.replace(/\\/g, "/");
  const encodedPath = encodeURI(normalizedPath);
  const markdown = `[${name}](file:///${encodedPath})\n`;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const parser = ctx.get(parserCtx);
    if (!parser) return;
    const doc = parser(markdown);
    if (!doc) return;
    const { state } = view;
    const { selection } = state;
    const tr = state.tr.replaceWith(selection.from, selection.to, doc.content);
    const endPos = selection.from + doc.content.size;
    tr.setSelection(TextSelection.create(tr.doc, endPos));
    view.dispatch(tr);
    view.focus();
  });
}

async function copyImage(blobUrl: string) {
  // Reverse-lookup the stored filename from the blob URL (same pattern as
  // deleteImage). The <img> DOM src is a blob: URL but the ProseMirror doc
  // attrs keep the original filename stored in imageUrlCache.
  let filename = "";
  for (const [fn, url] of imageUrlCache.entries()) {
    if (url === blobUrl) { filename = fn; break; }
  }
  if (!filename) {
    // Fallback: if somehow the src is a raw filename (not a blob URL)
    filename = blobUrl.split(/[/\\]/).pop() || blobUrl;
  }
  const assetsDir = await getAssetsDir();
  const sourcePath = await join(assetsDir, filename);
  await invoke("copy_image_to_clipboard", { sourcePath });
  toast("图片已复制~");
}

function hideFloatMenu() {
  floatMenu.value.visible = false;
}

// --- Find functions ---

// Manually scroll .milkdown-container so that the given position is centered in
// view. ProseMirror's built-in tr.scrollIntoView() often fails in Tauri /
// WebView because the scrollable ancestor (.milkdown-container) is one
// wrapper above view.dom (.editor), and the internal ancestor detection can
// miss it.
function scrollToMatch(pos: number) {
  if (!editor) return;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const container = editorRef.value;
    if (!container) return;
    const coords = view.coordsAtPos(pos);
    // Relative to the scrollable container
    const containerRect = container.getBoundingClientRect();
    const offsetTop = coords.top - containerRect.top + container.scrollTop;
    // Center the match vertically
    container.scrollTop = offsetTop - container.clientHeight / 2 + 20;
  });
}

function find(query: string) {
  if (!editor) return 0;
  let count = 0;
  let firstFrom = -1;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const ranges = searchDoc(view.state.doc, query);
    count = ranges.length;
    const currentIndex = ranges.length ? 0 : -1;
    const tr = view.state.tr.setMeta(FIND_META, { ranges, currentIndex, query });
    if (ranges.length) {
      firstFrom = ranges[0].from;
      tr.setSelection(TextSelection.create(view.state.doc, ranges[0].from, ranges[0].to));
    }
    view.dispatch(tr);
  });
  if (firstFrom >= 0) scrollToMatch(firstFrom);
  return count;
}

function findNext() {
  if (!editor) return;
  let targetPos = -1;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const state = findHighlightKey.getState(view.state);
    if (!state || !state.ranges.length) return;
    const next = (state.currentIndex + 1) % state.ranges.length;
    const r = state.ranges[next];
    targetPos = r.from;
    const tr = view.state.tr
      .setMeta(FIND_META, { ...state, currentIndex: next })
      .setSelection(TextSelection.create(view.state.doc, r.from, r.to));
    view.dispatch(tr);
  });
  if (targetPos >= 0) scrollToMatch(targetPos);
}

function findPrev() {
  if (!editor) return;
  let targetPos = -1;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const state = findHighlightKey.getState(view.state);
    if (!state || !state.ranges.length) return;
    const prev = (state.currentIndex - 1 + state.ranges.length) % state.ranges.length;
    const r = state.ranges[prev];
    targetPos = r.from;
    const tr = view.state.tr
      .setMeta(FIND_META, { ...state, currentIndex: prev })
      .setSelection(TextSelection.create(view.state.doc, r.from, r.to));
    view.dispatch(tr);
  });
  if (targetPos >= 0) scrollToMatch(targetPos);
}

function clearFind() {
  if (!editor) return;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const tr = view.state.tr.setMeta(FIND_META, { ranges: [], currentIndex: -1, query: "" });
    view.dispatch(tr);
  });
}

function replaceCurrent(replaceText: string): { count: number; index: number } | null {
  if (!editor) return null;
  let result: { count: number; index: number } | null = null;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const state = findHighlightKey.getState(view.state);
    if (!state || !state.ranges.length || state.currentIndex < 0) return;
    const r = state.ranges[state.currentIndex];
    const replaceEnd = r.from + replaceText.length;
    const tr = view.state.tr.replaceWith(r.from, r.to, view.state.schema.text(replaceText));
    // Re-search after replace
    const newRanges = searchDoc(tr.doc, state.query);
    // Find the next match that starts at or after the end of the replaced text
    let newIndex = -1;
    for (let i = 0; i < newRanges.length; i++) {
      if (newRanges[i].from >= replaceEnd) {
        newIndex = i;
        break;
      }
    }
    // If no match after, wrap to beginning
    if (newIndex < 0) newIndex = 0;
    tr.setMeta(FIND_META, { ranges: newRanges, currentIndex: newIndex, query: state.query });
    if (newRanges.length && newIndex >= 0) {
      const nr = newRanges[newIndex];
      tr.setSelection(TextSelection.create(tr.doc, nr.from, nr.to));
      scrollToMatch(nr.from);
    }
    view.dispatch(tr);
    result = { count: newRanges.length, index: newIndex };
  });
  return result;
}

function replaceAll(replaceText: string): number {
  if (!editor) return 0;
  let count = 0;
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const state = findHighlightKey.getState(view.state);
    if (!state || !state.ranges.length) return;
    // Replace from end to start to maintain position validity
    const ranges = [...state.ranges].reverse();
    let tr = view.state.tr;
    for (const r of ranges) {
      tr = tr.replaceWith(r.from, r.to, view.state.schema.text(replaceText));
      count++;
    }
    tr.setMeta(FIND_META, { ranges: [], currentIndex: -1, query: "" });
    view.dispatch(tr);
  });
  return count;
}

function getSelectedText(): string {
  if (!editor) return "";
  let text = "";
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const { state } = view;
    const { from, to, empty } = state.selection;
    if (!empty) {
      text = state.doc.textBetween(from, to, " ");
    }
  });
  return text;
}

defineExpose({ execCommand, insertImageFromPath, focus, copyImage, find, findNext, findPrev, clearFind, replaceCurrent, replaceAll, getSelectedText, hideFloatMenu });

// --- Editor lifecycle ---
async function createEditor() {
  if (!editorRef.value) return;

  // Setup image resolution BEFORE Milkdown renders, so broken icons never appear
  const imageResolutionCleanup = setupImageResolution();
  if (imageResolutionCleanup) {
    cleanupFns.push(imageResolutionCleanup);
  }

  editor = await Editor.make()
    .config((ctx) => {
      ctx.set(rootCtx, editorRef.value!);
      ctx.set(defaultValueCtx, props.modelValue);
    })
    .use(commonmark)
    .use(gfm)
    .use(nord)
    .use(history)
    .use(clipboard)
    .use(listener)
    .use(underlineAttr)
    .use(underlineSchema)
    .use(toggleUnderlineCommand)
    .create();

  editorRefForSync.value = editor;

  // Setup sync listener
  setupListener(editor);

  // Add link decoration plugin + color plugin
  editor.action((ctx) => {
    const view = ctx.get(editorViewCtx);
    const linkPlugin = createLinkPlugin();
    const colorPlugin = createColorPlugin(props.colorRanges || []);
    const codeHighlightPlugin = createCodeHighlightPlugin();
    const trailingParagraphPlugin = createTrailingParagraphPlugin();
    const findPlugin = createFindHighlightPlugin();
    const state = view.state.reconfigure({
      plugins: [...view.state.plugins, linkPlugin, colorPlugin, codeHighlightPlugin, trailingParagraphPlugin, findPlugin],
    });
    view.updateState(state);
  });

  // Add keyboard shortcuts
  addKeymapPlugin(editor);

  // Handle copy in capture phase — copy image when image node is selected
  const onCopy = async (e: ClipboardEvent) => {
    if (!editor) return;
    let imageSrc = "";
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state } = view;
      const { from, to } = state.selection;
      // Check if selection contains an image node
      state.doc.nodesBetween(from, to, (node: any) => {
        if (node.type.name === "image" && node.attrs.src) {
          imageSrc = node.attrs.src;
        }
      });
    });
    if (imageSrc) {
      e.preventDefault();
      e.stopPropagation();
      try {
        // imageSrc is the stored filename in the ProseMirror doc attrs.
        // The <img> DOM src is swapped to a blob: URL by setupImageResolution,
        // but the doc attrs keep the raw filename, so resolve to an absolute
        // path under the assets dir and let the backend decode any format.
        let filename = imageSrc;
        // Fallback: if a blob URL leaked into attrs, reverse-lookup the filename
        if (imageSrc.includes("://")) {
          for (const [fn, blobUrl] of imageUrlCache.entries()) {
            if (blobUrl === imageSrc) { filename = fn; break; }
          }
        }
        const assetsDir = await getAssetsDir();
        const sourcePath = await join(assetsDir, filename);
        await invoke("copy_image_to_clipboard", { sourcePath });
        toast("图片已复制~");
      } catch (err) {
        console.error("Failed to copy image:", err);
        toast("图片复制失败");
      }
    }
  };
  editorRef.value?.addEventListener("copy", onCopy, true);
  cleanupFns.push(() => editorRef.value?.removeEventListener("copy", onCopy, true));

  // Handle all paste in capture phase — before Milkdown clipboard plugin
  const onPaste = async (e: ClipboardEvent) => {
    const files = e.clipboardData?.files;
    const imageFile = files && Array.from(files).find((f) => f.type.startsWith("image/"));

    // Image file paste (screenshot, etc.) — save to disk and insert markdown
    if (imageFile) {
      e.preventDefault();
      e.stopPropagation();
      await saveAndInsertImage(imageFile);
      return;
    }

    // HTML with file:// / blob: image URLs (Windows clipboard history) — strip invalid images
    const html = e.clipboardData?.getData("text/html");
    if (html && /<img[^>]+src=["'](?:file:|blob:)/i.test(html)) {
      e.preventDefault();
      e.stopPropagation();
      const clean = html.replace(/<img[^>]+src=["'][^"']*(?:file:|blob:)[^"']*["'][^>]*\/?>/gi, "");
      if (clean && editor) {
        editor.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          const parser = ctx.get(parserCtx);
          if (!parser) return;
          const doc = parser(clean);
          if (!doc) return;
          const { state } = view;
          const tr = state.tr.replaceWith(state.selection.from, state.selection.to, doc.content);
          view.dispatch(tr);
        });
      }
    }
    // Normal text/HTML paste — let Milkdown handle it
  };
  editorRef.value?.addEventListener("paste", onPaste, true);
  cleanupFns.push(() => editorRef.value?.removeEventListener("paste", onPaste, true));

  // Handle file drag & drop via Tauri's native drag-drop API
  try {
    const appWindow = getCurrentWebviewWindow();
    fileDropUnlisten = await appWindow.onDragDropEvent(async (event) => {
      if (event.payload.type !== "drop") return;
      const paths = event.payload.paths;
      if (!paths?.length) return;
      for (const path of paths) {
        const ext = path.split(".").pop()?.toLowerCase() || "";
        // Image files - save and insert as image
        if (["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg"].includes(ext)) {
          try {
            const filename = path.split(/[/\\]/).pop() || path;
            const bytes = await readFile(path);
            await saveAndInsertImage(new File([bytes], filename, { type: `image/${ext}` }));
          } catch (err) {
            console.error("Failed to insert dropped image:", err);
            toast("图片插入失败");
          }
          continue;
        }
        // Other files - insert as file link
        const name = path.split(/[/\\]/).pop() || path;
        insertFileLink(name, path);
      }
    });
  } catch (e) {
    console.error("Failed to setup file drop handler:", e);
  }

  // Setup image double-click preview
  setupImagePreview();

  // Setup image selection style
  setupImageSelection();

  // Setup checkbox click
  setupCheckboxClick();

  // Setup float selection menu
  setupFloatMenu();

  // Move cursor to end-of-doc only when clicking the empty space *below* the
  // last block (e.g. below a trailing code block). ProseMirror otherwise leaves
  // the cursor inside the last block because there is no node under the click.
  // We compare the click's Y against the bottom edge of the last block node —
  // clicks above that (paragraph margins, list indents, etc.) must NOT trigger,
  // otherwise clicking normal whitespace jumps the cursor to the doc end.
  const onContainerClick = (e: MouseEvent) => {
    if (props.readonly || !editor) return;
    const view0 = (() => {
      let v: any = null;
      editor!.action((ctx) => { v = ctx.get(editorViewCtx); });
      return v;
    })();
    if (!view0) return;
    const { doc } = view0.state;
    if (!doc.lastChild) return;
    const lastPos = doc.content.size - doc.lastChild.nodeSize;
    // Bottom edge of the last block, in viewport coords.
    let endOfLast = lastPos + doc.lastChild.nodeSize;
    const lastBottom = view0.coordsAtPos(Math.min(endOfLast, doc.content.size)).bottom;
    // Only act when the click is strictly below the last block (the trailing
    // gap), not when it lands on actual content or normal inter-paragraph space.
    if (e.clientY < lastBottom) return;
    editor!.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const end = view.state.doc.content.size;
      const tr = view.state.tr.setSelection(TextSelection.create(view.state.doc, end));
      view.dispatch(tr);
      view.focus();
    });
  };
  editorRef.value?.addEventListener("click", onContainerClick);
  cleanupFns.push(() => editorRef.value?.removeEventListener("click", onContainerClick));

  updateReadonly();

  // Watch for external colorRanges changes (e.g. loaded from DB after mount)
  watch(() => props.colorRanges, (newRanges) => {
    if (!editor || !newRanges) return;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const tr = view.state.tr.setMeta(COLOR_META, { ranges: newRanges });
      view.dispatch(tr);
    });
  }, { deep: true });
}

onMounted(() => {
  createEditor();
});

onUnmounted(() => {
  fileDropUnlisten?.();
  // Revoke all Blob URLs to prevent memory leaks
  for (const url of imageUrlCache.values()) {
    if (url.startsWith("blob:")) {
      URL.revokeObjectURL(url);
    }
  }
  imageUrlCache.clear();
  cleanupFns.forEach(fn => fn());
  cleanupFns.length = 0;
  cleanupShortcuts();
  editor?.destroy();
});
</script>

<style>
.milkdown-container {
  overflow-y: auto;
  padding: 0 15px 15px 15px;
  height: 100%;
  font-family: "JetBrains Mono", "PingFang SC", "Microsoft YaHei", sans-serif;
  position: relative;
  z-index: 1;
}

.milkdown-container .editor {
  outline: none;
  min-height: 200px;
  color: rgba(255, 255, 255, 0.9);
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}

.milkdown-container h1 { font-size: 1.5em; font-weight: 700; margin: 0.8em 0 0.4em; }
.milkdown-container h2 { font-size: 1.3em; font-weight: 700; margin: 0.6em 0 0.3em; }
.milkdown-container h3 { font-size: 1.1em; font-weight: 700; margin: 0.5em 0 0.2em; }
.milkdown-container p { margin: 0.25em 0; }
.milkdown-container strong { font-weight: 800; letter-spacing: 0.2px; }
.milkdown-container del { text-decoration: line-through; text-decoration-thickness: 1px; text-decoration-color: rgba(255, 255, 255, 0.65); opacity: 0.65; }
.milkdown-container u { text-decoration: underline; text-underline-offset: 3px; text-decoration-color: rgba(255, 255, 255, 0.6); }
.milkdown-container code { background: rgba(255,255,255,0.1); padding: 0.15em 0.4em; border-radius: 4px; font-size: 0.9em; }
/* Code block — Xcode Dark token palette over the original translucent background */
.milkdown-container pre {
  background: rgba(0,0,0,0.25);
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  color: #DFDFE0;
  font-size: 13px;
  line-height: 1.55;
}
.milkdown-container pre code {
  background: none;
  padding: 0;
  color: inherit;
  font-size: inherit;
}
/* Xcode Dark token palette */
.milkdown-container pre code .token.comment,
.milkdown-container pre code .token.prolog,
.milkdown-container pre code .token.doctype,
.milkdown-container pre code .token.cdata { color: #7C8995; font-style: italic; }
.milkdown-container pre code .token.punctuation { color: #DEE2EC; }
.milkdown-container pre code .token.property,
.milkdown-container pre code .token.tag,
.milkdown-container pre code .token.constant,
.milkdown-container pre code .token.symbol,
.milkdown-container pre code .token.deleted { color: #9EF1DD; }
.milkdown-container pre code .token.boolean,
.milkdown-container pre code .token.number { color: #D0BF69; }
.milkdown-container pre code .token.selector,
.milkdown-container pre code .token.attr-name,
.milkdown-container pre code .token.string,
.milkdown-container pre code .token.char,
.milkdown-container pre code .token.builtin,
.milkdown-container pre code .token.inserted { color: #FF8170; }
.milkdown-container pre code .token.operator,
.milkdown-container pre code .token.entity,
.milkdown-container pre code .token.url,
.milkdown-container pre code .language-css .token.string,
.milkdown-container pre code .style .token.string { color: #FF7AB2; }
.milkdown-container pre code .token.atrule,
.milkdown-container pre code .token.attr-value,
.milkdown-container pre code .token.keyword { color: #FF7AB2; }
.milkdown-container pre code .token.function,
.milkdown-container pre code .token.class-name { color: #A167E6; }
.milkdown-container pre code .token.regex,
.milkdown-container pre code .token.important,
.milkdown-container pre code .token.variable { color: #9EFFFF; }
.milkdown-container pre code .token.interpolation,
.milkdown-container pre code .token.interpolation-punctuation { color: #9EF1DD; }
.milkdown-container pre code .token.namespace { opacity: 0.7; }
.milkdown-container blockquote { border-left: 3px solid rgba(255,255,255,0.2); padding-left: 12px; color: rgba(255,255,255,0.6); margin: 0.5em 0; }
.milkdown-container table { border-collapse: separate; border-spacing: 0; width: 100%; margin: 0.6em 0; border: 1px solid rgba(var(--border), 0.2); border-radius: 8px; overflow: hidden; }
.milkdown-container th, .milkdown-container td { padding: 6px 12px; border-right: 1px solid rgba(var(--border), 0.15); border-bottom: 1px solid rgba(var(--border), 0.15); text-align: left; font-size: 13px; line-height: 1.5; }
.milkdown-container th { background: rgba(255,255,255,0.06); font-weight: 700; color: rgba(255, 255, 255, 0.9); }
.milkdown-container td { background: rgba(255,255,255,0.02); color: rgba(255, 255, 255, 0.8); }
.milkdown-container tr:hover td { background: rgba(255,255,255,0.04); }
.milkdown-container th:last-child, .milkdown-container td:last-child { border-right: none; }
.milkdown-container tr:last-child td { border-bottom: none; }
.milkdown-container hr { border: none; border-top: 1px solid rgba(var(--border), 0.2); margin: 1em 0; }
.milkdown-container a { color: rgb(var(--accent)); text-decoration: none; }
.milkdown-container ul { padding-left: 1.5em; list-style-type: disc; }
.milkdown-container ol { padding-left: 1.5em; list-style-type: decimal; }
.milkdown-container ul:has(> li[data-item-type="task"]),
.milkdown-container ol:has(> li[data-item-type="task"]) { padding-left: 0; }
.milkdown-container li { margin: 0.1em 0; }
.milkdown-container li[data-item-type="task"] {
  list-style: none;
  position: relative;
  padding-left: 22px;
}
.milkdown-container li[data-item-type="task"]::before {
  content: "";
  position: absolute;
  left: 0;
  top: 0.35em;
  width: 14px;
  height: 14px;
  border: 1.5px solid rgba(var(--border), 0.3);
  border-radius: 3px;
  cursor: pointer;
}
.milkdown-container li[data-item-type="task"][data-checked="true"]::before {
  background: rgba(var(--accent-soft), 0.6);
  border-color: rgba(var(--accent-soft), 0.8);
}
.milkdown-container li[data-item-type="task"][data-checked="true"]::after {
  content: "";
  position: absolute;
  left: 3px;
  top: 0.55em;
  width: 8px;
  height: 5px;
  border-left: 1.5px solid white;
  border-bottom: 1.5px solid white;
  transform: rotate(-45deg);
  cursor: pointer;
}
.milkdown-container li[data-item-type="task"][data-checked="true"] > p {
  text-decoration: line-through;
  opacity: 0.5;
}
.milkdown-container img { max-width: 100%; border-radius: 8px; cursor: pointer; -webkit-user-drag: none; user-select: none; }
/* Hide broken images (including icon) before blob URL is set */
.milkdown-container img:not([src*="://"]) { visibility: hidden; height: 0; margin: 0; padding: 0; }
.milkdown-container img.image-selected { box-shadow: 0 0 0 2px rgba(var(--accent-soft), 0.8); }

/* Detected link style */
.milkdown-container .detected-link,
.milkdown-container a[href] {
  color: rgb(var(--accent));
  text-decoration: underline;
  text-underline-offset: 2px;
  cursor: pointer;
}

/* Float selection menu */
.editor-float-menu {
  position: fixed;
  z-index: 10001;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 6px;
  border-radius: 10px;
  background: rgba(var(--bg-panel), 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(var(--border), 0.1);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  animation: float-menu-in 0.15s ease-out;
}
.editor-float-menu button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.55);
  transition: all 0.15s;
  background: none;
  border: none;
  cursor: pointer;
}
.editor-float-menu button:hover {
  color: rgba(255, 255, 255, 0.95);
  background: rgba(var(--interactive), 0.1);
}
.float-menu-divider {
  width: 1px;
  height: 16px;
  background: rgba(var(--interactive), 0.12);
  margin: 0 4px;
}
.stats-popup {
  position: fixed;
  z-index: 10002;
  min-width: 240px;
  max-width: 360px;
  max-height: 260px;
  padding: 12px 16px;
  border-radius: 10px;
  background: rgba(var(--bg-panel), 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(var(--border), 0.1);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  animation: float-menu-in 0.15s ease-out;
  user-select: text;
  display: flex;
  flex-direction: column;
}
.stats-total {
  text-align: center;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(var(--border), 0.08);
  flex-shrink: 0;
}
.stats-details {
  padding: 6px 0;
  border-bottom: 1px solid rgba(var(--border), 0.08);
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}
.stats-detail-item {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  line-height: 1.7;
}
.stats-bottom {
  display: flex;
  align-items: center;
  padding-top: 8px;
  flex-shrink: 0;
}
.stats-bottom-btns { flex: 1; display: flex; justify-content: center; }
.stats-copy-btn {
  padding: 4px 20px;
  border-radius: 6px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
  background: rgba(var(--interactive), 0.08);
  border: none;
  cursor: pointer;
  transition: all 0.15s;
}
.stats-copy-btn:hover {
  color: rgba(255, 255, 255, 0.95);
  background: rgba(var(--interactive), 0.15);
}
.stats-count {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
}
@keyframes float-menu-in {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}

</style>
