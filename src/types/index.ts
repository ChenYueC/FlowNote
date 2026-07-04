export interface Note {
  id: string;
  title: string;
  content: string;
  workspace?: string;
  created_at: number;
  updated_at: number;
  archived: number;
  favorite: number;
  type: string;
  color_ranges: string;
}

export interface ReminderCounts {
  all: number;
  today: number;
  overdue: number;
  cancelled: number;
  completed: number;
}

export interface ReminderListResponse {
  reminders: Reminder[];
  counts: ReminderCounts;
}

export interface TimelineItem {
  id: string;
  title: string;
  content: string;
  created_at: number;
  item_type: "note" | "reminder";
  item_action: string;
  archived: number;
}

export interface Image {
  id: string;
  note_id: string;
  path: string;
  width: number | null;
  height: number | null;
  hash: string | null;
  created_at: number;
}

export interface FloatingWindow {
  id: string;
  note_id: string;
  type: string;
  title: string | null;
  x: number;
  y: number;
  width: number;
  height: number;
  minimized: number;
  pinned: number;
  auto_hide: number;
  locked: number;
  opacity: number;
  monitor_id: string | null;
  created_at: number;
  updated_at: number;
}

export interface CreateNoteInput {
  title: string;
  content: string;
}

export interface UpdateNoteInput {
  id: string;
  title?: string;
  content?: string;
  color_ranges?: string;
}

export interface CreateFloatingWindowInput {
  note_id: string;
  x: number;
  y: number;
  width?: number;
  height?: number;
}

export interface UpdateWindowStateInput {
  id: string;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  pinned?: number;
  auto_hide?: number;
  locked?: number;
  opacity?: number;
  minimized?: number;
}

export interface Reminder {
  id: string;
  note_id: string | null;
  title: string;
  content: string;
  remind_at: number;
  completed: number;
  snoozed: number;
  deleted: number;
  cancelled: number;
  created_at: number;
  updated_at: number;
}

export interface CreateReminderInput {
  title: string;
  note_id?: string;
  content?: string;
  remind_at: number;
}

export interface UpdateReminderInput {
  id: string;
  title?: string;
  content?: string;
  remind_at?: number;
  completed?: number;
  snoozed?: number;
}

export type WindowType = "main" | "editor" | "floating" | "screenshot" | "reminder" | "radial" | "edgedock" | "editor-dock" | "image-preview" | "tray-menu" | "settings" | "area-select" | "update-dialog";
