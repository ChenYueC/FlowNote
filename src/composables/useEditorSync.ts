// src/composables/useEditorSync.ts
import { type Ref, watch } from "vue";
import { Editor, editorViewCtx, parserCtx } from "@milkdown/core";
import { listenerCtx } from "@milkdown/plugin-listener";

interface UseEditorSyncOptions {
  editor: Ref<Editor | null>;
  getModelValue: () => string;
  emit: (event: "update:modelValue", value: string) => void;
}

export function useEditorSync(options: UseEditorSyncOptions) {
  const { editor, getModelValue, emit } = options;
  let lastEmittedMarkdown = "";

  function setupListener(ed: Editor) {
    ed.action((ctx) => {
      const listenerConfig = ctx.get(listenerCtx);
      listenerConfig.markdownUpdated((_, markdown) => {
        lastEmittedMarkdown = markdown;
        emit("update:modelValue", markdown);
      });
    });
  }

  function syncExternalValue() {
    const val = getModelValue();
    const ed = editor.value;
    if (!ed || !val) return;
    if (val === lastEmittedMarkdown) return;

    ed.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const docText = view.state.doc.textContent;

      // Skip if editor has focus AND already has content (don't disrupt active editing)
      if (view.hasFocus() && docText) return;

      // Update if editor is empty or content differs
      if (!docText || val !== lastEmittedMarkdown) {
        const parser = ctx.get(parserCtx);
        if (parser) {
          const doc = parser(val);
          if (doc) {
            const tr = view.state.tr.replaceWith(
              0,
              view.state.doc.content.size,
              doc.content,
            );
            view.dispatch(tr);
          }
        }
      }
    });
  }

  watch(() => getModelValue(), syncExternalValue);

  return { setupListener, syncExternalValue };
}
