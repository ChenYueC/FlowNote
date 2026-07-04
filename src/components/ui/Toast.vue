<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed bottom-24 left-1/2 z-[99999] px-5 py-2.5 rounded-full text-xs text-white/90 animate-toast-shake whitespace-nowrap glass-strong"
      style="transform: translateX(-50%);"
    >
      {{ message }}
    </div>
  </Teleport>
</template>

<script setup lang="ts">
defineProps<{
  message: string;
  visible: boolean;
}>();
</script>

<style scoped>
@keyframes toast-shake {
  0%, 100% { transform: translateX(-50%); }
  15% { transform: translateX(calc(-50% - 6px)); }
  30% { transform: translateX(calc(-50% + 6px)); }
  45% { transform: translateX(calc(-50% - 4px)); }
  60% { transform: translateX(calc(-50% + 4px)); }
  75% { transform: translateX(calc(-50% - 2px)); }
  90% { transform: translateX(calc(-50% + 2px)); }
}

.animate-toast-shake {
  animation: toast-shake 0.5s ease-in-out;
}
</style>
