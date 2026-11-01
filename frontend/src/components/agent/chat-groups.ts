const GROUPS = ['Today', 'Yesterday', 'Previous 7 days', 'Previous 30 days', 'Older'] as const;
export type ChatGroupLabel = (typeof GROUPS)[number];

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

function groupLabel(date: Date, now: Date): ChatGroupLabel {
  const days = Math.round(
    (startOfDay(now).getTime() - startOfDay(date).getTime()) / (1000 * 60 * 60 * 24)
  );
  if (days <= 0) return 'Today';
  if (days === 1) return 'Yesterday';
  if (days < 7) return 'Previous 7 days';
  if (days < 30) return 'Previous 30 days';
  return 'Older';
}

/** Sidebar sections by last activity, newest first (input is already sorted). */
export function groupChatsByDate<T extends { updated_at: string }>(
  chats: T[],
  now = new Date()
): { label: ChatGroupLabel; chats: T[] }[] {
  const grouped = new Map<ChatGroupLabel, T[]>();
  for (const chat of chats) {
    const label = groupLabel(new Date(chat.updated_at), now);
    grouped.set(label, [...(grouped.get(label) ?? []), chat]);
  }
  return GROUPS.filter((label) => grouped.has(label)).map((label) => ({
    label,
    chats: grouped.get(label)!
  }));
}
