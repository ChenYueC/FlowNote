// src/composables/useImagePaste.ts
import { type Ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeFile } from "@tauri-apps/plugin-fs";
import { tempDir, join } from "@tauri-apps/api/path";
import { useToast } from "@/composables/useToast";

interface UseImagePasteOptions {
  noteId: string;
  containerRef: Ref<HTMLElement | undefined>;
  insertMarkdown: (text: string) => void;
}

export function useImagePaste(options: UseImagePasteOptions) {
  const { noteId, containerRef, insertMarkdown } = options;
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
        noteId: noteId || null,
        sourcePath: tempPath,
      });

      if (image?.path) {
        const filename = image.path.split(/[/\\]/).pop() || tempName;
        insertMarkdown(`![${filename}](${filename})`);
      }
    } catch (err) {
      console.error("Failed to save pasted image:", err);
      toast("图片保存失败");
    }
  }

  function handlePaste(e: ClipboardEvent) {
    const files = e.clipboardData?.files;
    if (!files || files.length === 0) return;

    for (const file of Array.from(files)) {
      if (!file.type.startsWith("image/")) continue;
      e.preventDefault();
      saveAndInsertImage(file);
      break;
    }
  }

  onMounted(() => {
    containerRef.value?.addEventListener("paste", handlePaste);
  });

  onUnmounted(() => {
    containerRef.value?.removeEventListener("paste", handlePaste);
  });
}
