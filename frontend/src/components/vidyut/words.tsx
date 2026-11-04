// A word of a sandhi reading and everything vidyut knows about it: its analyses (kosha), which
// one Jev picked in context, and ways on: the derivation, all forms of the stem / root, the
// dictionary.
import { Link } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import {
  ArrowRightIcon,
  BookOpenIcon,
  CheckIcon,
  ChevronLeftIcon,
  GitBranchIcon,
  SparklesIcon,
  TableIcon
} from 'lucide-react';
import { Popover, PopoverContent, PopoverTitle, PopoverTrigger } from '@/components/ui/popover';
import { useOpenDerivation } from './derivation';
import { padaQuery, type PadaInfo, type Part, type WordSpec } from '@/lib/vidyut/api';
import { padaTags } from '@/lib/vidyut/labels';
import { cn } from '@/lib/utils';

/** Word class → the accent of its tool (dhātu emerald, nāma sky, avyaya amber). */
export const KIND = {
  tinanta: {
    label: 'क्रियापदम्',
    hint: 'verb',
    dot: 'bg-emerald-500',
    text: 'text-emerald-700 dark:text-emerald-300'
  },
  subanta: {
    label: 'सुबन्तम्',
    hint: 'noun / adjective',
    dot: 'bg-sky-500',
    text: 'text-sky-700 dark:text-sky-300'
  },
  avyaya: {
    label: 'अव्ययम्',
    hint: 'indeclinable',
    dot: 'bg-amber-500',
    text: 'text-amber-700 dark:text-amber-300'
  }
} as const;

/** Where "all forms" of an analysis' stem or root live. */
export function formsLink(a: PadaInfo) {
  const w = a.word;
  if (!w) return null;
  const dhatu = (d: Extract<WordSpec, { kind: 'tinanta' }>['dhatu'], krt = false) =>
    ({
      to: '/vidyut/dhatu',
      search: {
        code: d.code,
        upa: d.upasargas?.length ? d.upasargas : undefined,
        sanadi: d.sanadi?.length ? d.sanadi : undefined,
        tab: krt ? 'krt' : undefined
      },
      label: krt ? 'Kṛdantas of the root' : 'Conjugation'
    }) as const;
  switch (w.kind) {
    case 'tinanta':
      return dhatu(w.dhatu);
    case 'subanta':
      return {
        to: '/vidyut/nama',
        search: {
          stem: a.lemma,
          linga: w.linga,
          from: w.stem.kind === 'basic' ? undefined : w.stem
        },
        label: 'Declension'
      } as const;
    case 'stem':
      return w.stem.kind === 'krdanta' ? dhatu(w.stem.dhatu, true) : null;
    default:
      return null;
  }
}

/** `उद्भवन ← उद्भू + ल्युट्` */
function Lemma({ a, className }: { a: PadaInfo; className?: string }) {
  return (
    <span className={cn('font-devanagari', className)}>
      {a.lemma}
      {a.dhatu && a.krt ? (
        <span className="text-muted-foreground">
          {' '}
          ← {a.dhatu} + {a.krt}
        </span>
      ) : null}
    </span>
  );
}

function Analysis({
  a,
  word,
  jev,
  first
}: {
  a: PadaInfo;
  word: string;
  /** Jev's probability when it picked this analysis. */
  jev?: number;
  first: boolean;
}) {
  const open = useOpenDerivation();
  const forms = formsLink(a);
  const kind = KIND[a.kind];
  const chosen = jev !== undefined;
  return (
    <li
      className={cn(
        'flex flex-col gap-1.5 rounded-lg px-2.5 py-2',
        chosen ? 'bg-violet-500/8 ring-1 ring-violet-500/25' : first ? 'bg-muted/60' : ''
      )}
    >
      <div className="flex items-start gap-2">
        <span className={cn('mt-1.5 size-2 shrink-0 rounded-full', kind.dot)} aria-hidden />
        <div className="flex min-w-0 flex-1 flex-col">
          <Lemma a={a} className="text-sm font-semibold" />
          <span className="font-devanagari text-xs text-muted-foreground">
            <span className={kind.text}>{kind.label}</span>
            {a.kind !== 'avyaya' ? ` · ${padaTags(a)}` : ''}
          </span>
        </div>
        {chosen ? (
          <span
            className="inline-flex shrink-0 items-center gap-0.5 text-[11px] font-semibold text-violet-700 dark:text-violet-300"
            title="Jev's reading of this word in the sentence"
          >
            <SparklesIcon className="size-3" aria-hidden />
            {Math.round(jev * 100)}%
          </span>
        ) : null}
      </div>
      {a.word || forms ? (
        <div className="flex flex-wrap gap-1 pl-4">
          {a.word ? (
            <button
              type="button"
              onClick={() =>
                open({
                  spec: a.word!,
                  form: word,
                  context: `${a.lemma} · ${padaTags(a) || kind.label}`
                })
              }
              className="inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium transition-colors hover:border-primary/40 hover:bg-primary/5 hover:text-primary focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            >
              <GitBranchIcon className="size-3" aria-hidden /> Derivation
            </button>
          ) : null}
          {forms ? (
            <Link
              to={forms.to}
              search={forms.search as never}
              className="inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium transition-colors hover:border-primary/40 hover:bg-primary/5 hover:text-primary focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            >
              <TableIcon className="size-3" aria-hidden /> {forms.label}
            </Link>
          ) : null}
        </div>
      ) : null}
    </li>
  );
}

/** Where the word comes from, in words (for badges and screen readers). */
function sourceOf(part: Part) {
  if (part.dictionary) return { label: 'In the dictionaries', short: 'कोशे' };
  if (part.known)
    return { label: 'Generated by the grammar (not in the dictionaries)', short: 'व्याकरणे' };
  return { label: 'Not a known word', short: '—' };
}

/**
 * A word card: the form, its (Jev-picked, else likeliest) analysis; opens every analysis with
 * its derivation, all forms and the dictionary.
 */
export function WordCard({
  part,
  jevKey,
  jevP
}: {
  part: Part;
  /** `PadaInfo.key` Jev picked for this word in context. */
  jevKey?: string;
  jevP?: number;
}) {
  const shown = part.analyses.find((a) => a.key === jevKey) ?? part.analyses[0];
  const source = sourceOf(part);
  const kind = shown ? KIND[shown.kind] : null;
  return (
    <Popover>
      <PopoverTrigger
        aria-label={`${part.text}: ${shown ? `${shown.lemma}, ${kind!.hint}` : source.label}. Show its analyses`}
        className={cn(
          'group flex min-w-20 flex-col items-start gap-0.5 rounded-xl border bg-background px-3 py-2 text-left shadow-xs transition-all hover:-translate-y-px hover:shadow-md focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none data-popup-open:ring-2 data-popup-open:ring-primary/40',
          part.known ? 'hover:border-primary/40' : 'border-dashed bg-muted/30'
        )}
      >
        <span
          className={cn(
            'font-devanagari text-xl leading-tight font-semibold',
            part.known ? 'text-foreground' : 'text-muted-foreground'
          )}
        >
          {part.text}
        </span>
        {shown ? (
          <>
            <span className="flex items-center gap-1.5 text-xs">
              <span className={cn('size-1.5 shrink-0 rounded-full', kind!.dot)} aria-hidden />
              <Lemma a={shown} className="text-foreground/80" />
            </span>
            <span className="font-devanagari text-[11px] leading-snug text-muted-foreground">
              {shown.kind === 'avyaya' ? kind!.label : padaTags(shown)}
              {jevKey && shown.key === jevKey ? (
                <SparklesIcon
                  className="ml-1 inline size-3 text-violet-500"
                  aria-label="picked by Jev"
                />
              ) : null}
            </span>
          </>
        ) : (
          <span className="text-[11px] text-muted-foreground">
            {part.dictionary ? 'in the dictionaries' : 'unknown'}
          </span>
        )}
      </PopoverTrigger>
      <PopoverContent align="start" className="w-[22rem] max-w-[calc(100vw-1rem)] gap-2 p-2">
        <WordDetails part={part} jevKey={jevKey} jevP={jevP} />
      </PopoverContent>
    </Popover>
  );
}

/** Everything about one word, for a popover: its source, every analysis (Jev's pick marked)
 * with derivation and forms, and the dictionary. */
export function WordDetails({
  part,
  jevKey,
  jevP,
  back
}: {
  part: Part;
  jevKey?: string;
  jevP?: number;
  /** A way back up (a word opened from its chunk): label and action. */
  back?: { label: string; onClick: () => void };
}) {
  const source = sourceOf(part);
  const lemmas = [...new Set(part.analyses.map((a) => a.lemma))].filter((l) => l !== part.text);
  return (
    <>
      {back ? (
        <button
          type="button"
          onClick={back.onClick}
          className="inline-flex items-center gap-1 self-start rounded-md px-1.5 py-0.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
        >
          <ChevronLeftIcon className="size-3.5" aria-hidden />
          <span className="font-devanagari">{back.label}</span>
        </button>
      ) : null}
      <div className="flex items-baseline justify-between gap-2 px-1.5 pt-1">
        <PopoverTitle className="font-devanagari text-2xl font-bold">{part.text}</PopoverTitle>
        <span
          className={cn(
            'inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium',
            part.dictionary
              ? 'bg-primary/10 text-primary'
              : part.known
                ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
                : 'bg-muted text-muted-foreground'
          )}
          title={source.label}
        >
          {part.known ? <CheckIcon className="size-3" aria-hidden /> : null}
          {part.dictionary ? 'Dictionary word' : part.known ? 'Grammar form' : 'Unknown'}
        </span>
      </div>
      {part.analyses.length ? (
        <>
          <p className="px-1.5 text-[11px] text-muted-foreground">
            {part.analyses.length === 1
              ? 'What it is'
              : `${part.analyses.length} ways to read it${jevKey ? ' · ✦ Jev’s pick in this sentence' : ', likeliest first'}`}
          </p>
          <ul className="flex max-h-80 flex-col gap-1 overflow-y-auto">
            {part.analyses.map((a, i) => (
              <Analysis
                key={a.key}
                a={a}
                word={part.text}
                first={!jevKey && i === 0}
                jev={a.key === jevKey ? jevP : undefined}
              />
            ))}
          </ul>
        </>
      ) : (
        <p className="px-1.5 text-xs text-muted-foreground">
          {part.dictionary
            ? 'A dictionary headword the grammar does not generate (an indeclinable, a name, a phrase).'
            : 'Neither the dictionaries nor the grammar know this form: likely not the intended split.'}
        </p>
      )}
      {part.known ? (
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1 border-t px-1.5 pt-2 text-xs">
          <BookOpenIcon className="size-3.5 text-muted-foreground" aria-hidden />
          {[part.text, ...lemmas].slice(0, 4).map((w) => (
            <Link
              key={w}
              to="/"
              search={{ q: w }}
              className="font-devanagari font-medium text-primary underline-offset-4 hover:underline"
            >
              {w}
            </Link>
          ))}
          <span className="text-muted-foreground">in the dictionaries</span>
        </div>
      ) : null}
    </>
  );
}

/** Where an analysis leads in one tool: the root's page (dhātu) or the stem's (nāma). */
function toolLink(a: PadaInfo, tool: 'dhatu' | 'nama') {
  const w = a.word;
  if (!w) return null;
  if (tool === 'nama') return w.kind === 'subanta' ? formsLink(a) : null;
  if (w.kind === 'tinanta') return formsLink(a);
  // A kṛdanta (गच्छता, गत्वा): its root's kṛdantas.
  const stem = w.kind === 'subanta' || w.kind === 'stem' ? w.stem : null;
  if (stem?.kind !== 'krdanta') return null;
  return {
    to: '/vidyut/dhatu',
    search: {
      code: stem.dhatu.code,
      upa: stem.dhatu.upasargas?.length ? stem.dhatu.upasargas : undefined,
      tab: 'krt'
    },
    label: 'Kṛdantas of the root'
  } as const;
}

/**
 * "गच्छति is a form of गम्": when a root / stem field holds an inflected form, links to the
 * root or stem it comes from (vidyut's kosha).
 */
export function FormSuggestions({
  text,
  tool,
  onPick
}: {
  text: string;
  tool: 'dhatu' | 'nama';
  onPick?: () => void;
}) {
  const word = text.trim();
  const enabled = word.length > 1 && !/\s/.test(word) && !/[0-9a-zA-Z]/.test(word);
  const q = useQuery({ ...padaQuery(word), enabled, retry: false });
  if (!enabled || !q.data) return null;
  // What a chip says: the root (dhātu tool; kṛdantas name their suffix) or the stem and its
  // grammar (nāma tool).
  const chip = (a: PadaInfo) =>
    tool === 'dhatu'
      ? {
          head: a.dhatu ?? a.lemma,
          tail: a.krt ? `${a.krt} → ${a.lemma}` : padaTags(a)
        }
      : {
          head: a.dhatu && a.krt ? `${a.lemma} ← ${a.dhatu} + ${a.krt}` : a.lemma,
          tail: padaTags(a) || KIND[a.kind].label
        };
  const seen = new Set<string>();
  const items = q.data.word.analyses
    .map((a) => ({ a, link: toolLink(a, tool), chip: chip(a) }))
    .filter(
      (
        x
      ): x is {
        a: PadaInfo;
        link: NonNullable<ReturnType<typeof toolLink>>;
        chip: ReturnType<typeof chip>;
      } => {
        if (!x.link) return false;
        // Not "गम् is a form of गम्".
        if (x.chip.head === word) return false;
        const k = `${x.chip.head}|${x.chip.tail}`;
        if (seen.has(k)) return false;
        seen.add(k);
        return true;
      }
    )
    .slice(0, 4);
  if (!items.length) return null;
  return (
    <div className="flex flex-wrap items-center gap-2 rounded-xl border border-dashed bg-muted/30 px-3 py-2 text-sm">
      <span className="text-foreground/75">
        <span className="font-devanagari font-semibold text-foreground">{q.data.word.text}</span> is
        a form of
      </span>
      {items.map(({ a, link, chip }) => (
        <Link
          key={`${chip.head}|${chip.tail}`}
          to={link.to}
          search={link.search as never}
          onClick={onPick}
          className="inline-flex items-center gap-1.5 rounded-full border bg-background px-2.5 py-0.5 transition-colors hover:border-primary/40 hover:bg-primary/5 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
        >
          <span className={cn('size-1.5 rounded-full', KIND[a.kind].dot)} aria-hidden />
          <span className="font-devanagari font-semibold">{chip.head}</span>
          <span className="font-devanagari text-xs text-muted-foreground">{chip.tail}</span>
          <ArrowRightIcon className="size-3 text-muted-foreground" aria-hidden />
        </Link>
      ))}
    </div>
  );
}
