import { useCallback, useEffect, useRef, useState } from 'react';

export interface QueuedAgentMessage {
  id: string;
  text: string;
}

let queuedMessageCounter = 0;

/** Client-only id (the queue is in-memory and never sent to the server). */
const nextQueuedId = () => `queued-${Date.now().toString(36)}-${(queuedMessageCounter += 1)}`;

const withId = (text: string): QueuedAgentMessage => ({ text, id: nextQueuedId() });

/**
 * In-memory per-chat queue of drafts typed while a reply streams. AgentChat mounts one of
 * these per chat (keyed by chat id), so switching chats never leaks queued drafts across chats.
 */
export function useMessageQueue() {
  const [queue, setQueue] = useState<QueuedAgentMessage[]>([]);
  // Synchronous mirror so take* can return the item in the same tick as removing it.
  const mirror = useRef<QueuedAgentMessage[]>([]);

  const update = useCallback((next: QueuedAgentMessage[]) => {
    mirror.current = next;
    setQueue(next);
  }, []);

  const enqueue = useCallback(
    (text: string) => update([...mirror.current, withId(text)]),
    [update]
  );

  const remove = useCallback(
    (id: string) => update(mirror.current.filter((item) => item.id !== id)),
    [update]
  );

  const take = useCallback(
    (id: string) => {
      const item = mirror.current.find((entry) => entry.id === id);
      if (item) update(mirror.current.filter((entry) => entry.id !== id));
      return item;
    },
    [update]
  );

  const takeFirst = useCallback(() => {
    const first = mirror.current[0];
    if (first) update(mirror.current.slice(1));
    return first;
  }, [update]);

  // Belt and braces: the instance is already per-chat, drop any draft on unmount.
  useEffect(
    () => () => {
      mirror.current = [];
    },
    []
  );

  return { queue, enqueue, remove, take, takeFirst };
}
