// src/composables/useEditorShortcuts.ts
// Three-layer command pattern: KeyEvent → normalize → commandId → handlerRegistry
import { type Ref } from "vue";
import { Editor, editorViewCtx } from "@milkdown/core";
import {
  toggleStrongCommand,
  toggleEmphasisCommand,
  wrapInHeadingCommand,
  wrapInBulletListCommand,
  wrapInOrderedListCommand,
  sinkListItemCommand,
  liftListItemCommand,
  splitListItemCommand,
} from "@milkdown/preset-commonmark";
import { toggleStrikethroughCommand } from "@milkdown/preset-gfm";
import { $markAttr, $markSchema, $command } from "@milkdown/utils";
import { toggleMark } from "@milkdown/prose/commands";
import { TextSelection } from "@milkdown/prose/state";
import { Plugin, PluginKey } from "@milkdown/prose/state";
import { registerCommand, executeCommand } from "@/composables/useCommandRegistry";
import { useSettingsStore } from "@/stores/settings";
import { useToast } from "@/composables/useToast";
import { readText } from "@tauri-apps/plugin-clipboard-manager";

// --- Underline mark definition ---
export const underlineAttr = $markAttr("underline");
export const underlineSchema = $markSchema("underline", () => ({
  parseDOM: [
    { tag: "u" },
    { style: "text-decoration", getAttrs: (value: string) => value === "underline" && null },
  ],
  toDOM: () => ["u", 0],
  parseMarkdown: {
    match: (node: any) => node.type === "underline",
    runner: (state: any, node: any, markType: any) => {
      state.openMark(markType);
      state.next(node.children);
      state.closeMark(markType);
    },
  },
  toMarkdown: {
    match: (mark: any) => mark.type.name === "underline",
    runner: (state: any, mark: any) => {
      state.withMark(mark, "underline");
    },
  },
}));
export const toggleUnderlineCommand = $command("ToggleUnderline", (ctx: any) => () => {
  return toggleMark(underlineSchema.type(ctx));
});

export function useEditorShortcuts(_editor: Ref<Editor | null>) {
  let headingLevel = 1;

  function isInList(view: any, typeName: string): boolean {
    const { $from } = view.state.selection;
    for (let d = $from.depth; d >= 0; d--) {
      if ($from.node(d).type.name === typeName) return true;
    }
    return false;
  }

  function toggleList(view: any, listType: "bullet" | "ordered") {
    const targetType = listType === "bullet" ? "bullet_list" : "ordered_list";
    const otherType = listType === "bullet" ? "ordered_list" : "bullet_list";
    const inTarget = isInList(view, targetType);
    const inOther = isInList(view, otherType);

    if (inTarget) {
      liftListItemCommand.run();
    } else {
      if (inOther) liftListItemCommand.run();
      if (listType === "bullet") wrapInBulletListCommand.run();
      else wrapInOrderedListCommand.run();
    }
  }

  // --- Layer 1: Register command handlers (stable, never rebuilt) ---
  let viewRef: any = null;
  const { toast } = useToast();

  function hasImageInSelection(): boolean {
    if (!viewRef) return false;
    let found = false;
    const { from, to } = viewRef.state.selection;
    viewRef.state.doc.nodesBetween(from, to, (node: any) => {
      if (node.type.name === "image") found = true;
    });
    return found;
  }

  function blockedToast(ctx: "table" | "codeblock" | "blockquote"): string {
    if (ctx === "table") return "表格内不支持此操作！";
    if (ctx === "codeblock") return "代码块内不支持此操作！";
    return "引用块内不支持此操作！";
  }

  function getBlockedContext(): "table" | "codeblock" | "blockquote" | null {
    if (!viewRef) return null;
    const { $from } = viewRef.state.selection;
    for (let d = $from.depth; d >= 0; d--) {
      const name = $from.node(d).type.name;
      if (name === "table") return "table";
      if (name === "code_block" || name === "codeblock") return "codeblock";
      if (name === "blockquote") return "blockquote";
    }
    return null;
  }

  registerCommand("bold", () => { if (viewRef && !viewRef.state.selection.empty) toggleStrongCommand.run(); });
  registerCommand("italic", () => toggleEmphasisCommand.run());
  registerCommand("strikethrough", () => { if (viewRef && !viewRef.state.selection.empty) toggleStrikethroughCommand.run(); });
  registerCommand("underline", () => { if (viewRef && !viewRef.state.selection.empty) toggleUnderlineCommand.run(); });
  registerCommand("heading", () => {
    // Cycle: 1 → 2 → 3 → 0 (cancel heading) → 1
    headingLevel = headingLevel >= 3 ? 0 : headingLevel + 1;
    wrapInHeadingCommand.run(headingLevel);
  });
  registerCommand("bulletList", () => { if (viewRef) toggleList(viewRef, "bullet"); });
  registerCommand("orderedList", () => { if (viewRef) toggleList(viewRef, "ordered"); });
  registerCommand("taskList", () => {
    if (!viewRef) return;
    const { $from } = viewRef.state.selection;
    let taskDepth = -1;
    for (let d = $from.depth; d >= 0; d--) {
      if ($from.node(d).type.name === "list_item" && $from.node(d).attrs.checked != null) {
        taskDepth = d; break;
      }
    }
    if (taskDepth >= 0) {
      liftListItemCommand.run();
    } else {
      if (!isInList(viewRef, "list_item")) wrapInBulletListCommand.run();
      const { state } = viewRef;
      const { $from: $f } = state.selection;
      for (let d = $f.depth; d >= 0; d--) {
        if ($f.node(d).type.name === "list_item" && $f.node(d).attrs.checked == null) {
          const pos = $f.before(d);
          const node = $f.node(d);
          const tr = state.tr.setNodeMarkup(pos, undefined, { ...node.attrs, checked: false });
          viewRef.dispatch(tr); break;
        }
      }
    }
  });
  registerCommand("pastePlain", () => {
    if (!viewRef) return;
    readText().then((text) => {
      if (text && viewRef) {
        const { state } = viewRef;
        const tr = state.tr.insertText(text, state.selection.from, state.selection.to);
        viewRef.dispatch(tr);
      }
    }).catch(() => {});
  });

  // --- Layer 2: Shortcut map from settings store ---
  const settingsStore = useSettingsStore();
  const shortcutMap = new Map<string, string>(); // normalizedKey → commandId

  function buildShortcutMap() {
    shortcutMap.clear();
    for (const [commandId, key] of Object.entries(settingsStore.shortcuts)) {
      if (key) shortcutMap.set(key, commandId);
    }
  }
  buildShortcutMap();

  // Rebuild on settings change (via Tauri event → store reload)
  let settingsUnlisten: (() => void) | null = null;
  import("@tauri-apps/api/event").then(({ listen }) => {
    listen("settings:updated", () => {
      setTimeout(buildShortcutMap, 50); // small delay to let store reload
    }).then((fn) => { settingsUnlisten = fn; });
  }).catch(() => {});

  // --- Layer 3: Keydown handler ---
  function createKeymapPlugin(): Plugin {
    function handleKeyDown(_view: any, event: KeyboardEvent): boolean {
      viewRef = _view;

      // Normalize key combo
      let normalized = "";
      if (event.shiftKey && (event.ctrlKey || event.metaKey)) {
        normalized = `shift-mod-${event.key.toLowerCase()}`;
      } else if (event.ctrlKey || event.metaKey) {
        normalized = `mod-${event.key.toLowerCase()}`;
      } else if (event.shiftKey && event.key === "Tab") {
        normalized = "shift-tab";
      } else if (event.key === "Tab") {
        normalized = "tab";
      } else if (event.key === "Enter") {
        normalized = "enter";
      } else {
        return false;
      }

      // Layer 3a: Configurable shortcuts via command registry
      const commandId = shortcutMap.get(normalized);
      if (commandId) {
        // Block text formatting shortcuts when image is selected
        const textCmds = new Set(["bold", "italic", "strikethrough", "underline", "heading", "fontColor"]);
        const listCmds = new Set(["bulletList", "orderedList", "taskList"]);
        if (textCmds.has(commandId)) {
          if (hasImageInSelection()) {
            toast("请框选文本后操作！");
            return true;
          }
          const blocked = getBlockedContext();
          if (blocked) {
            toast(blockedToast(blocked));
            return true;
          }
        } else if (listCmds.has(commandId)) {
          const blocked = getBlockedContext();
          if (blocked) {
            toast(blockedToast(blocked));
            return true;
          }
        }
        return executeCommand(commandId);
      }

      // Layer 3b: Non-configurable key handlers (Tab/Enter/Shift+Tab / Ctrl+A)
      switch (normalized) {
        case "mod-a": {
          // Ctrl+A inside a table cell → select that cell's content.
          // Ctrl+A inside a code_block → select the entire code block content.
          // Otherwise → let ProseMirror handle normal Ctrl+A (select all).
          const { state } = _view;
          const { $from } = state.selection;
          let blockPos = -1;
          let blockNode: any = null;
          for (let d = $from.depth; d >= 0; d--) {
            const name = $from.node(d).type.name;
            if (name === "table_cell" || name === "table_header") {
              // Select the current table cell
              blockPos = $from.before(d);
              blockNode = $from.node(d);
              break;
            }
            if (name === "code_block" || name === "codeblock") {
              // Select the entire code block content
              blockPos = $from.before(d);
              blockNode = $from.node(d);
              break;
            }
          }
          if (blockNode) {
            const from = blockPos + 1;
            const to = blockPos + blockNode.nodeSize - 1;
            const tr = state.tr.setSelection(TextSelection.create(state.doc, from, to));
            _view.dispatch(tr);
            return true;
          }
          return false;
        }
        case "tab": {
          const { $from } = _view.state.selection;
          if ($from.node(-1)?.type.name === "list_item") {
            sinkListItemCommand.run();
            return true;
          }
          const { state } = _view;
          const tr = state.tr.insertText("    ", state.selection.from, state.selection.to);
          _view.dispatch(tr);
          return true;
        }
        case "shift-tab": {
          const { $from } = _view.state.selection;
          if ($from.node(-1)?.type.name === "list_item") {
            liftListItemCommand.run();
            return true;
          }
          const { state: st } = _view;
          const { from } = st.selection;
          const lineText = st.doc.textBetween(Math.max(0, from - 4), from, "\n", "�");
          const spaces = lineText.match(/ {1,4}$/);
          if (spaces) {
            const tr = st.tr.delete(from - spaces[0].length, from);
            _view.dispatch(tr);
          }
          return true;
        }
        case "enter": {
          const { $from } = _view.state.selection;
          const listItem = $from.node(-1);
          if (listItem?.type.name !== "list_item") return false;
          if (listItem.textContent === "") {
            liftListItemCommand.run();
            return true;
          }
          splitListItemCommand.run();
          return true;
        }
        default:
          return false;
      }
    }

    return new Plugin({
      key: new PluginKey("editor-shortcuts"),
      props: { handleKeyDown },
    });
  }

  function addKeymapPlugin(ed: Editor) {
    ed.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      viewRef = view;
      const plugin = createKeymapPlugin();
      // Insert at the beginning so our handler runs before Milkdown's built-in keymaps
      const state = view.state.reconfigure({
        plugins: [plugin, ...view.state.plugins],
      });
      view.updateState(state);
    });
  }

  function cleanup() {
    if (settingsUnlisten) { settingsUnlisten(); settingsUnlisten = null; }
  }

  return { addKeymapPlugin, cleanup };
}
