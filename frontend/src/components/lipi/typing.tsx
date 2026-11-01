import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type FormEvent,
  type KeyboardEvent,
  type ReactNode
} from 'react';
import {
  clearTypingContextOnKeyDown,
  createTypingContext,
  handleTypingBeforeInputEvent
} from 'lipilekhika/typing';

type FieldElement = HTMLInputElement | HTMLTextAreaElement;
type TypingContext = ReturnType<typeof createTypingContext>;

const LipiTypingContext = createContext<TypingContext | null>(null);

/** One shared Devanagari typing context (phonetic Latin → Devanagari). */
export function LipiTypingProvider({ children }: { children: ReactNode }) {
  const typingContext = useMemo(() => createTypingContext('Devanagari'), []);
  // Eagerly load the typing data so the first keystroke has no latency.
  useEffect(() => {
    void typingContext.ready;
  }, [typingContext]);
  return <LipiTypingContext.Provider value={typingContext}>{children}</LipiTypingContext.Provider>;
}

function useTypingContext(): TypingContext {
  const ctx = useContext(LipiTypingContext);
  if (!ctx) throw new Error('useLipiInput must be used within LipiTypingProvider');
  return ctx;
}

function isLipiToggleKey(event: KeyboardEvent<FieldElement>): boolean {
  return (
    event.altKey &&
    (event.key === 'x' || event.key === 'X' || event.key === 'c' || event.key === 'C')
  );
}

export interface LipiInputProps {
  value: string;
  onChange: (event: FormEvent<FieldElement>) => void;
  onBeforeInput: (event: FormEvent<FieldElement>) => void;
  onBlur: () => void;
  onKeyDown: (event: KeyboardEvent<FieldElement>) => void;
}

/**
 * Binds Lipi Lekhika realtime transliteration typing to a controlled input.
 * `defaultEnabled` follows the context (on for Sanskrit, off for English);
 * Alt+X / Alt+C flips it for the current field until `defaultEnabled` changes.
 */
export function useLipiInput(
  value: string,
  setValue: (next: string) => void,
  defaultEnabled: boolean
): { props: LipiInputProps; enabled: boolean; setEnabled: (next: boolean) => void } {
  const typingContext = useTypingContext();
  const [enabled, setEnabled] = useState(defaultEnabled);
  // Render-time adjustment: reset the override when the context flips.
  const [syncedDefault, setSyncedDefault] = useState(defaultEnabled);
  if (syncedDefault !== defaultEnabled) {
    setSyncedDefault(defaultEnabled);
    setEnabled(defaultEnabled);
  }

  const onChange = useCallback(
    (event: FormEvent<FieldElement>) => {
      setValue(event.currentTarget.value);
    },
    [setValue]
  );

  const onBeforeInput = useCallback(
    (event: FormEvent<FieldElement>) => {
      void handleTypingBeforeInputEvent(typingContext, event, setValue, enabled);
    },
    [typingContext, setValue, enabled]
  );

  const onBlur = useCallback(() => {
    typingContext.clearContext();
  }, [typingContext]);

  const onKeyDown = useCallback(
    (event: KeyboardEvent<FieldElement>) => {
      if (isLipiToggleKey(event)) {
        event.preventDefault();
        typingContext.clearContext();
        setEnabled((v) => !v);
        return;
      }
      clearTypingContextOnKeyDown(event, typingContext);
    },
    [typingContext]
  );

  const props = useMemo(
    () => ({ value, onChange, onBeforeInput, onBlur, onKeyDown }),
    [value, onChange, onBeforeInput, onBlur, onKeyDown]
  );
  const setEnabledClearing = useCallback(
    (next: boolean) => {
      typingContext.clearContext();
      setEnabled(next);
    },
    [typingContext]
  );
  return { props, enabled, setEnabled: setEnabledClearing };
}
