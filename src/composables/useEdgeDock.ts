import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { PhysicalPosition } from "@tauri-apps/api/window";
import { listen, emit } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getMonitorInfo, invalidateMonitorCache, detectEdge, isPositionOnAnyMonitor, clampToMonitor, DRAG_REGION_TIMEOUT } from "./monitorUtils";
import type { Edge } from "./monitorUtils";

// ── Global singleton ──────────────────────────────────────────────
const windowState = ref<WindowState>("visible");
const dockedEdge = ref<Edge>(null);

let unlistenMove: UnlistenFn | null = null;
let unlistenListeners: UnlistenFn[] = [];
let lastPos = { x: 0, y: 0 };
let lastSize = { width: 360, height: 680 };
let animFrame: number | null = null;
let restorePos = { x: 0, y: 0 };
let dockWindowLabel: string | null = null;
let hideTimer: ReturnType<typeof setTimeout> | null = null;
let hoverCheckInterval: ReturnType<typeof setInterval> | null = null;
let initialized = false;
let pointerInContent = false; // pointer is in content area (pointerleave-based, reliable)
let lastDragPointerMove = 0; // last pointermove on drag-region (timeout-based)
let suppressHideUntil = 0; // cooldown after showWindow — prevents hide during state instability
let hideRequestId = 0; // invalidates stale async hide tasks
let unlistenFocus: UnlistenFn | null = null;
let leaveTimer: ReturnType<typeof setTimeout> | null = null;

const ANIMATION_DURATION = 300;
const HIDE_DELAY = 100;

type WindowState = "visible" | "hiding" | "docked" | "restoring";

// ── Composable ────────────────────────────────────────────────────
export function useEdgeDock() {
  const domCleanupFns: (() => void)[] = [];

  function animateWindow(window: any, fromX: number, fromY: number, toX: number, toY: number): Promise<void> {
    if (animFrame) {
      cancelAnimationFrame(animFrame);
      animFrame = null;
    }
    return new Promise((resolve) => {
      const start = performance.now();
      function step(now: number) {
        const elapsed = now - start;
        const progress = Math.min(elapsed / ANIMATION_DURATION, 1);
        const eased = progress < 0.5
          ? 16 * progress * progress * progress * progress * progress
          : 1 - Math.pow(-2 * progress + 2, 5) / 2;
        const currentX = Math.round(fromX + (toX - fromX) * eased);
        const currentY = Math.round(fromY + (toY - fromY) * eased);
        window.setPosition(new PhysicalPosition(currentX, currentY)).catch(() => {});
        if (progress < 1) {
          animFrame = requestAnimationFrame(step);
        } else {
          animFrame = null;
          resolve();
        }
      }
      animFrame = requestAnimationFrame(step);
    });
  }

  // ── hideWindow: visible → hiding → docked ─────────────────────
  async function hideWindow(edge: Edge) {
    if (windowState.value !== "visible") return;

    hideRequestId++;
    const requestId = hideRequestId;

    windowState.value = "hiding";
    dockedEdge.value = edge;

    const { current: monitor, all: allMonitors } = await getMonitorInfo();
    const window = getCurrentWebviewWindow();

    try {
      const pos = await window.outerPosition();
      restorePos = { x: pos.x, y: pos.y };
      const size = await window.outerSize();
      lastSize = { width: size.width, height: size.height };
    } catch {
      restorePos = { ...lastPos };
    }

    // Abort if showWindow() or another hideWindow() ran during the awaits above
    if (requestId !== hideRequestId) return;

    localStorage.setItem("main-window-pos", JSON.stringify({ x: restorePos.x, y: restorePos.y }));
    localStorage.setItem("main-window-docked", JSON.stringify({ edge }));

    let targetX = restorePos.x;
    let targetY = restorePos.y;

    switch (edge) {
      case "left": targetX = monitor.x - lastSize.width; break;
      case "right": targetX = monitor.x + monitor.width; break;
      case "top": targetY = monitor.y - lastSize.height; break;
    }

    // Create dock icon
    if (!dockWindowLabel) {
      const dockLabel = `edgedock-${Date.now()}`;
      const LOGICAL_WIDTH = 38;
      const LOGICAL_HEIGHT = 120;
      const PHYSICAL_HEIGHT = Math.round(LOGICAL_HEIGHT * monitor.scaleFactor);

      let dockX = 0, dockY = 0;

      switch (edge) {
        case "left":
          dockX = monitor.x;
          dockY = restorePos.y + Math.round(lastSize.height / 2) - Math.round(PHYSICAL_HEIGHT / 2);
          break;
        case "right":
          dockX = monitor.x;
          dockY = restorePos.y + Math.round(lastSize.height / 2) - Math.round(PHYSICAL_HEIGHT / 2);
          break;
        case "top":
          dockX = restorePos.x + Math.round(lastSize.width / 2) - Math.round(PHYSICAL_HEIGHT / 2);
          dockY = monitor.y;
          break;
      }

      try {
        dockWindowLabel = dockLabel;
        const dockWin = new WebviewWindow(dockLabel, {
          url: `index.html?window=edgedock&edge=${edge}`,
          title: "FlowNote Dock",
          width: edge === "top" ? LOGICAL_HEIGHT : LOGICAL_WIDTH,
          height: edge === "top" ? LOGICAL_WIDTH : LOGICAL_HEIGHT,
          x: Math.round(dockX / monitor.scaleFactor),
          y: Math.round(dockY / monitor.scaleFactor),
          decorations: false,
          transparent: true,
          shadow: false,
          alwaysOnTop: true,
          skipTaskbar: true,
          resizable: false,
          visible: false,
        });
        // Wait for dock to be fully ready before starting hide animation
        await new Promise<void>((resolve) => {
          dockWin.once('tauri://created', () => {
            setTimeout(async () => {
              try {
                let finalX = dockX;
                let finalY = dockY;

                if (edge === "right") {
                  const outer = await dockWin.outerSize();
                  const inner = await dockWin.innerSize();
                  const totalBorder = outer.width - inner.width;
                  const halfBorder = Math.round(totalBorder / 2);
                  finalX = monitor.x + monitor.width - inner.width - halfBorder;
                }

                if (edge === "left" || edge === "right") {
                  const PHYSICAL_DOCK_H = Math.round(LOGICAL_HEIGHT * monitor.scaleFactor);
                  finalY = Math.max(monitor.y, Math.min(finalY, monitor.y + monitor.height - PHYSICAL_DOCK_H));
                }
                if (edge === "top") {
                  const PHYSICAL_DOCK_W = Math.round(LOGICAL_HEIGHT * monitor.scaleFactor);
                  finalX = Math.max(monitor.x, Math.min(finalX, monitor.x + monitor.width - PHYSICAL_DOCK_W));
                }

                await dockWin.setIgnoreCursorEvents(true);
                await dockWin.setPosition(new PhysicalPosition(finalX, finalY));
                await dockWin.show();
                await dockWin.setPosition(new PhysicalPosition(finalX, finalY));
                await dockWin.setAlwaysOnTop(true);
              } catch {}
              resolve();
            }, 85);
          });
        });
      } catch {
        dockWindowLabel = null;
      }
    }

    await animateWindow(window, restorePos.x, restorePos.y, targetX, targetY);
    if (requestId !== hideRequestId) return;
    await window.hide();
    windowState.value = "docked";
  }

  // ── showWindow: docked/hiding/restoring → visible ─────────────
  async function showWindow() {
    if (windowState.value === "visible") return;
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }

    windowState.value = "restoring";
    hideRequestId++;

    const window = getCurrentWebviewWindow();

    const { current: monitor, all: allMonitors } = await getMonitorInfo();
    let startX = restorePos.x;
    let startY = restorePos.y;
    if (dockedEdge.value === "left") startX = monitor.x - lastSize.width;
    else if (dockedEdge.value === "right") startX = monitor.x + monitor.width;
    else if (dockedEdge.value === "top") startY = monitor.y - lastSize.height;

    // Validate restorePos is still on a valid monitor
    let finalRestorePos = { ...restorePos };
    if (!isPositionOnAnyMonitor(finalRestorePos.x, finalRestorePos.y, allMonitors)) {
      // Restore position is off-screen (monitor disconnected, arrangement changed, etc.)
      // Center the window on the current monitor instead
      finalRestorePos = {
        x: monitor.x + Math.round((monitor.width - lastSize.width) / 2),
        y: monitor.y + Math.round((monitor.height - lastSize.height) / 2),
      };
    } else {
      // Clamp to current monitor to avoid partial off-screen
      finalRestorePos = clampToMonitor(finalRestorePos, lastSize, monitor);
    }

    await window.setPosition(new PhysicalPosition(startX, startY));

    // Fade out dock handle before closing for smooth transition.
    if (dockWindowLabel) {
      const labelToClose = dockWindowLabel;
      dockWindowLabel = null;
      try {
        await emit("edgedock:fade-out");
        await new Promise(r => setTimeout(r, 150));
        const dockWin = await WebviewWindow.getByLabel(labelToClose);
        if (dockWin) await dockWin.close();
      } catch {}
    }

    await window.show();
    await window.setFocus();
    await animateWindow(window, startX, startY, finalRestorePos.x, finalRestorePos.y);
    lastPos = { ...finalRestorePos };

    localStorage.removeItem("main-window-docked");
    windowState.value = "visible";

    // The window appeared under the pointer (dock was at the same position).
    lastDragPointerMove = Date.now();
    suppressHideUntil = Date.now() + 300;
  }

  // ── requestHideCheck: unified hide trigger ─────────────────────
  function requestHideCheck() {
    if (Date.now() < suppressHideUntil) return;
    if (windowState.value !== "visible") return;
    if (pointerInContent) return;
    if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
    if (hideTimer) return;

    const requestId = ++hideRequestId;
    hideTimer = setTimeout(async () => {
      hideTimer = null;
      if (requestId !== hideRequestId) return;
      if (pointerInContent) return;
      if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
      if (Date.now() < suppressHideUntil) return;
      if (windowState.value !== "visible") return;
      const { current: monitor, all: allMonitors } = await getMonitorInfo();
      const edge = detectEdge(lastPos.x, lastPos.y, lastSize.width, lastSize.height, monitor, allMonitors);
      if (edge) {
        await hideWindow(edge);
      }
    }, HIDE_DELAY);
  }

  // ── Lifecycle ─────────────────────────────────────────────────
  onMounted(async () => {
    if (initialized) return;
    initialized = true;

    const window = getCurrentWebviewWindow();

    try {
      const allWindows = await WebviewWindow.getAll();
      for (const win of allWindows) {
        if (win.label.startsWith('edgedock-')) await win.close();
      }
    } catch {}

    try {
      const pos = await window.outerPosition();
      lastPos = { x: pos.x, y: pos.y };
      const size = await window.outerSize();
      lastSize = { width: size.width, height: size.height };
    } catch {}

    // Restore docked state
    const dockedState = localStorage.getItem("main-window-docked");
    if (dockedState) {
      try {
        const { edge } = JSON.parse(dockedState);
        if (edge) {
          setTimeout(async () => {
            await hideWindow(edge);
          }, 500);
        }
      } catch {}
    }

    unlistenMove = await window.onMoved((e) => {
      if (windowState.value === "visible") {
        lastPos = { x: e.payload.x, y: e.payload.y };
      }
      // Invalidate monitor cache when window moves (might be on a different monitor now)
      invalidateMonitorCache();
    });

    unlistenListeners.push(await listen('edgedock:show', () => showWindow()));
    unlistenListeners.push(await listen('edgedock:undock', () => showWindow()));

    // Dual tracking:
    // - pointermove on root: track pointer presence (fires only when pointer is inside)
    // - pointerleave: backup signal (unreliable from drag-region on some platforms)
    // - Staleness check: if no pointermove for 500ms, treat as pointer left
    const root = document.querySelector('[data-window="main"]');
    let lastPointerSeen = 0;
    if (root) {
      const onPointerMove = (e: Event) => {
        lastPointerSeen = Date.now();
        const onDrag = !!(e.target as HTMLElement).closest('.drag-region');
        if (onDrag) {
          lastDragPointerMove = Date.now();
        }
        pointerInContent = !onDrag;
        if (leaveTimer) { clearTimeout(leaveTimer); leaveTimer = null; }
      };
      root.addEventListener('pointermove', onPointerMove);
      domCleanupFns.push(() => root!.removeEventListener('pointermove', onPointerMove));

      const onPointerLeave = () => {
        if (leaveTimer) clearTimeout(leaveTimer);
        leaveTimer = setTimeout(() => {
          leaveTimer = null;
          pointerInContent = false;
          lastDragPointerMove = 0;
          requestHideCheck();
        }, 50);
      };
      root.addEventListener('pointerleave', onPointerLeave);
      domCleanupFns.push(() => root!.removeEventListener('pointerleave', onPointerLeave));
    }

    // Periodic check — paused when window loses focus
    // Periodic check — 500ms interval (no perceptible delay for auto-hide, saves CPU vs 200ms)
    hoverCheckInterval = setInterval(() => {
      if (windowState.value !== "visible") return;
      // Staleness check: no pointermove for 500ms → pointer left the window
      if (lastPointerSeen > 0 && Date.now() - lastPointerSeen > 500) {
        pointerInContent = false;
        lastDragPointerMove = 0;
      }
      if (pointerInContent) return;
      if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
      requestHideCheck();
    }, 500);

    // Focus-lost insurance + pause/resume animations and timers
    unlistenFocus = await window.onFocusChanged(({ payload: isFocused }) => {
      document.documentElement.classList.toggle("window-unfocused", !isFocused);
      if (hoverCheckInterval) { clearInterval(hoverCheckInterval); hoverCheckInterval = null; }
      if (isFocused) {
        hoverCheckInterval = setInterval(() => {
          if (windowState.value !== "visible") return;
          if (pointerInContent) return;
          if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
          requestHideCheck();
        }, 500);
      }

      if (isFocused) return;
      if (windowState.value !== "visible") return;
      if (pointerInContent) return;
      if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
      requestHideCheck();
    });

  });

  onUnmounted(() => {
    if (unlistenMove) unlistenMove();
    if (unlistenFocus) { unlistenFocus(); unlistenFocus = null; }
    if (leaveTimer) { clearTimeout(leaveTimer); leaveTimer = null; }
    if (animFrame) cancelAnimationFrame(animFrame);
    unlistenListeners.forEach((fn) => fn());
    unlistenListeners = [];
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    if (hoverCheckInterval) { clearInterval(hoverCheckInterval); hoverCheckInterval = null; }
    domCleanupFns.forEach((fn) => fn());
    domCleanupFns.length = 0;
    suppressHideUntil = 0;
    hideRequestId++;
    windowState.value = "visible";
    dockedEdge.value = null;
    initialized = false;
  });

  return {
    windowState,
    dockedEdge,
    showWindow,
    requestHideCheck,
  };
}
