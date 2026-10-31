/**
 * TanStack Router JSON-parses search params, so `?gana=3` arrives as the
 * number 3 and `?q=1.1` as the number 1.1. Validators must accept both forms.
 */

/** String param; numbers/booleans are turned back into their text. */
export function searchString(value: unknown): string | undefined {
  if (typeof value === 'string') return value;
  if (typeof value === 'number' || typeof value === 'boolean') return String(value);
  return undefined;
}

/** Non-negative integer param from a number or a numeric string. */
export function searchInt(value: unknown): number | undefined {
  const n = typeof value === 'number' ? value : typeof value === 'string' ? Number(value) : NaN;
  return Number.isInteger(n) && n >= 0 ? n : undefined;
}
