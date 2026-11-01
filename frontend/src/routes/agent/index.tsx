import { createFileRoute } from '@tanstack/react-router';
import { NewChatPanel } from '@/components/agent/new-chat-panel';

export const Route = createFileRoute('/agent/')({
  component: NewChatPanel
});
