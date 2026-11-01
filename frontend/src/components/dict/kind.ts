import type { DictKind } from '@/lib/api/types';

/** Per-kind accent colours (badge, stripe, dot). Full class names for Tailwind. */
export const KIND_ACCENT: Record<DictKind, { badge: string; stripe: string; dot: string }> = {
  lexicon: {
    badge: 'bg-emerald-500/12 text-emerald-700 ring-emerald-500/25 dark:text-emerald-300',
    stripe: 'from-emerald-500 to-teal-400',
    dot: 'bg-emerald-500'
  },
  thesaurus: {
    badge: 'bg-violet-500/12 text-violet-700 ring-violet-500/25 dark:text-violet-300',
    stripe: 'from-violet-500 to-fuchsia-400',
    dot: 'bg-violet-500'
  },
  dhatu_work: {
    badge: 'bg-amber-500/12 text-amber-700 ring-amber-500/25 dark:text-amber-300',
    stripe: 'from-amber-500 to-orange-400',
    dot: 'bg-amber-500'
  },
  sutra_commentary: {
    badge: 'bg-sky-500/12 text-sky-700 ring-sky-500/25 dark:text-sky-300',
    stripe: 'from-sky-500 to-indigo-400',
    dot: 'bg-sky-500'
  },
  concordance: {
    badge: 'bg-rose-500/12 text-rose-700 ring-rose-500/25 dark:text-rose-300',
    stripe: 'from-rose-500 to-pink-400',
    dot: 'bg-rose-500'
  }
};
