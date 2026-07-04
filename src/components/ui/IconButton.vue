<template>
  <button
    class="no-drag flex items-center justify-center w-8 h-8 rounded-full transition-all duration-200 hover:bg-white/10 active:scale-95"
    :class="active ? 'text-white bg-white/10' : 'text-white/70 hover:text-white'"
    @click="$emit('click')"
  >
    <slot />
  </button>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ active?: boolean }>(), { active: false });
defineEmits<{ click: [] }>();
</script>
