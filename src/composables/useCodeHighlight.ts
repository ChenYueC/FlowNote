/**
 * Code block syntax highlighting for Milkdown code_block nodes.
 *
 * Strategy: a ProseMirror decoration plugin (same shape as createColorPlugin /
 * createLinkPlugin in MilkdownEditor.vue). For every `code_block` node we run
 * Prism.tokenize on its text and emit `Decoration.inline` spans carrying
 * `token <type>` classes. CSS (in MilkdownEditor.vue) paints the Xcode Dark
 * palette over those classes.
 *
 * Languages are imported as side-effect modules bundled at build time — no
 * network request, works fully offline.
 */
import Prism from "prismjs";

// --- Language registration (side-effect imports) ---------------------------
// Order matters: derivative languages must come AFTER their base.
// Prism component files attach grammar to the shared `Prism` global exported
// above, so importing them registers the language as a side effect.
import "prismjs/components/prism-clike";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-typescript";
import "prismjs/components/prism-jsx";
import "prismjs/components/prism-tsx";
import "prismjs/components/prism-css";
import "prismjs/components/prism-json";
import "prismjs/components/prism-markup"; // html / xml
import "prismjs/components/prism-markdown";
import "prismjs/components/prism-python";
import "prismjs/components/prism-rust";
import "prismjs/components/prism-java";
import "prismjs/components/prism-c";
import "prismjs/components/prism-cpp";
import "prismjs/components/prism-go";
import "prismjs/components/prism-bash";
import "prismjs/components/prism-sql";
import "prismjs/components/prism-yaml";
import "prismjs/components/prism-toml";
import "prismjs/components/prism-regex";

import { Plugin, PluginKey } from "@milkdown/prose/state";
import { Decoration, DecorationSet } from "@milkdown/prose/view";

const codeHighlightKey = new PluginKey<DecorationSet>("code-highlight");

/** Map of aliases users might type in a fenced block to a registered Prism id. */
const ALIASES: Record<string, string> = {
  js: "javascript",
  ts: "typescript",
  py: "python",
  rs: "rust",
  sh: "bash",
  shell: "bash",
  zsh: "bash",
  rb: "ruby",
  // ruby not bundled; falls back to plain below
  "c++": "cpp",
  "c#": "csharp",
  cs: "csharp",
  // csharp not bundled; plain fallback
  yml: "yaml",
  md: "markdown",
  html: "markup",
  xml: "markup",
  svg: "markup",
  golang: "go",
};

function resolveLanguage(raw: string): string | null {
  const lang = (raw || "").trim().toLowerCase();
  if (!lang) return null;
  const target = ALIASES[lang] ?? lang;
  return Prism.languages[target] ? target : null;
}

/**
 * Walk a Prism token stream into flat {text, type} ranges.
 * Prism nests tokens (e.g. a string containing interpolation); we flatten and
 * join ancestor types with spaces so the CSS can match either `token string`
 * or `token interpolation`.
 */
interface FlatToken {
  text: string;
  /** Space-joined ancestor types, e.g. "string-interpolation interpolation". */
  type: string;
}

function flattenTokens(tokens: (string | Prism.Token)[], inherited = ""): FlatToken[] {
  const out: FlatToken[] = [];
  for (const t of tokens) {
    if (typeof t === "string") {
      if (t) out.push({ text: t, type: inherited });
      continue;
    }
    // Prism.Token: alias may be string | string[]
    const aliasTypes = t.alias ? (Array.isArray(t.alias) ? t.alias : [t.alias]) : [];
    const type = [inherited, t.type, ...aliasTypes].filter(Boolean).join(" ").trim();
    if (typeof t.content === "string") {
      out.push({ text: t.content, type });
    } else {
      out.push(...flattenTokens(t.content as (string | Prism.Token)[], type));
    }
  }
  return out;
}

function buildHighlightDecorations(doc: any): DecorationSet {
  const decos: Decoration[] = [];

  doc.descendants((node: any, pos: number) => {
    if (node.type.name !== "code_block") return;
    const lang = resolveLanguage(node.attrs?.language);
    if (!lang) return;

    const text = node.textBetween(0, node.content.size, "\n");
    if (!text) return;

    const grammar = Prism.languages[lang]!;
    const tokens = flattenTokens(Prism.tokenize(text, grammar));

    let offset = 0;
    for (const tk of tokens) {
      if (!tk.text) continue;
      const from = pos + 1 + offset; // +1 to step into the <code> text node
      const to = from + tk.text.length;
      if (tk.type) {
        decos.push(Decoration.inline(from, to, { class: `token ${tk.type}` }));
      }
      offset += tk.text.length;
    }
  });

  return DecorationSet.create(doc, decos);
}

/** ProseMirror plugin that highlights code_block nodes via inline decorations. */
export function createCodeHighlightPlugin(): Plugin<DecorationSet> {
  return new Plugin<DecorationSet>({
    key: codeHighlightKey,
    state: {
      init(_, state) {
        return buildHighlightDecorations(state.doc);
      },
      apply(tr, old) {
        // Rebuild when the document changes; otherwise keep decorations mapped
        // through the transaction (handles selection-only / minor edits cheaply).
        if (tr.docChanged) return buildHighlightDecorations(tr.doc);
        return old.map(tr.mapping, tr.doc);
      },
    },
    props: {
      decorations(state) {
        return codeHighlightKey.getState(state);
      },
    },
  });
}
