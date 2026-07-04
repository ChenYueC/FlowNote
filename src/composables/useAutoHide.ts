import { ref, onMounted, onUnmounted } from "vue";
import { getCurrentWebviewWindow, WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { PhysicalPosition } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import {
  getMonitorInfo, invalidateMonitorCache, detectEdge,
  isPositionOnAnyMonitor, clampToMonitor, DRAG_REGION_TIMEOUT,
} from "./monitorUtils";

type Edge = "left" | "right" | "top" | "bottom" | null;

const SNAP_THRESHOLD = 15;
const ANIMATION_DURATION = 300;
const RESTORE_COOLDOWN = 600;
const HIDE_DELAY = 100;



export function useAutoHide(dockId?: string, windowSelector?: string) {
  const isHidden = ref(false);
  // const isCollapsed = ref(false); // 已收起状态已移除
  const hiddenEdge = ref<Edge>(null);
  const autoHideEnabled = ref(true);
  const snapEnabled = ref(true);
  const suppressAutoHide = ref(false); // 外部抑制自动隐藏（如右键菜单打开时）

  let unlistenResize: UnlistenFn | null = null;
  let unlistenFocus: UnlistenFn | null = null;
  let unlistenClose: UnlistenFn | null = null;
  let unlistenDockShow: UnlistenFn | null = null;
  let unlistenMove: UnlistenFn | null = null;
  let lastPos = { x: 0, y: 0 };
  let lastSize = { width: 320, height: 240 };
  let isAnimating = false;
  let restorePos = { x: 0, y: 0 };
  let suppressUntil = 0;
  let hideRequestId = 0;
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let hoverCheckInterval: ReturnType<typeof setInterval> | null = null;
  let leaveTimer: ReturnType<typeof setTimeout> | null = null;
  let pointerInContent = false;
  let lastDragPointerMove = 0;
  let lastPointerSeen = 0;
  let taskbarHidden = false;
  let restoringFromDock = false;

  let dockWindowLabel: string | null = null;
  const domCleanupFns: (() => void)[] = [];

  // ── Create dock handle window (starts off-screen, slides in) ───
  async function createDockWindow(edge: Edge) {
    if (!dockId || !edge) return;

    const { current: monitor } = await getMonitorInfo();
    const LOGICAL_WIDTH = 38;
    const LOGICAL_HEIGHT = 120;
    const PHYSICAL_WIDTH = Math.round(LOGICAL_WIDTH * monitor.scaleFactor);
    const PHYSICAL_HEIGHT = Math.round(LOGICAL_HEIGHT * monitor.scaleFactor);

    // Final (on-screen) position
    let finalX = 0, finalY = 0;
    // Start (off-screen) position
    let startX = 0, startY = 0;

    switch (edge) {
      case "left":
        finalX = monitor.x;
        finalY = restorePos.y + Math.round(lastSize.height / 2) - Math.round(PHYSICAL_HEIGHT / 2);
        startX = finalX - PHYSICAL_WIDTH;
        startY = finalY;
        break;
      case "right":
        finalX = monitor.x + monitor.width - PHYSICAL_WIDTH;
        finalY = restorePos.y + Math.round(lastSize.height / 2) - Math.round(PHYSICAL_HEIGHT / 2);
        startX = monitor.x + monitor.width;
        startY = finalY;
        break;
      case "top":
        finalX = restorePos.x + Math.round(lastSize.width / 2) - Math.round(PHYSICAL_HEIGHT / 2);
        finalY = monitor.y;
        startX = finalX;
        startY = finalY - PHYSICAL_WIDTH;
        break;
    }

    // Clamp dock position to stay within monitor bounds
    if (edge === "left" || edge === "right") {
      finalY = Math.max(monitor.y, Math.min(finalY, monitor.y + monitor.height - PHYSICAL_HEIGHT));
      startY = finalY;
    }
    if (edge === "top") {
      finalX = Math.max(monitor.x, Math.min(finalX, monitor.x + monitor.width - PHYSICAL_HEIGHT));
      startX = finalX;
    }

    try {
      const label = `editor-dock-${dockId}-${Date.now()}`;
      dockWindowLabel = label;

      const dockWin = new WebviewWindow(label, {
        url: `index.html?window=editor-dock&edge=${edge}&dock_id=${dockId}`,
        title: "Editor Dock",
        width: edge === "top" ? LOGICAL_HEIGHT : LOGICAL_WIDTH,
        height: edge === "top" ? LOGICAL_WIDTH : LOGICAL_HEIGHT,
        x: Math.round(startX / monitor.scaleFactor),
        y: Math.round(startY / monitor.scaleFactor),
        decorations: false,
        transparent: true,
        shadow: false,
        alwaysOnTop: true,
        skipTaskbar: true,
        resizable: false,
        visible: false,
      });

      // Wait for window to be created and positioned
      await new Promise<void>((resolve) => {
        dockWin.once('tauri://created', async () => {
          try {
            await new Promise(r => setTimeout(r, 85));

            // Adjust for right-edge invisible border
            let adjFinalX = finalX;
            if (edge === "right") {
              const outer = await dockWin.outerSize();
              const inner = await dockWin.innerSize();
              const totalBorder = outer.width - inner.width;
              const halfBorder = Math.round(totalBorder / 2);
              adjFinalX = monitor.x + monitor.width - inner.width - halfBorder;
            }

            await dockWin.setIgnoreCursorEvents(true);
            await dockWin.setPosition(new PhysicalPosition(adjFinalX, finalY));
            await dockWin.show();
            await dockWin.setPosition(new PhysicalPosition(adjFinalX, finalY));
            await dockWin.setAlwaysOnTop(true);
          } catch {}
          resolve();
        });
      });
    } catch {
      dockWindowLabel = null;
    }
  }

  // ── Close dock handle window (with fade-out) ─────────────────────
  async function closeDockWindow() {
    if (!dockWindowLabel) return;
    const label = dockWindowLabel;
    dockWindowLabel = null;
    try {
      // Fade out dock handle before closing for smooth transition
      const { emit } = await import("@tauri-apps/api/event");
      await emit(`editor-dock:fade-out-${dockId}`);
      await new Promise(r => setTimeout(r, 150));
      const dockWin = await WebviewWindow.getByLabel(label);
      if (dockWin) await dockWin.close();
    } catch {}
  }

  // ── Hide: editor slides off-screen + dock slides in from off-screen ─
  async function hideToEdge(edge: Edge) {
    if (!autoHideEnabled.value || isAnimating) return;
    isAnimating = true;

    // Pre-compute target position using cached values (no async calls)
    const { current: monitor } = await getMonitorInfo();
    const window = getCurrentWebviewWindow();

    restorePos = { ...lastPos };

    let targetX = lastPos.x;
    let targetY = lastPos.y;

    switch (edge) {
      case "left":
        targetX = monitor.x - lastSize.width;
        break;
      case "right":
        targetX = monitor.x + monitor.width;
        break;
      case "top":
        targetY = monitor.y - lastSize.height;
        break;
    }

    hiddenEdge.value = edge;

    // Reset taskbar flag before animation — prevents onMoved from calling
    // show_taskbar_icon during the slide-off animation (it was set true by
    // the previous restoreFromEdge).
    taskbarHidden = false;

    // 1. Create dock, then animate editor off-screen
    await createDockWindow(edge);
    await animatePosition(window, lastPos.x, lastPos.y, targetX, targetY);

    // 4. Set hidden state after animation completes
    isHidden.value = true;
    lastPos = { x: targetX, y: targetY };
    isAnimating = false;
    // Fully hide the window so it drops its taskbar icon — the dock handle
    // becomes the sole restore entry point. The previous "已收起" collapsed
    // UI existed only as a taskbar-hover preview; with the taskbar icon gone
    // that preview is no longer needed, so it was removed from EditorWindow.
    // restoreFromEdge() calls window.show() to bring it back.
    try { await window.hide(); } catch {}
  }

  // ── Restore: close dock first, then editor slides in ────────────
  async function restoreFromEdge() {
    if (!isHidden.value || isAnimating) return;
    isAnimating = true;
    restoringFromDock = true;

    const window = getCurrentWebviewWindow();

    const { current: monitor, all: allMonitors } = await getMonitorInfo();

    // Validate restorePos is still on a valid monitor
    let finalRestorePos = { ...restorePos };
    if (!isPositionOnAnyMonitor(finalRestorePos.x, finalRestorePos.y, allMonitors)) {
      // Restore position is off-screen — center on current monitor
      finalRestorePos = {
        x: monitor.x + Math.round((monitor.width - lastSize.width) / 2),
        y: monitor.y + Math.round((monitor.height - lastSize.height) / 2),
      };
    } else {
      // Clamp to current monitor to avoid partial off-screen
      finalRestorePos = clampToMonitor(finalRestorePos, lastSize, {
        x: monitor.x, y: monitor.y, width: monitor.width, height: monitor.height,
      });
    }

    // Close dock first, then show window and animate
    await closeDockWindow();

    // Hide taskbar icon BEFORE showing window
    try { await invoke("hide_taskbar_icon"); taskbarHidden = true; } catch {}

    // Show window at off-screen position
    try { await window.show(); } catch {}

    // Hide taskbar icon AGAIN after show (Tauri may have re-added it)
    try { await invoke("hide_taskbar_icon"); } catch {}

    await window.setAlwaysOnTop(true);
    await window.setFocus();

    isHidden.value = false;
    hiddenEdge.value = null;

    // Editor slides in from off-screen
    await animatePosition(window, lastPos.x, lastPos.y, finalRestorePos.x, finalRestorePos.y);

    try { await window.setAlwaysOnTop(false); } catch {}

    lastPos = { ...finalRestorePos };
    isAnimating = false;
    restoringFromDock = false;
    suppressUntil = Date.now() + RESTORE_COOLDOWN;

    // Refresh cache after restore for next hide
    invalidateMonitorCache();
  }

  // ── Animate window position ────────────────────────────────────
  function animatePosition(
    window: any,
    fromX: number,
    fromY: number,
    toX: number,
    toY: number,
  ): Promise<void> {
    return new Promise((resolve) => {
      const start = performance.now();
      let localAnimFrame: number | null = null;

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
          localAnimFrame = requestAnimationFrame(step);
        } else {
          resolve();
        }
      }

      localAnimFrame = requestAnimationFrame(step);
    });
  }

  // ── Snap to edge (on drag end) ─────────────────────────────────
  async function handleSnap() {
    if (!snapEnabled.value || isHidden.value) return;
    if (Date.now() < suppressUntil) return;

    const { current: monitor } = await getMonitorInfo();
    const window = getCurrentWebviewWindow();
    const relX = lastPos.x - monitor.x;
    const relY = lastPos.y - monitor.y;

    let snapX = lastPos.x;
    let snapY = lastPos.y;

    if (relX < SNAP_THRESHOLD) snapX = monitor.x;
    else if (relX + lastSize.width > monitor.width - SNAP_THRESHOLD) {
      snapX = monitor.x + monitor.width - lastSize.width;
    }

    if (relY < SNAP_THRESHOLD) snapY = monitor.y;

    if (snapX !== lastPos.x || snapY !== lastPos.y) {
      await animatePosition(window, lastPos.x, lastPos.y, snapX, snapY);
      lastPos = { x: snapX, y: snapY };
    }
  }

  function doCheckAutoHide() {
    if (isHidden.value) return;
    if (suppressAutoHide.value) return;
    if (Date.now() < suppressUntil) return;
    if (pointerInContent) return;
    if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
    // Refresh cache if invalidated (e.g. window moved to another monitor), then retry
    getMonitorInfo().then(({ current, all }) => {
      if (isHidden.value || pointerInContent || Date.now() < suppressUntil) return;
      if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
      const edge = detectEdge(lastPos.x, lastPos.y, lastSize.width, lastSize.height, current, all);
      if (edge) hideToEdge(edge);
    });
  }

  function requestHideCheck() {
    if (suppressAutoHide.value) return;
    if (Date.now() < suppressUntil) return;
    if (isHidden.value) return;
    if (pointerInContent) return;
    if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
    if (hideTimer) return;

    const requestId = ++hideRequestId;
    hideTimer = setTimeout(async () => {
      hideTimer = null;
      if (requestId !== hideRequestId) return;
      if (pointerInContent) return;
      if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
      if (Date.now() < suppressUntil) return;
      if (isHidden.value) return;
      // Refresh cache if invalidated, then check edge
      const { current, all } = await getMonitorInfo();
      // Re-validate after async gap — focus may have been regained (rapid
      // taskbar clicks), invalidating this stale hide request.
      if (requestId !== hideRequestId) return;
      if (isHidden.value || pointerInContent || Date.now() < suppressUntil) return;
      const edge = detectEdge(lastPos.x, lastPos.y, lastSize.width, lastSize.height, current, all);
      if (edge) hideToEdge(edge);
    }, HIDE_DELAY);
  }

  // ── Lifecycle ──────────────────────────────────────────────────
  let onCloseCallback: (() => void) | null = null;

  function setOnClose(callback: () => void) {
    onCloseCallback = callback;
  }

  onMounted(async () => {
    const window = getCurrentWebviewWindow();

    try {
      const pos = await window.outerPosition();
      lastPos = { x: pos.x, y: pos.y };
    } catch { /* ignore */ }

    try {
      const size = await window.outerSize();
      lastSize = { width: size.width, height: size.height };
    } catch { /* ignore */ }

    // Pre-warm cache (so first hide has no IPC delay)
    await getMonitorInfo();

    // Track position changes (for snap calculations)
    // Also invalidate monitor cache when window moves to a different monitor
    //
    // NOTE: on Windows, minimizing a window (e.g. clicking its taskbar icon)
    // moves it to an off-screen position (~ -32000, -32000). We must not let
    // that overwrite the real last position — otherwise detectEdge would
    // misread it as a left/top edge dock and wrongly auto-hide the window.
    // (The in-app minimize button avoids this only by luck: the mouse is
    // still inside the window so pointerInContent causes requestHideCheck to
    // bail. A taskbar click has the mouse outside, so the check runs and hits
    // the corrupted position.)
    let moveSeq = 0;
    unlistenMove = await window.onMoved((e) => {
      const seq = ++moveSeq;
      window.isMinimized().then((min) => {
        if (min) return;
        if (seq !== moveSeq) return; // a newer move superseded this one
        lastPos = { x: e.payload.x, y: e.payload.y };
      }).catch(() => {});
      invalidateMonitorCache();

      // Show taskbar icon immediately when window is moved (but not during dock restore animation)
      if (taskbarHidden && !restoringFromDock) {
        invoke("show_taskbar_icon").then(() => { taskbarHidden = false; }).catch(() => {});
      }
    });

    unlistenResize = await window.onResized((e) => {
      window.isMinimized().then((min) => {
        if (min) return;
        lastSize = {
          width: e.payload.width,
          height: e.payload.height,
        };
      }).catch(() => {});
    });

    // Pointer tracking — detect mouse leaving the window
    if (windowSelector) {
      const root = document.querySelector(windowSelector);
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
        domCleanupFns.push(() => root.removeEventListener('pointermove', onPointerMove));

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
        domCleanupFns.push(() => root.removeEventListener('pointerleave', onPointerLeave));
      }
    }

    // Periodic staleness check (500ms — no perceptible delay for auto-hide)
    hoverCheckInterval = setInterval(() => {
      if (isHidden.value) return;
      if (lastPointerSeen > 0 && Date.now() - lastPointerSeen > 500) {
        pointerInContent = false;
        lastDragPointerMove = 0;
      }
      if (pointerInContent) return;
      if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
      requestHideCheck();
    }, 500);

    // Auto-hide on focus loss (backup trigger)
    unlistenFocus = await window.onFocusChanged(({ payload: isFocused }) => {
      if (!isFocused) {
        if (hoverCheckInterval) { clearInterval(hoverCheckInterval); hoverCheckInterval = null; }
        // Restart periodic check when focused again
      } else {
        // Window regained focus — cancel any pending auto-hide that was
        // scheduled during the brief focus loss (e.g. rapid taskbar clicks).
        // Without this, a quick lose→regain focus cycle still fires the
        // 100ms hide timer and wrongly docks the window.
        if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
        if (leaveTimer) { clearTimeout(leaveTimer); leaveTimer = null; }
        hideRequestId++;
        if (!hoverCheckInterval) {
          hoverCheckInterval = setInterval(() => {
            if (isHidden.value) return;
            if (pointerInContent) return;
            if (Date.now() - lastDragPointerMove < DRAG_REGION_TIMEOUT) return;
            requestHideCheck();
          }, 500);
        }
      }
      if (!isFocused) {
        requestHideCheck();
      }
    });

    // Listen for dock show event (restore from hidden state)
    if (dockId) {
      unlistenDockShow = await listen(`editor-dock:show-${dockId}`, () => {
        if (isHidden.value) {
          restoreFromEdge();
        }
      });
    }

    // Clean up dock when window is closed (e.g. from taskbar)
    unlistenClose = await window.onCloseRequested(async () => {
      // Save immediately
      if (onCloseCallback) {
        await onCloseCallback();
      }
      // Clean up image-preview localStorage entries
      for (let i = localStorage.length - 1; i >= 0; i--) {
        const key = localStorage.key(i);
        if (key?.startsWith("image-preview-")) {
          localStorage.removeItem(key);
        }
      }
      // Destroy image preview windows
      const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
      const allWindows = await WebviewWindow.getAll();
      for (const w of allWindows) {
        if (w.label.startsWith("image-preview-")) {
          w.destroy();
        }
      }
      // Remove from open-editors tracking
      let openEditors: string[] = [];
      try { openEditors = JSON.parse(localStorage.getItem("open-editors") || "[]"); } catch { openEditors = []; }
      const updatedEditors = openEditors.filter((l: string) => l !== window.label);
      localStorage.setItem("open-editors", JSON.stringify(updatedEditors));
      // Emit event before destroying window
      const { emit } = await import("@tauri-apps/api/event");
      await emit("editor:closed");
      // Close dock window
      await closeDockWindow();
      // Destroy window last
      await window.destroy();
    });
  });

  onUnmounted(() => {
    if (unlistenMove) unlistenMove();
    if (unlistenResize) unlistenResize();
    if (unlistenFocus) unlistenFocus();
    if (unlistenClose) unlistenClose();
    if (unlistenDockShow) unlistenDockShow();
    if (leaveTimer) { clearTimeout(leaveTimer); leaveTimer = null; }
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    if (hoverCheckInterval) { clearInterval(hoverCheckInterval); hoverCheckInterval = null; }
    domCleanupFns.forEach((fn) => fn());
    domCleanupFns.length = 0;
    closeDockWindow();
  });

  async function refreshPosition() {
    const window = getCurrentWebviewWindow();
    try {
      const pos = await window.outerPosition();
      lastPos = { x: pos.x, y: pos.y };
    } catch {}
    try {
      const size = await window.outerSize();
      lastSize = { width: size.width, height: size.height };
    } catch {}
  }

  return {
    isHidden,
    // isCollapsed, // 已收起状态已移除
    hiddenEdge,
    autoHideEnabled,
    snapEnabled,
    suppressAutoHide,
    handleSnap,
    onWindowMoved: handleSnap,
    refreshPosition,
    checkAutoHide: doCheckAutoHide,
    setOnClose,
  };
}
