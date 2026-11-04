// Reading a text as words, shared by the Cheda page and Sandhi's split tab: the split (per chunk,
// its best readings), Jev's pick among each chunk's readings with the whole text in view, the
// reader's own choices, and Jev's pick of each word's analysis in the sentence.
import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { ScissorsIcon } from 'lucide-react';
import { radioKeys } from './controls';
import { JevBadge, JevBar, isJevChoice, jevProbability } from './jev';
import {
  jevAnalysesQuery,
  jevSplitQuery,
  splitQuery,
  type ChunkSplit,
  type JevPick,
  type Reading,
  type WordPick
} from '@/lib/vidyut/api';
import { cn } from '@/lib/utils';

export const CHEDA_SAMPLES = [
  {
    label: 'गीता १.१',
    text: 'धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः।\nमामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय॥'
  },
  {
    label: 'विद्या ददाति विनयम्',
    text: 'विद्या ददाति विनयं विनयाद्याति पात्रताम्।\nपात्रत्वाद्धनमाप्नोति धनाद्धर्मं ततः सुखम्॥'
  },
  { label: 'रामो वनं गच्छति', text: 'रामो वनं गच्छति' }
];

export function useReading(q: string) {
  const split = useQuery({ ...splitQuery(q), enabled: !!q });
  const chunks = split.data ?? [];
  const askJev = chunks.some((c) => c.readings.length > 1);
  const jev = useQuery({ ...jevSplitQuery(q), enabled: !!q && askJev });
  // The reader's choice of reading per chunk (by label); default: Jev's pick, else the first.
  const [chosen, setChosen] = useState<Record<string, string>>({});
  const chunkPick = (i: number): JevPick | undefined => {
    const p = jev.data?.[i];
    return p && p.chunk === chunks[i]?.chunk ? (p.pick ?? undefined) : undefined;
  };
  const selected = chunks.map((c, i) => {
    const jp = chunkPick(i);
    const label = chosen[`${i}:${c.chunk}`] ?? (jp ? jp.options[jp.choice] : undefined);
    return Math.max(0, label ? c.readings.findIndex((r) => r.label === label) : 0);
  });
  // Jev then reads each word of the chosen readings in the context of the whole text.
  const words = chunks.flatMap((c, i) => c.readings[selected[i]!]?.parts ?? []);
  const ambiguous = words.some((p) => p.analyses.length > 1);
  const settled = !askJev || !jev.isFetching;
  const picks = useQuery({
    ...jevAnalysesQuery(
      q,
      words.map((p) => p.text)
    ),
    enabled: !!q && ambiguous && settled
  });
  // Where each chunk's words start in `words`; picks only count for the words sent.
  const sizes = chunks.map((c, i) => c.readings[selected[i]!]?.parts.length ?? 0);
  const starts = sizes.map((_, i) => sizes.slice(0, i).reduce((a, b) => a + b, 0));
  const fresh = picks.data?.length === words.length;
  return {
    split,
    chunks,
    selected,
    chunkPick,
    choose: (i: number, label: string) =>
      setChosen((m) => ({ ...m, [`${i}:${chunks[i]!.chunk}`]: label })),
    /** Jev's analysis pick per word of chunk `i`'s selected reading. */
    wordPicks: (i: number): (WordPick | undefined)[] | undefined =>
      fresh ? picks.data!.slice(starts[i], starts[i]! + sizes[i]!) : undefined,
    readingsPending: askJev && jev.isFetching,
    wordsPending: picks.isFetching && !jev.isFetching,
    jevFailed: jev.isError || picks.isError
  };
}

/** Jev's chosen analysis key and its probability, for a `WordCard`. */
export function pickOf(p: WordPick | undefined) {
  const pick = p?.pick;
  return pick
    ? { jevKey: pick.options[pick.choice], jevP: pick.probabilities[pick.choice] }
    : { jevKey: undefined, jevP: undefined };
}

/** Marks the reading vidyut's statistical segmenter chose. */
export function ChedaBadge() {
  return (
    <span
      title="cheda, vidyut's statistical segmenter (trained on Sanskrit texts), picked this reading"
      className="inline-flex shrink-0 items-center gap-1 rounded-full bg-emerald-500/12 px-2 py-0.5 text-[11px] font-semibold text-emerald-800 ring-1 ring-emerald-500/30 dark:text-emerald-300"
    >
      <ScissorsIcon className="size-3" aria-hidden />
      cheda
    </span>
  );
}

/** One reading as plain text: dictionary words bold, grammar-only forms green, unknowns struck. */
export function ReadingText({ reading }: { reading: Reading }) {
  return (
    <span className="flex flex-wrap items-baseline gap-x-1.5 font-devanagari">
      {reading.parts.map((p, i) => (
        <span key={i} className="flex items-baseline gap-1.5">
          {i > 0 ? <span className="text-muted-foreground">+</span> : null}
          <span
            className={cn(
              p.dictionary
                ? 'font-semibold text-foreground'
                : p.known
                  ? 'font-semibold text-emerald-700 dark:text-emerald-300'
                  : 'text-muted-foreground line-through decoration-muted-foreground/40'
            )}
          >
            {p.text}
          </span>
        </span>
      ))}
    </span>
  );
}

/** A chunk's readings as a radio group, with cheda's and Jev's marks. */
export function ReadingChoices({
  chunk,
  jev,
  selected,
  onSelect
}: {
  chunk: ChunkSplit;
  jev?: JevPick;
  selected: number;
  onSelect: (i: number) => void;
}) {
  return (
    <div
      role="radiogroup"
      aria-label={`Readings of ${chunk.chunk}`}
      onKeyDown={radioKeys}
      className="flex flex-col gap-0.5"
    >
      {chunk.readings.map((r, i) => {
        const on = i === selected;
        const p = jevProbability(jev, r.label);
        return (
          <button
            key={r.label}
            type="button"
            role="radio"
            aria-checked={on}
            tabIndex={on ? 0 : -1}
            onClick={() => onSelect(i)}
            className={cn(
              'flex items-center gap-3 rounded-lg px-2.5 py-1.5 text-left text-base transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
              on ? 'bg-primary/8 ring-1 ring-primary/25' : 'hover:bg-muted/70'
            )}
          >
            <span
              className={cn(
                'grid size-4 shrink-0 place-items-center rounded-full border',
                on ? 'border-primary bg-primary' : 'border-muted-foreground/40'
              )}
              aria-hidden
            >
              {on ? <span className="size-1.5 rounded-full bg-primary-foreground" /> : null}
            </span>
            <ReadingText reading={r} />
            <span className="ml-auto flex shrink-0 items-center gap-2">
              {r.cheda ? <ChedaBadge /> : null}
              {isJevChoice(jev, r.label) ? (
                <JevBadge p={p} />
              ) : p !== undefined ? (
                <JevBar p={p} />
              ) : null}
            </span>
          </button>
        );
      })}
    </div>
  );
}

/** Legend for the word cards. */
export function Legend({ children }: { children?: React.ReactNode }) {
  const dot = (c: string) => <span className={cn('size-2 rounded-full', c)} aria-hidden />;
  return (
    <p className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
      <span className="flex items-center gap-1.5">{dot('bg-emerald-500')} verb</span>
      <span className="flex items-center gap-1.5">{dot('bg-sky-500')} noun / adjective</span>
      <span className="flex items-center gap-1.5">{dot('bg-amber-500')} indeclinable</span>
      {children ?? <span>Click a word for all its analyses, its derivation and its forms.</span>}
    </p>
  );
}
