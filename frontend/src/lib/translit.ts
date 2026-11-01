import type { ScriptLangType, TransliterationOptions } from 'lipilekhika';

/**
 * Central transliteration entry point (same approach as ../krida
 * `tools/transliterator.ts`): the JS `transliterate` is used until the WASM
 * build has been fetched, which happens lazily after first use / idle.
 * Everything that transliterates on the client goes through here.
 */

type TransliterateFn = typeof import('lipilekhika').transliterate;

let fn: TransliterateFn | null = null;
let jsLoad: Promise<TransliterateFn> | null = null;
let wasmLoad: Promise<void> | null = null;

function loadJs(): Promise<TransliterateFn> {
  jsLoad ??= import('lipilekhika').then((m) => {
    fn ??= m.transliterate;
    return fn;
  });
  return jsLoad;
}

/** Swap in the WASM transliterator (idempotent; failures keep the JS one). */
export function loadWasmTransliterate(): Promise<void> {
  wasmLoad ??= import('lipilekhika')
    .then(async (m) => {
      await m.preloadWasm();
      fn = m.transliterate_wasm as TransliterateFn;
    })
    .catch(() => {
      // Keep the JS implementation.
    });
  return wasmLoad;
}

if (typeof window !== 'undefined') {
  // Like krida: load the WASM build shortly after start-up, off the critical path.
  setTimeout(() => void loadWasmTransliterate(), 1400);
}

export async function transliterateText<T extends string | string[]>(
  text: T,
  from: ScriptLangType,
  to: ScriptLangType,
  options?: TransliterationOptions
): Promise<T extends string ? string : string[]> {
  const f = fn ?? (await loadJs());
  return (await f(text, from, to, options)) as T extends string ? string : string[];
}

const DEVANAGARI_DIGITS = '०१२३४५६७८९';

/** Devanagari digits → ASCII (१.१.१ → 1.1.1). */
export function asciiDigits(text: string): string {
  return text.replace(/[०-९]/g, (d) => String(DEVANAGARI_DIGITS.indexOf(d)));
}

const HAS_LATIN = /[A-Za-zÀ-ɏḀ-ỿ]/;
const HAS_DIACRITIC_LATIN = /[À-ɏḀ-ỿ]/;

/**
 * Devanagari readings of a (partly) Latin query: lipilekhika "Normal"
 * (vRddhi → वृद्धि) and, when IAST diacritics are present, "Romanized"
 * (vṛddhi → वृद्धि). Devanagari input is returned as is.
 */
export async function devanagariCandidates(text: string): Promise<string[]> {
  const t = text.trim();
  if (!t) return [];
  if (!HAS_LATIN.test(t)) return [t];
  const out = [await transliterateText(t, 'Normal', 'Devanagari')];
  if (HAS_DIACRITIC_LATIN.test(t)) out.push(await transliterateText(t, 'Romanized', 'Devanagari'));
  return [...new Set(out)];
}
