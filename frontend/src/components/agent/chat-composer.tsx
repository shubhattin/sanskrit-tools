import { useEffect, useRef, useState, type ReactNode } from 'react';
import { ArrowUpIcon, SquareIcon } from 'lucide-react';
import { useLipiInput } from '@/components/lipi/typing';
import { Kbd } from '@/components/ui/kbd';
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupTextarea
} from '@/components/ui/input-group';
import { cn } from '@/lib/utils';

export interface ComposerDraft {
  text: string;
}

/**
 * Message field: Enter sends, Shift+Enter adds a line, Stop while the agent replies.
 * Latin keystrokes become Devanagari while Lipi typing is on (default off; Alt+X flips it).
 */
export function ChatComposer({
  onSend,
  onStop,
  busy,
  placeholder = 'Ask about a word, root, sūtra…',
  toolbar,
  autoFocus,
  editTicket = null
}: {
  onSend: (message: ComposerDraft) => void;
  onStop?: () => void;
  busy: boolean;
  placeholder?: string;
  /** Left side of the bottom bar (e.g. the model picker). */
  toolbar?: ReactNode;
  autoFocus?: boolean;
  /** Editing a queued draft puts its text back here (nonce bumps per edit). */
  editTicket?: { message: ComposerDraft; nonce: number } | null;
}) {
  const [text, setText] = useState('');
  const formRef = useRef<HTMLFormElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Lipi Lekhika transliteration typing, default off per project convention.
  const { props: lipiProps, enabled: lipiOn } = useLipiInput(text, setText, false);

  // A queued draft under edit: restore its text and focus for tweaks.
  useEffect(() => {
    if (!editTicket) return;
    setText(editTicket.message.text);
    textareaRef.current?.focus();
    // Only the ticket nonce triggers this; nothing else is read here.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [editTicket]);

  const canSend = text.trim().length > 0;

  return (
    <form
      ref={formRef}
      className="w-full"
      onSubmit={(event) => {
        event.preventDefault();
        if (!canSend) return;
        onSend({ text: text.trim() });
        setText('');
      }}
    >
      <InputGroup
        className="rounded-2xl shadow-sm"
        style={{ backgroundColor: 'var(--color-card)' }}
      >
        <InputGroupTextarea
          {...lipiProps}
          ref={textareaRef}
          onChange={(event) => {
            lipiProps.onChange(event);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && !event.shiftKey && !event.nativeEvent.isComposing) {
              event.preventDefault();
              formRef.current?.requestSubmit();
              return;
            }
            lipiProps.onKeyDown(event);
          }}
          placeholder={placeholder}
          aria-label="Message"
          autoFocus={autoFocus}
          rows={3}
          style={{ fieldSizing: 'fixed' }}
          className="max-h-60 min-h-14 resize-none overflow-y-auto px-3.5 pt-3 font-devanagari text-base md:text-[15px]"
        />
        <InputGroupAddon align="block-end" className="gap-1 px-2 pb-2">
          {toolbar}
          <span
            className="mr-auto hidden items-center gap-1.5 pr-1 text-[11px] leading-none text-muted-foreground sm:flex"
            title="Alt+X toggles Devanagari typing"
          >
            {lipiOn ? (
              <>
                <span className="inline-flex h-5 items-center gap-1 rounded-md bg-muted px-1.5 leading-none">
                  <span className="font-mono text-[11px]">a</span>
                  <span aria-hidden>→</span>
                  <span className="font-devanagari text-xs">अ</span>
                </span>
                <span className="leading-none">typing</span>
              </>
            ) : (
              'Latin typing'
            )}
            <Kbd className="ml-1">Alt X</Kbd>
          </span>
          {busy && onStop ? (
            <>
              <InputGroupButton
                type="button"
                variant="outline"
                size="icon-sm"
                className={cn('rounded-full')}
                aria-label="Stop"
                onClick={onStop}
              >
                <SquareIcon className="size-3 fill-current" />
              </InputGroupButton>
              <InputGroupButton
                type="submit"
                variant="default"
                size="icon-sm"
                className="rounded-full transition-opacity duration-300 disabled:opacity-70"
                disabled={!canSend}
                aria-label="Queue message"
                title="Queue — sends after the current reply"
              >
                <ArrowUpIcon />
              </InputGroupButton>
            </>
          ) : (
            <InputGroupButton
              type="submit"
              variant="default"
              size="icon-sm"
              className="rounded-full transition-opacity duration-300 disabled:opacity-70"
              disabled={!canSend}
              aria-label="Send"
            >
              <ArrowUpIcon />
            </InputGroupButton>
          )}
        </InputGroupAddon>
      </InputGroup>
    </form>
  );
}
