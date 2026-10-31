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
import type { ScriptListType } from 'lipilekhika';
import { SCRIPT_LIST } from 'lipilekhika';

export { SCRIPT_LIST };
export type { ScriptListType };

type FieldElement = HTMLInputElement | HTMLTextAreaElement;

interface LipiTypingState {
  enabled: boolean;
  setEnabled: (next: boolean) => void;
  toggle: () => void;
  script: ScriptListType;
  setScript: (next: ScriptListType) => void;
  typingContext: ReturnType<typeof createTypingContext>;
}

const LipiTypingContext = createContext<LipiTypingState | null>(null);

const ENABLED_KEY = 'sans-dict-lipi-enabled';
const SCRIPT_KEY = 'sans-dict-lipi-script';

function readStoredEnabled(): boolean {
  try {
    return localStorage.getItem(ENABLED_KEY) === '1';
  } catch {
    return false;
  }
}

function readStoredScript(): ScriptListType {
  try {
    const raw = localStorage.getItem(SCRIPT_KEY);
    if (raw && (SCRIPT_LIST as readonly string[]).includes(raw)) {
      return raw as ScriptListType;
    }
  } catch {
    // Ignore storage errors and fall back to the default.
  }
  return 'Devanagari';
}

export function LipiTypingProvider({ children }: { children: ReactNode }) {
  // Lipi Lekhika typing is off by default; the user opts in per device.
  const [enabled, setEnabledState] = useState<boolean>(() =>
    typeof window === 'undefined' ? false : readStoredEnabled()
  );
  const [script, setScriptState] = useState<ScriptListType>(() =>
    typeof window === 'undefined' ? 'Devanagari' : readStoredScript()
  );

  const typingContext = useMemo(() => createTypingContext(script), [script]);

  // Eagerly load the typing data so the first keystroke has no latency.
  useEffect(() => {
    void typingContext.ready;
  }, [typingContext]);

  const setEnabled = useCallback((next: boolean) => {
    setEnabledState(next);
    try {
      localStorage.setItem(ENABLED_KEY, next ? '1' : '0');
    } catch {
      // Non-fatal: typing still works for this session.
    }
  }, []);

  const toggle = useCallback(() => {
    setEnabledState((prev) => {
      try {
        localStorage.setItem(ENABLED_KEY, prev ? '0' : '1');
      } catch {
        // Non-fatal.
      }
      return !prev;
    });
  }, []);

  const setScript = useCallback((next: ScriptListType) => {
    setScriptState(next);
    try {
      localStorage.setItem(SCRIPT_KEY, next);
    } catch {
      // Non-fatal.
    }
  }, []);

  const value = useMemo<LipiTypingState>(
    () => ({ enabled, setEnabled, toggle, script, setScript, typingContext }),
    [enabled, setEnabled, toggle, script, setScript, typingContext]
  );

  return <LipiTypingContext.Provider value={value}>{children}</LipiTypingContext.Provider>;
}

export function useLipiTyping(): LipiTypingState {
  const ctx = useContext(LipiTypingContext);
  if (!ctx) throw new Error('useLipiTyping must be used within LipiTypingProvider');
  return ctx;
}

export function isLipiToggleKey(event: KeyboardEvent<FieldElement>): boolean {
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
 * Alt+X / Alt+C toggles typing (preventDefault so the browser menu stays shut).
 */
export function useLipiInput(value: string, setValue: (next: string) => void): LipiInputProps {
  const { enabled, toggle, typingContext } = useLipiTyping();

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
        toggle();
        return;
      }
      clearTypingContextOnKeyDown(event, typingContext);
    },
    [toggle, typingContext]
  );

  return useMemo(
    () => ({ value, onChange, onBeforeInput, onBlur, onKeyDown }),
    [value, onChange, onBeforeInput, onBlur, onKeyDown]
  );
}
