import { queryOptions } from '@tanstack/react-query';
import { ApiError } from '@/lib/api/client';
import type { AgentChat, AgentChatSummary, AgentContext, AgentUIMessage } from './types';

async function fetchAgentJson<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  if (!res.ok) {
    let message = `Request failed (${res.status})`;
    try {
      const body = (await res.json()) as { error?: string };
      if (body?.error) message = body.error;
    } catch {
      // Fall through to the generic message.
    }
    throw new ApiError(res.status, message);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export function agentChatsQuery() {
  return queryOptions({
    queryKey: ['agent-chats'],
    queryFn: () => fetchAgentJson<AgentChatSummary[]>('/api/agent/chats'),
    staleTime: 1000 * 30
  });
}

export function agentChatQuery(id: string) {
  return queryOptions({
    queryKey: ['agent-chat', id],
    queryFn: () =>
      fetchAgentJson<{ chat: AgentChat; messages: AgentUIMessage[] }>(
        `/api/agent/chats/${encodeURIComponent(id)}`
      ),
    staleTime: 1000 * 10,
    retry: false
  });
}

/** Exactly what the model receives on every turn, for the preferred dictionaries. */
export function agentContextQuery(dicts: string[]) {
  return queryOptions({
    queryKey: ['agent-context', dicts],
    queryFn: () => {
      const sp = new URLSearchParams({ dicts: dicts.join(',') });
      return fetchAgentJson<AgentContext>(`/api/agent/context?${sp.toString()}`);
    },
    staleTime: 1000 * 60 * 5
  });
}

export async function renameAgentChat(id: string, title: string): Promise<AgentChatSummary> {
  return fetchAgentJson<AgentChatSummary>(`/api/agent/chats/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title })
  });
}

export async function deleteAgentChat(id: string): Promise<void> {
  await fetchAgentJson<void>(`/api/agent/chats/${encodeURIComponent(id)}`, { method: 'DELETE' });
}

/** Ask the server to abort the reply being generated (fires on any instance). */
export async function stopAgentReply(id: string): Promise<void> {
  try {
    await fetchAgentJson<void>(`/api/agent/chats/${encodeURIComponent(id)}/stop`, {
      method: 'POST'
    });
  } catch {
    // Best effort: the local stop() already halted this tab's stream.
  }
}
