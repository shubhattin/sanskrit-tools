import { SparklesIcon } from 'lucide-react';
import type { JevPick } from '@/lib/vidyut/api';
import { cn } from '@/lib/utils';

const JEV_TITLE =
  "Jev (TypeSafe's decision model) weighed the options the grammar leaves open and picked this one.";

/** Probability Jev gave an option, by its label (undefined when Jev was not asked). */
export function jevProbability(pick: JevPick | null | undefined, label: string) {
  if (!pick) return undefined;
  const i = pick.options.indexOf(label);
  return i >= 0 ? pick.probabilities[i] : undefined;
}

export function isJevChoice(pick: JevPick | null | undefined, label: string) {
  return !!pick && pick.options[pick.choice] === label;
}

/** "✦ Jev 93%" chip marking the option Jev chose. */
export function JevBadge({ p, className }: { p?: number; className?: string }) {
  return (
    <span
      title={JEV_TITLE}
      className={cn(
        'inline-flex shrink-0 items-center gap-1 rounded-full bg-violet-500/12 px-2 py-0.5 text-[11px] font-semibold text-violet-700 ring-1 ring-violet-500/30 dark:text-violet-300',
        className
      )}
    >
      <SparklesIcon className="size-3" aria-hidden />
      Jev{p !== undefined ? ` ${Math.round(p * 100)}%` : ''}
    </span>
  );
}

/** Thin bar for an option's probability. */
export function JevBar({ p }: { p: number }) {
  return (
    <span
      className="h-1 w-12 shrink-0 overflow-hidden rounded-full bg-muted"
      title={`Jev: ${Math.round(p * 100)}%`}
    >
      <span
        className="block h-full rounded-full bg-violet-500/70"
        style={{ width: `${Math.max(3, p * 100)}%` }}
      />
    </span>
  );
}

/** Small "thinking" line while Jev is asked. */
export function JevPending({ what }: { what: string }) {
  return (
    <span className="inline-flex items-center gap-1 text-[11px] text-violet-600/80 dark:text-violet-300/80">
      <SparklesIcon className="size-3 animate-pulse" aria-hidden /> Jev is weighing {what}…
    </span>
  );
}
