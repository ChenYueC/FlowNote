import { onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function useTauriEvent(event: string, handler: (payload: unknown) => void) {
  let unlisten: UnlistenFn | null = null;
  let disposed = false;

  onMounted(async () => {
    try {
      const fn = await listen(event, (e) => handler(e.payload));
      // If component was unmounted while we were waiting for listen(), clean up immediately
      if (disposed) {
        fn();
      } else {
        unlisten = fn;
      }
    } catch (err) {
      console.error(`Failed to listen to event ${event}:`, err);
    }
  });

  onUnmounted(() => {
    disposed = true;
    if (unlisten) unlisten();
  });
}
