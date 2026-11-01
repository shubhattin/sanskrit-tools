import { ArrowUpIcon, PencilIcon, XIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import type { QueuedAgentMessage } from './use-message-queue';

function QueueButton({
  label,
  onClick,
  children
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="icon-xs"
      aria-label={label}
      title={label}
      onClick={onClick}
      className="shrink-0 text-muted-foreground"
    >
      {children}
    </Button>
  );
}

/** Drafts queued while a reply streams: one truncated line each, above the composer. */
export function QueuedMessages({
  queue,
  onEdit,
  onSendNow,
  onRemove
}: {
  queue: QueuedAgentMessage[];
  onEdit: (id: string) => void;
  onSendNow: (id: string) => void;
  onRemove: (id: string) => void;
}) {
  if (queue.length === 0) return null;
  return (
    <ul aria-label="Queued messages" className="mb-2 flex flex-col gap-1">
      {queue.map((item, index) => (
        <li
          key={item.id}
          className="flex items-center gap-1 rounded-xl border border-border/70 bg-muted/40 py-1 pr-1 pl-3"
        >
          <span
            className="min-w-0 flex-1 truncate font-devanagari text-xs text-muted-foreground"
            title={item.text}
          >
            <span className="mr-1.5 font-sans font-semibold text-foreground tabular-nums">
              {index + 1}
            </span>
            {item.text}
          </span>
          <QueueButton label="Edit queued message" onClick={() => onEdit(item.id)}>
            <PencilIcon />
          </QueueButton>
          <QueueButton label="Send now" onClick={() => onSendNow(item.id)}>
            <ArrowUpIcon />
          </QueueButton>
          <QueueButton label="Remove queued message" onClick={() => onRemove(item.id)}>
            <XIcon />
          </QueueButton>
        </li>
      ))}
    </ul>
  );
}
