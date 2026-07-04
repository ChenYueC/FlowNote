<template>
  <div class="px-2 space-y-3">
    <!-- Group by date -->
    <div v-for="group in groupedItems" :key="group.date" class="animate-fade-in-up">
      <!-- Date header -->
      <div class="flex items-center gap-3 px-2 py-2 sticky top-0 z-10">
        <div class="w-2 h-2 rounded-full bg-blue-400/60 flex-shrink-0" />
        <span class="text-xs font-medium text-white/50">{{ group.label }}</span>
        <div class="flex-1 h-px bg-white/5" />
        <span class="text-[10px] text-white/25">{{ group.items.length }} 条</span>
      </div>

      <!-- Timeline items -->
      <div class="ml-4 pl-4 space-y-1.5">
        <div
          v-for="item in group.items"
          :key="item.id"
          class="group relative px-3 py-2 rounded-xl"
          :class="getItemClass(item)"
          @click="onItemClick(item)"
        >
          <!-- Timeline dot -->
          <div
            class="absolute -left-[22px] top-1/2 -translate-y-1/2 w-2 h-2 rounded-full transition-colors"
            :class="item.item_type === 'reminder' ? 'bg-amber-400/50 group-hover:bg-amber-400/80' : 'bg-white/15 group-hover:bg-blue-400/60'"
          />

          <div class="flex items-center gap-1.5">
            <!-- Reminder icon -->
            <svg v-if="item.item_type === 'reminder'" class="w-3 h-3 text-amber-400/70 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <h4
              class="text-sm truncate"
              :class="item.archived ? 'text-white/30 line-through' : 'text-white/80'"
            >{{ getItemTitle(item) }}</h4>
          </div>
          <div class="mt-1 text-[10px] text-white/25">
            <span>{{ formatTime(item.created_at) }}</span>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="items.length === 0"
      class="flex flex-col items-center justify-center py-16 text-white/25 text-sm"
    >
      <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" class="mb-4 opacity-30">
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
      时间轴为空
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useToast } from "@/composables/useToast";
import type { Note, TimelineItem } from "@/types";

const props = defineProps<{
  items: TimelineItem[];
}>();

const emit = defineEmits<{
  open: [note: Note];
}>();

const { toast } = useToast();

const clickTimers: Record<string, ReturnType<typeof setTimeout> | null> = {};

function onItemClick(item: TimelineItem) {
  // Reminders don't support double-click
  if (item.item_type === "reminder") return;

  if (item.archived) {
    toast("笔记已被删除！");
    return;
  }

  if (clickTimers[item.id]) {
    clearTimeout(clickTimers[item.id]!);
    clickTimers[item.id] = null;
    emit("open", { id: item.id, title: item.title } as Note);
  } else {
    clickTimers[item.id] = setTimeout(() => {
      clickTimers[item.id] = null;
    }, 400);
  }
}

interface Group {
  date: string;
  label: string;
  items: TimelineItem[];
}

const groupedItems = computed<Group[]>(() => {
  const groups = new Map<string, TimelineItem[]>();

  for (const item of props.items) {
    const d = new Date(item.created_at * 1000);
    const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(item);
  }

  return Array.from(groups.entries()).map(([date, items]) => ({
    date,
    label: formatDateLabel(date),
    items,
  }));
});

function getItemClass(item: TimelineItem): string {
  if (item.item_type === "reminder") {
    return item.archived ? "cursor-default opacity-50" : "cursor-default hover:bg-white/8";
  }
  return item.archived ? "cursor-not-allowed opacity-50" : "cursor-pointer hover:bg-white/8";
}

function getItemTitle(item: TimelineItem): string {
  const title = item.title || "无标题";
  if (item.item_type === "reminder") {
    return item.archived ? `已完成 · ${title}` : `提醒 · ${title}`;
  }
  if (item.item_action === "add") return `创建了 · ${title}`;
  return title;
}

function formatDateLabel(dateStr: string): string {
  const d = new Date(dateStr);
  const now = new Date();
  const diff = Math.floor((now.getTime() - d.getTime()) / 86400000);

  if (diff === 0) return "今天";
  if (diff === 1) return "昨天";
  if (diff < 7) return `${diff} 天前`;

  const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return `${d.getMonth() + 1}月${d.getDate()}日 ${weekdays[d.getDay()]}`;
}

function formatTime(ts: number): string {
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}
</script>
