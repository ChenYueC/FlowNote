import { ref } from "vue";

const message = ref("");
const visible = ref(false);
let timer: ReturnType<typeof setTimeout> | null = null;

export function useToast() {
  function toast(msg: string, duration = 2500) {
    if (timer) clearTimeout(timer);
    message.value = msg;
    visible.value = true;
    timer = setTimeout(() => {
      visible.value = false;
    }, duration);
  }

  return { message, visible, toast };
}
