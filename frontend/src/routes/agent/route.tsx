import { createFileRoute } from '@tanstack/react-router';
import { AgentLayout } from '@/components/agent/agent-layout';

export const Route = createFileRoute('/agent')({
  component: AgentLayout
});
