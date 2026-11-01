import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useChat } from '@ai-sdk/react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { DefaultChatTransport, isToolUIPart } from 'ai';
import { ArrowDownIcon, AlertCircleIcon, RotateCcwIcon } from 'lucide-react';
import { dictionariesQuery } from '@/lib/api/client';
import { agentChatQuery, agentChatsQuery, stopAgentReply } from '@/lib/agent/client';
import { storeModel, type AgentModelId } from '@/lib/agent/models';
import type { AgentChat as AgentChatType, AgentUIMessage } from '@/lib/agent/types';
import { resolveSelectedSlugs, useSearchPrefs } from '@/lib/search-prefs';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { AssistantMessage, ThinkingLine, UserMessage } from './agent-message';
import { ChatHeader } from './chat-header';
import { ChatComposer, type ComposerDraft } from './chat-composer';
import { ModelPicker } from './model-picker';
import { QueuedMessages } from './queued-messages';
import { hasPendingMessage, takePendingMessage } from './pending-message';
import { useMessageQueue } from './use-message-queue';

export interface AgentChatData {
  chat: AgentChatType;
  messages: AgentUIMessage[];
}

/** A tool call still waiting on client-side input: the run may continue by itself. */
function hasPendingToolInput(messages: AgentUIMessage[]): boolean {
  const last = [...messages].reverse().find((message) => message.role === 'assistant');
  return (
    last?.parts.some(
      (part) =>
        isToolUIPart(part) && (part.state === 'input-streaming' || part.state === 'input-available')
    ) === true
  );
}

export function AgentChat({ data }: { data: AgentChatData }) {
  const { chat } = data;
  const queryClient = useQueryClient();
  const [model, setModelState] = useState<AgentModelId>(chat.model);
  const modelRef = useRef(model);
  modelRef.current = model;

  // The reader's preferred dictionaries travel with every message.
  const prefs = useSearchPrefs();
  const dictsQuery = useQuery(dictionariesQuery());
  const selected = dictsQuery.data
    ? resolveSelectedSlugs(prefs, dictsQuery.data)
    : prefs.dicts === 'all'
      ? []
      : prefs.dicts;
  const dictsRef = useRef<string[]>(selected);
  dictsRef.current = selected;

  const setModel = useCallback((next: AgentModelId) => {
    setModelState(next);
    storeModel(next);
  }, []);

  const transport = useMemo(
    () =>
      new DefaultChatTransport<AgentUIMessage>({
        api: '/api/agent/chat',
        // Only the new message (plus the per-request model/dicts): history is loaded server-side.
        prepareSendMessagesRequest: ({ id, messages, body }) => ({
          body: { ...body, id, message: messages.at(-1) }
        })
      }),
    []
  );

  const refreshList = useCallback(
    () => void queryClient.invalidateQueries({ queryKey: agentChatsQuery().queryKey }),
    [queryClient]
  );

  // Decided once at mount (useChat reads `resume` only then). Never with a parked first message:
  // that one is sent below, and the reconnect would race the new reply.
  const [resumeOnMount] = useState(() => chat.streaming && !hasPendingMessage(chat.id));

  const { messages, sendMessage, setMessages, status, stop, error, regenerate } =
    useChat<AgentUIMessage>({
      id: chat.id,
      messages: data.messages,
      transport,
      // A reply still generating when the chat opens (reload, navigation back): reconnect to
      // `GET /api/agent/chat/{id}/stream`, which replays it from the start and follows it live.
      // 204 when it runs on another instance — the background poll below covers that.
      resume: resumeOnMount,
      onFinish: () => {
        refreshList();
        // Title + streaming flag land with the save; pick up the fresh chat state.
        void queryClient.invalidateQueries({ queryKey: agentChatQuery(chat.id).queryKey });
        // The title is saved as the stream closes; pick it up a moment later too.
        setTimeout(refreshList, 1_500);
      }
    });

  useEffect(() => {
    const pending = takePendingMessage(chat.id);
    if (pending) {
      void sendMessage(
        { parts: [{ type: 'text' as const, text: pending.text }] },
        { body: { model: modelRef.current, dicts: dictsRef.current } }
      );
    }
    // Sends the parked first message exactly once, on mount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [chat.id]);

  const busy = status === 'submitted' || status === 'streaming';

  // Drafts typed while a reply streams. Per chat (this view remounts per chat id) and
  // in-memory only — the queue hook clears it on unmount.
  const messaging = useMessageQueue();
  const [editTicket, setEditTicket] = useState<{ message: ComposerDraft; nonce: number } | null>(
    null
  );
  // Set when the user stops the run: the queue stays put for them to decide.
  const stoppedManually = useRef(false);
  // This view sent at least once: background-stream adoption below must not clobber it.
  const sentLocally = useRef(false);

  const sendNow = useCallback(
    (text: string) => {
      sentLocally.current = true;
      stoppedManually.current = false;
      void sendMessage(
        { parts: [{ type: 'text' as const, text }] },
        { body: { model: modelRef.current, dicts: dictsRef.current } }
      );
    },
    [sendMessage]
  );

  const send = (draft: ComposerDraft) => {
    // While a reply streams the composer stays usable: Enter queues instead of interrupting.
    if (busy) messaging.enqueue(draft.text);
    else sendNow(draft.text);
  };

  const handleStop = () => {
    stoppedManually.current = true;
    void stopAgentReply(chat.id);
    stop();
  };

  const sendQueuedNow = (id: string) => {
    const item = messaging.take(id);
    if (!item) return;
    // Interrupt the run for this one; the rest of the queue still auto-sends afterwards.
    if (busy) {
      stoppedManually.current = true;
      void stopAgentReply(chat.id);
      stop();
    }
    sendNow(item.text);
  };

  const editQueued = (id: string) => {
    const item = messaging.take(id);
    if (!item) return;
    setEditTicket((ticket) => ({
      message: { text: item.text },
      nonce: (ticket?.nonce ?? 0) + 1
    }));
  };

  // A run that finished normally (no error, not stopped, no tool continuation pending)
  // picks up the first queued draft. After an error or Stop the queue stays for the user.
  const prevStatus = useRef(status);
  useEffect(() => {
    const wasBusy = prevStatus.current === 'submitted' || prevStatus.current === 'streaming';
    prevStatus.current = status;
    if (!wasBusy || status !== 'ready' || error || stoppedManually.current) return;
    if (hasPendingToolInput(messages)) return;
    const next = messaging.takeFirst();
    if (next) sendNow(next.text);
  }, [status, error, messages, messaging, sendNow]);

  // A reply generated elsewhere (background stream, another tab) lands in the DB while this
  // view is idle: adopt it so refresh / back-navigation shows the finished turn. Never while
  // this view sent or streams itself — the local conversation wins.
  useEffect(() => {
    if (
      !busy &&
      !sentLocally.current &&
      !data.chat.streaming &&
      data.messages.length > messages.length
    ) {
      setMessages(data.messages);
    }
  }, [busy, data.chat.streaming, data.messages, messages.length, setMessages]);

  const lastMessage = messages.at(-1);

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col">
      <ChatHeader title={chat.title || 'New chat'} model={model} messages={messages} />
      {!busy && chat.streaming ? (
        <p
          role="status"
          className="flex items-center justify-center gap-2 border-b border-border/70 px-3 py-1.5 text-xs text-muted-foreground"
        >
          <span className="size-1.5 animate-pulse rounded-full bg-primary" />
          Replying in the background — it will appear here when it lands.
        </p>
      ) : null}

      <div className="relative min-h-0 flex-1">
        {messages.length === 0 && !busy ? (
          <div className="mx-auto flex h-full w-full max-w-2xl items-center justify-center px-4">
            <ThinkingLine />
          </div>
        ) : (
          <MessageList
            messages={messages}
            busy={busy}
            lastMessageId={lastMessage?.id}
            status={status}
            error={error}
            onRetry={() => void regenerate({ body: { model, dicts: dictsRef.current } })}
          />
        )}
      </div>

      <div className="mx-auto w-full max-w-3xl px-4 pb-4">
        <QueuedMessages
          queue={messaging.queue}
          onEdit={editQueued}
          onSendNow={sendQueuedNow}
          onRemove={messaging.remove}
        />
        <ChatComposer
          busy={busy}
          onStop={handleStop}
          onSend={send}
          placeholder="Ask about a word, root, sūtra…"
          toolbar={<ModelPicker value={model} onChange={setModel} disabled={busy} />}
          editTicket={editTicket}
          autoFocus
        />
      </div>
    </div>
  );
}

function MessageList({
  messages,
  busy,
  lastMessageId,
  status,
  error,
  onRetry
}: {
  messages: AgentUIMessage[];
  busy: boolean;
  lastMessageId: string | undefined;
  status: string;
  error: Error | undefined;
  onRetry: () => void;
}) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const stuckRef = useRef(true);
  const [stuck, setStuck] = useState(true);

  const scrollToBottom = useCallback((smooth = false) => {
    const el = viewportRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: smooth ? 'smooth' : 'auto' });
  }, []);

  const onScroll = useCallback(() => {
    const el = viewportRef.current;
    if (!el) return;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 120;
    stuckRef.current = nearBottom;
    setStuck(nearBottom);
  }, []);

  useEffect(() => {
    if (stuckRef.current) scrollToBottom();
  }, [messages, status, scrollToBottom]);

  const lastMessage = messages.at(-1);

  return (
    <div className="absolute inset-0">
      <div
        ref={viewportRef}
        onScroll={onScroll}
        className="h-full overflow-y-auto overscroll-contain"
      >
        <div className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6">
          {messages.map((message) => (
            <div key={message.id}>
              {message.role === 'user' ? (
                <UserMessage message={message} />
              ) : (
                <AssistantMessage
                  message={message}
                  streaming={busy && message.id === lastMessageId}
                />
              )}
            </div>
          ))}
          {status === 'submitted' && lastMessage?.role === 'user' ? <ThinkingLine /> : null}
          {error && !busy ? (
            <div className="flex items-start gap-2 rounded-xl border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
              <AlertCircleIcon className="mt-0.5 size-4 shrink-0" />
              <div className="flex min-w-0 flex-1 flex-col gap-2">
                <p className="break-words">{error.message || 'The agent failed to reply.'}</p>
                <Button size="xs" variant="outline" className="self-start" onClick={onRetry}>
                  <RotateCcwIcon />
                  Retry
                </Button>
              </div>
            </div>
          ) : null}
        </div>
      </div>
      <button
        type="button"
        aria-label="Scroll to bottom"
        onClick={() => scrollToBottom(true)}
        className={cn(
          'absolute right-4 bottom-4 grid size-9 place-items-center rounded-full border bg-background shadow-md transition-all hover:bg-muted',
          stuck ? 'pointer-events-none translate-y-2 opacity-0' : 'translate-y-0 opacity-100'
        )}
        tabIndex={stuck ? -1 : 0}
      >
        <ArrowDownIcon className="size-4" />
      </button>
    </div>
  );
}
