import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { agentChatQuery } from '@/lib/agent/client';
import { loadStoredModel } from '@/lib/agent/models';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { AgentChat, type AgentChatData } from './agent-chat';
import { hasPendingMessage } from './pending-message';

/** A brand-new chat has no row yet (404 until the first reply saves). */
function freshChatData(chatId: string): AgentChatData {
  const now = new Date().toISOString();
  return {
    chat: {
      id: chatId,
      title: '',
      model: loadStoredModel(),
      streaming: false,
      created_at: now,
      updated_at: now,
      dicts: []
    },
    messages: []
  };
}

export function ChatPage({ chatId }: { chatId: string }) {
  // Decided once at mount: with a pending first message this is a fresh chat, so the
  // view never flips to "not found" while the first reply streams. The detail query
  // stays enabled throughout: once the reply saves, the real chat flows into the same
  // AgentChat instance (no remount, queue and stream state intact).
  const [freshData] = useState<AgentChatData | null>(() =>
    hasPendingMessage(chatId) ? freshChatData(chatId) : null
  );
  const chatQuery = useQuery({
    ...agentChatQuery(chatId),
    // While a reply streams in the background, poll so it appears when it lands.
    refetchInterval: (query) => (query.state.data?.chat.streaming ? 2500 : false),
    refetchOnWindowFocus: false
  });

  const data = chatQuery.data ?? freshData;
  if (!data) {
    if (chatQuery.isError) {
      return (
        <div className="flex h-full flex-col items-center justify-center gap-3 px-4 text-center">
          <p className="text-sm text-muted-foreground">Chat not found.</p>
          <Link to="/agent">
            <Button variant="outline" size="sm">
              Start a new chat
            </Button>
          </Link>
        </div>
      );
    }
    return (
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-4 px-4 py-6">
        <Skeleton className="h-6 w-56" />
        <Skeleton className="ml-auto h-10 w-2/3 rounded-2xl" />
        <Skeleton className="h-24 w-full" />
      </div>
    );
  }

  return <AgentChat key={chatId} data={data} />;
}
