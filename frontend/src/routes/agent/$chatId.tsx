import { createFileRoute } from '@tanstack/react-router';
import { ChatPage } from '@/components/agent/chat-page';

export const Route = createFileRoute('/agent/$chatId')({
  component: RouteComponent
});

function RouteComponent() {
  const { chatId } = Route.useParams();
  return <ChatPage chatId={chatId} />;
}
