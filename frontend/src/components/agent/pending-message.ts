/**
 * The first message of a chat started from the New chat screen. The chat row is
 * created server-side on the first `/api/agent/chat` send, so the message is
 * parked here, the app navigates to the chat's URL immediately, and the chat
 * page picks it up and sends it — the reply streams on the chat's own URL.
 * Kept in sessionStorage (module state as fallback) so a reload between
 * navigation and mount doesn't orphan the chat.
 */
export interface PendingAgentMessage {
  text: string;
}

const pending = new Map<string, PendingAgentMessage>();

const storageKey = (chatId: string) => `sans-dict-agent-pending:${chatId}`;

function readStored(chatId: string): PendingAgentMessage | undefined {
  try {
    const stored = sessionStorage.getItem(storageKey(chatId));
    if (stored === null) return undefined;
    const parsed: unknown = JSON.parse(stored);
    // SAFETY: only `setPendingMessage` writes this key, always a PendingAgentMessage.
    return parsed as PendingAgentMessage;
  } catch {
    return undefined;
  }
}

export function setPendingMessage(chatId: string, message: PendingAgentMessage): void {
  pending.set(chatId, message);
  try {
    sessionStorage.setItem(storageKey(chatId), JSON.stringify(message));
  } catch {
    // Private mode etc: module state still covers the same-tab flow.
  }
}

export function takePendingMessage(chatId: string): PendingAgentMessage | undefined {
  const message = pending.get(chatId) ?? readStored(chatId);
  pending.delete(chatId);
  try {
    sessionStorage.removeItem(storageKey(chatId));
  } catch {
    // ignore
  }
  return message;
}

/** Peek without consuming: the AgentChat mount takes (and sends) the pending message. */
export function hasPendingMessage(chatId: string): boolean {
  if (pending.has(chatId)) return true;
  try {
    return sessionStorage.getItem(storageKey(chatId)) !== null;
  } catch {
    return false;
  }
}
