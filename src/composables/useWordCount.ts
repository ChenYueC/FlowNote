import { type Ref, type ComputedRef, computed } from "vue";

export function useWordCount(content: Ref<string>): {
  displayWordCount: ComputedRef<string>;
} {
  const displayWordCount = computed(() => {
    // Strip image markdown (including base64 data URLs), then strip remaining markdown syntax
    const stripped = content.value
      .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
      .replace(/[#*`>\[\]()!_~|=\-+]/g, "");
    return `${stripped.length} 字`;
  });

  return { displayWordCount };
}
