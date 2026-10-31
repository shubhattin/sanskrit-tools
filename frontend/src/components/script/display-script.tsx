import { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { transliterate, type ScriptListType } from 'lipilekhika';

export type DisplayScript = ScriptListType;

interface DisplayScriptState {
  script: DisplayScript;
  setScript: (next: DisplayScript) => void;
}

const DisplayScriptContext = createContext<DisplayScriptState | null>(null);

const STORAGE_KEY = 'sans-dict-display-script';

function readStored(): DisplayScript {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return raw as DisplayScript;
  } catch {
    // Ignore and fall back.
  }
  return 'Devanagari';
}

/** Module-level cache so repeated headwords transliterate once per session. */
const transliterationCache = new Map<string, string>();

async function transliterateCached(text: string, to: DisplayScript): Promise<string> {
  if (!text || to === 'Devanagari') return text;
  const key = `${to}${text}`;
  const hit = transliterationCache.get(key);
  if (hit !== undefined) return hit;
  try {
    const out = await transliterate(text, 'Devanagari', to);
    transliterationCache.set(key, out);
    return out;
  } catch {
    return text;
  }
}

export function DisplayScriptProvider({ children }: { children: ReactNode }) {
  const [script, setScriptState] = useState<DisplayScript>(() =>
    typeof window === 'undefined' ? 'Devanagari' : readStored()
  );

  const setScript = (next: DisplayScript) => {
    setScriptState(next);
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // Non-fatal.
    }
  };

  const value = useMemo(() => ({ script, setScript }), [script]);
  return <DisplayScriptContext.Provider value={value}>{children}</DisplayScriptContext.Provider>;
}

export function useDisplayScript(): DisplayScriptState {
  const ctx = useContext(DisplayScriptContext);
  if (!ctx) throw new Error('useDisplayScript must be used within DisplayScriptProvider');
  return ctx;
}

/** Transliterate one Devanagari string to the display script (no-op when Devanagari). */
export function useDisplayText(text: string): string {
  const { script } = useDisplayScript();
  const [out, setOut] = useState<string>(text);
  // Sync fast path during render (no effect): Devanagari and cache hits resolve
  // immediately; the effect below only fetches cache misses.
  const [sig, setSig] = useState<string>(() => `${script}${text}`);
  const current = `${script}${text}`;
  if (sig !== current) {
    setSig(current);
    if (script === 'Devanagari') setOut(text);
    else {
      const hit = transliterationCache.get(current);
      if (hit !== undefined) setOut(hit);
    }
  }

  useEffect(() => {
    if (script === 'Devanagari') return;
    if (transliterationCache.has(`${script}${text}`)) return;
    let cancelled = false;
    void transliterateCached(text, script).then((result) => {
      if (!cancelled) setOut(result);
    });
    return () => {
      cancelled = true;
    };
  }, [text, script]);

  if (script === 'Devanagari') return text;
  return out;
}

/**
 * Batch-transliterate headwords to the display script in one call.
 * Returns the source texts unchanged while Devanagari is selected (zero cost).
 */
export function useDisplayTexts(texts: string[]): string[] {
  const { script } = useDisplayScript();
  const key = useMemo(() => texts.join('\n'), [texts]);
  const [outs, setOuts] = useState<string[]>(texts);
  // Sync fast path during render (no effect): see useDisplayText.
  const [sig, setSig] = useState<string>(() => `${script}\n${key}`);
  const current = `${script}\n${key}`;
  if (sig !== current) {
    setSig(current);
    if (script === 'Devanagari') {
      setOuts(texts);
    } else {
      const parts = key.split('\n');
      if (parts.every((p) => transliterationCache.has(`${script}${p}`))) {
        setOuts(parts.map((p) => transliterationCache.get(`${script}${p}`) ?? p));
      }
    }
  }

  useEffect(() => {
    if (script === 'Devanagari') return;
    const parts = key.split('\n');
    if (parts.every((p) => transliterationCache.has(`${script}${p}`))) return;
    let cancelled = false;
    void transliterate(parts, 'Devanagari', script)
      .then((results) => {
        if (cancelled) return;
        parts.forEach((p, i) => transliterationCache.set(`${script}${p}`, results[i]!));
        setOuts(results);
      })
      .catch(() => {
        if (!cancelled) setOuts(parts);
      });
    return () => {
      cancelled = true;
    };
  }, [key, script]);

  if (script === 'Devanagari') return texts;
  return outs;
}
