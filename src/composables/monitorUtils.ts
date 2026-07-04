// src/composables/monitorUtils.ts
import { currentMonitor, availableMonitors } from "@tauri-apps/api/window";

// ── Shared Constants ──────────────────────────────────────────────
export const EDGE_THRESHOLD = 30;
export const TOP_THRESHOLD = 50;
export const MONITOR_GAP_TOLERANCE = 8;
export const DRAG_REGION_TIMEOUT = 2000;

// ── Types ─────────────────────────────────────────────────────────
export type Edge = "left" | "right" | "top" | null;

export interface MonitorInfo {
  x: number;
  y: number;
  width: number;
  height: number;
  scaleFactor: number;
}

export interface MonitorBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

// ── Multi-monitor helpers ─────────────────────────────────────────

/**
 * Check if a monitor edge is adjacent to another monitor.
 */
export function isEdgeSharedWithMonitor(
  monitor: MonitorBounds,
  edge: "left" | "right" | "top",
  allMonitors: MonitorBounds[]
): boolean {
  for (const other of allMonitors) {
    if (
      other.x === monitor.x &&
      other.y === monitor.y &&
      other.width === monitor.width &&
      other.height === monitor.height
    ) continue;
    switch (edge) {
      case "left":
        if (
          Math.abs((other.x + other.width) - monitor.x) <= MONITOR_GAP_TOLERANCE &&
          other.y + other.height > monitor.y &&
          other.y < monitor.y + monitor.height
        ) return true;
        break;
      case "right":
        if (
          Math.abs(other.x - (monitor.x + monitor.width)) <= MONITOR_GAP_TOLERANCE &&
          other.y + other.height > monitor.y &&
          other.y < monitor.y + monitor.height
        ) return true;
        break;
      case "top":
        if (
          Math.abs((other.y + other.height) - monitor.y) <= MONITOR_GAP_TOLERANCE &&
          other.x + other.width > monitor.x &&
          other.x < monitor.x + monitor.width
        ) return true;
        break;
    }
  }
  return false;
}

/**
 * Check if a position is within any monitor's bounds.
 */
export function isPositionOnAnyMonitor(
  x: number, y: number, monitors: MonitorBounds[]
): boolean {
  for (const m of monitors) {
    if (x >= m.x && x < m.x + m.width && y >= m.y && y < m.y + m.height) {
      return true;
    }
  }
  return false;
}

/**
 * Clamp a window position so it stays within the given monitor's bounds.
 */
export function clampToMonitor(
  pos: { x: number; y: number },
  size: { width: number; height: number },
  monitor: MonitorBounds
): { x: number; y: number } {
  return {
    x: Math.max(monitor.x, Math.min(pos.x, monitor.x + monitor.width - size.width)),
    y: Math.max(monitor.y, Math.min(pos.y, monitor.y + monitor.height - size.height)),
  };
}

/**
 * Detect which edge the window is docked to based on its position relative
 * to the current monitor. Returns null if not near any edge.
 *
 * Guard: positions far off the monitor (e.g. a minimized window reported at
 * ~-32000,-32000 on Windows) must NOT be misread as a left/top dock. Only a
 * window whose top-left corner actually sits on this monitor can be docked.
 */
export function detectEdge(
  x: number, y: number, width: number, _height: number,
  monitor: MonitorBounds,
  allMonitors: MonitorBounds[]
): Edge {
  // Reject off-screen / minimized positions defensively
  if (x < monitor.x - EDGE_THRESHOLD || y < monitor.y - TOP_THRESHOLD) return null;

  const relX = x - monitor.x;
  const relY = y - monitor.y;
  // Check top first (most common edge)
  if (relY <= TOP_THRESHOLD) {
    if (!isEdgeSharedWithMonitor(monitor, "top", allMonitors)) return "top";
  }
  if (relX <= EDGE_THRESHOLD) {
    if (!isEdgeSharedWithMonitor(monitor, "left", allMonitors)) return "left";
  }
  if (relX + width >= monitor.width - EDGE_THRESHOLD) {
    if (!isEdgeSharedWithMonitor(monitor, "right", allMonitors)) return "right";
  }
  return null;
}

// ── Monitor cache management ─────────────────────────────────────

let _monitorCache: MonitorInfo | null = null;
let _allMonitorsCache: MonitorBounds[] | null = null;

export async function getMonitorInfo(): Promise<{ current: MonitorInfo; all: MonitorBounds[] }> {
  if (_monitorCache && _allMonitorsCache) {
    return { current: _monitorCache, all: _allMonitorsCache };
  }
  try {
    const [monitor, monitors] = await Promise.all([currentMonitor(), availableMonitors()]);
    if (monitor) {
      _monitorCache = {
        x: monitor.position.x,
        y: monitor.position.y,
        width: monitor.size.width,
        height: monitor.size.height,
        scaleFactor: monitor.scaleFactor,
      };
    }
    _allMonitorsCache = monitors.map(m => ({
      x: m.position.x,
      y: m.position.y,
      width: m.size.width,
      height: m.size.height,
    }));
  } catch {}

  if (!_monitorCache) {
    _monitorCache = { x: 0, y: 0, width: 1920, height: 1080, scaleFactor: 1 };
  }
  if (!_allMonitorsCache) {
    _allMonitorsCache = [{ x: _monitorCache.x, y: _monitorCache.y, width: _monitorCache.width, height: _monitorCache.height }];
  }
  return { current: _monitorCache, all: _allMonitorsCache };
}

export function invalidateMonitorCache() {
  _monitorCache = null;
  _allMonitorsCache = null;
}
