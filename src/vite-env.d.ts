/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>;
  export default component;
}

declare module "@milkdown/theme-nord" {
  import type { MilkdownPlugin } from "@milkdown/core";
  export const nord: MilkdownPlugin;
}
