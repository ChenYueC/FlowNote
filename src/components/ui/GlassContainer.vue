<template>
  <div
    class="glass rounded-3xl transition-all duration-200"
    :class="paddingClass"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    padding?: "sm" | "md" | "lg";
  }>(),
  { padding: "md" },
);

const paddingClass = computed(() => {
  switch (props.padding) {
    case "sm":
      return "p-3";
    case "lg":
      return "p-6";
    default:
      return "p-4";
  }
});
</script>
