// src/composables/useCommandRegistry.ts
// Singleton command handler registry — stable references, never rebuilt.
// Layer 1: handlers register once at init
// Layer 2: shortcutMap binds keys → commandIds (from settings store)
// Layer 3: keydown → normalize → shortcutMap → executeCommand

const handlers = new Map<string, () => void>();

export function registerCommand(id: string, handler: () => void) {
  handlers.set(id, handler);
}

export function executeCommand(id: string): boolean {
  const handler = handlers.get(id);
  if (handler) {
    handler();
    return true;
  }
  return false;
}

export function getCommandIds(): string[] {
  return Array.from(handlers.keys());
}