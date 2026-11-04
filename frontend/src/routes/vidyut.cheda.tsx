// पदच्छेदः: a verse or sentence segmented into words and each word tagged. The split, Jev's
// readings (with the whole text in view) and Jev's per-word analyses come from `useReading`;
// this page lays them out line by line, with other readings of a chunk behind a popover.
import { useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import {
  ChevronRightIcon,
  GitBranchIcon,
  ScanTextIcon,
  ScrollTextIcon,
  TableIcon
} from 'lucide-react';
import { Button, buttonVariants } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTitle, PopoverTrigger } from '@/components/ui/popover';
import { DevaTextarea, ErrorNote, LoadingBlock, Segmented } from '@/components/vidyut/controls';
import { JevPending } from '@/components/vidyut/jev';
import {
  CHEDA_SAMPLES,
  ChedaBadge,
  ReadingChoices,
  pickOf,
  useReading
} from '@/components/vidyut/reading';
import { KIND, WordDetails, formsLink } from '@/components/vidyut/words';
import { useOpenDerivation } from '@/components/vidyut/derivation';
import { searchString } from '@/lib/search-params';
import { padaTags } from '@/lib/vidyut/labels';
import type { ChunkSplit, JevPick, Part, WordPick } from '@/lib/vidyut/api';
import { cn } from '@/lib/utils';

type View = 'cards' | 'table';

interface ChedaSearch {
  q?: string;
  view?: View;
}

export const Route = createFileRoute('/vidyut/cheda')({
  validateSearch: (s: Record<string, unknown>): ChedaSearch => ({
    q: searchString(s.q),
    view: s.view === 'table' ? 'table' : undefined
  }),
  component: ChedaTool
});

function ChedaTool() {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: Route.fullPath });
  const q = (search.q ?? '').trim();
  const [text, setText] = useState(search.q ?? '');
  const [synced, setSynced] = useState(search.q);
  if (synced !== search.q) {
    setSynced(search.q);
    setText(search.q ?? '');
  }
  const read = (t: string) => void navigate({ search: (prev) => ({ ...prev, q: t || undefined }) });
  return (
    <div className="flex flex-col gap-5">
      <form
        className="flex max-w-3xl flex-col gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          read(text.trim());
        }}
      >
        <DevaTextarea
          value={text}
          onChange={setText}
          label="Text to read"
          rows={3}
          placeholder={
            'A śloka or a sentence, as written (up to 24 words)\nधर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः।'
          }
        />
        <div className="flex flex-wrap items-center gap-2">
          <Button type="submit" size="sm" disabled={!text.trim()}>
            <ScanTextIcon aria-hidden /> Read
          </Button>
          <span className="ml-1 text-xs text-muted-foreground">Try</span>
          {CHEDA_SAMPLES.map((s) => (
            <button
              key={s.label}
              type="button"
              onClick={() => {
                setText(s.text);
                read(s.text);
              }}
              className="rounded-full border px-2.5 py-0.5 font-devanagari text-xs text-foreground/80 transition-colors hover:border-primary/50 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
            >
              {s.label}
            </button>
          ))}
        </div>
      </form>
      {q ? (
        <Result
          key={q}
          q={q}
          view={search.view ?? 'cards'}
          onView={(v) =>
            void navigate({
              search: (prev) => ({ ...prev, view: v === 'cards' ? undefined : v }),
              replace: true
            })
          }
        />
      ) : (
        <p className="max-w-2xl text-sm text-muted-foreground">
          The text is split into words (sandhi undone), and each word is tagged with what it is:
          stem and gender, case and number; or root, tense, person and number. cheda, vidyut’s
          statistical segmenter, and Jev read the whole text at once, so a word is read in the light
          of its neighbours. Where a word could be split another way, the other readings are one
          click away.
        </p>
      )}
    </div>
  );
}

/** A written token: punctuation around the letters (युयुत्सवः। → युयुत्सवः + ।). */
const TOKEN = /^([^\u0900-\u0963\u0970-\u097F]*)(.*?)([^\u0900-\u0963\u0970-\u097F]*)$/u;
const HAS_LETTER = /[\u0900-\u0963\u0970-\u097F]/u;

type Token = { before: string; word: string; after: string; chunk: number };

/** The text as written, line by line, each lettered token tied to its chunk (in order). Null when
 * the count differs from the server's chunks. */
function layout(q: string, chunks: number): Token[][] | null {
  let n = 0;
  const lines = q
    .split('\n')
    .map((l) =>
      l
        .split(/\s+/)
        .filter((t) => HAS_LETTER.test(t))
        .map((t) => {
          const m = TOKEN.exec(t)!;
          return { before: m[1]!, word: m[2]!, after: m[3]!, chunk: n++ };
        })
    )
    .filter((l) => l.length);
  return n === chunks ? lines : null;
}

function Result({ q, view, onView }: { q: string; view: View; onView: (v: View) => void }) {
  const r = useReading(q);
  if (r.split.isPending) return <LoadingBlock rows={3} label="Reading" />;
  if (r.split.isError) return <ErrorNote error={r.split.error} />;
  const chunks = r.chunks;
  const lines = layout(q, chunks.length) ?? [
    chunks.map((c, i) => ({ before: '', word: c.chunk, after: '', chunk: i }))
  ];
  return (
    <section aria-label="Words of the text" className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <Segmented
          size="sm"
          label="View"
          value={view}
          onChange={onView}
          options={[
            {
              id: 'cards',
              label: (
                <span className="inline-flex items-center gap-1">
                  <ScrollTextIcon className="size-3.5" aria-hidden /> Text
                </span>
              )
            },
            {
              id: 'table',
              label: (
                <span className="inline-flex items-center gap-1">
                  <TableIcon className="size-3.5" aria-hidden /> Table
                </span>
              )
            }
          ]}
        />
        <span className="text-xs text-muted-foreground">
          {view === 'cards' ? 'Click a word to see how it splits and what it is.' : null}
        </span>
        <span className="ml-auto" aria-live="polite">
          {r.readingsPending ? (
            <JevPending what="the readings" />
          ) : r.wordsPending ? (
            <JevPending what="what each word is" />
          ) : null}
        </span>
      </div>
      {view === 'cards' ? (
        <div className="rounded-2xl border bg-card px-5 py-6 shadow-sm sm:px-8">
          <div className="flex flex-col gap-3 font-devanagari text-2xl leading-relaxed sm:text-[1.7rem]">
            {lines.map((line, li) => (
              <p key={li} className="flex flex-wrap items-baseline gap-x-3 gap-y-2">
                {line.map((t) => (
                  <span key={t.chunk} className="inline-flex items-baseline">
                    {t.before ? <span className="text-muted-foreground">{t.before}</span> : null}
                    <ChunkWord
                      text={t.word}
                      chunk={chunks[t.chunk]!}
                      selected={r.selected[t.chunk]!}
                      jev={r.chunkPick(t.chunk)}
                      picks={r.wordPicks(t.chunk)}
                      onSelect={(k) => r.choose(t.chunk, chunks[t.chunk]!.readings[k]!.label)}
                    />
                    {t.after ? <span className="text-muted-foreground">{t.after}</span> : null}
                  </span>
                ))}
              </p>
            ))}
          </div>
        </div>
      ) : (
        <WordTable
          rows={chunks.flatMap((c, i) => {
            const picks = r.wordPicks(i);
            return (c.readings[r.selected[i]!]?.parts ?? []).map((p, k) => ({
              chunk: c,
              part: p,
              pick: picks?.[k],
              first: k === 0
            }));
          })}
        />
      )}
      {r.jevFailed ? (
        <p className="text-xs text-muted-foreground">
          Jev did not answer, so the words show the grammar’s and cheda’s likeliest readings.
        </p>
      ) : null}
      <p className="text-xs text-muted-foreground">
        Want to take a word apart by hand, at the sound you choose?{' '}
        <Link
          to="/vidyut/sandhi"
          search={{ tab: 'split', q }}
          className="font-medium text-primary underline-offset-4 hover:underline"
        >
          Sandhi split
        </Link>
      </p>
    </section>
  );
}

/**
 * A written word of the text, dotted-underlined. Its popover: how it splits (the chosen
 * reading's words, other readings below) → one word's analyses, derivation and forms.
 */
function ChunkWord({
  text,
  chunk,
  selected,
  jev,
  picks,
  onSelect
}: {
  text: string;
  chunk: ChunkSplit;
  selected: number;
  jev?: JevPick;
  picks?: (WordPick | undefined)[];
  onSelect: (i: number) => void;
}) {
  const reading = chunk.readings[selected];
  const parts = reading?.parts ?? [];
  // One word and one way to read it: open straight on the word.
  const direct = parts.length === 1 && chunk.readings.length === 1;
  const [word, setWord] = useState<number | null>(null);
  const known = parts.every((p) => p.known);
  const shown = parts.length === 1 ? parts[0]!.analyses[0] : undefined;
  return (
    <Popover onOpenChange={(o) => !o && setWord(null)}>
      <PopoverTrigger
        aria-label={`${text}: ${parts.map((p) => p.text).join(' + ')}. Show how it splits and what it is`}
        className={cn(
          'rounded-sm underline decoration-dotted decoration-[1.5px] underline-offset-[7px] transition-colors hover:bg-primary/8 hover:decoration-primary focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none data-popup-open:bg-primary/10 data-popup-open:decoration-primary',
          known
            ? 'decoration-muted-foreground/60'
            : 'text-muted-foreground decoration-muted-foreground/40',
          shown && KIND_UNDERLINE[shown.kind]
        )}
      >
        {text}
      </PopoverTrigger>
      <PopoverContent align="start" className="w-[22rem] max-w-[calc(100vw-1rem)] gap-2 p-2">
        {direct || word !== null ? (
          <WordDetails
            part={parts[word ?? 0]!}
            {...pickOf(picks?.[word ?? 0])}
            back={direct ? undefined : { label: chunk.chunk, onClick: () => setWord(null) }}
          />
        ) : (
          <>
            <div className="flex items-baseline justify-between gap-2 px-1.5 pt-1">
              <PopoverTitle className="font-devanagari text-2xl font-bold">
                {chunk.chunk}
              </PopoverTitle>
              {reading?.cheda ? <ChedaBadge /> : null}
            </div>
            <p className="px-1.5 text-[11px] text-muted-foreground">
              {parts.length > 1 ? `${parts.length} words` : 'One word'} · choose one for what it is
            </p>
            <ul className="flex flex-col gap-0.5">
              {parts.map((p, k) => {
                const { jevKey } = pickOf(picks?.[k]);
                const a = p.analyses.find((x) => x.key === jevKey) ?? p.analyses[0];
                return (
                  <li key={k}>
                    <button
                      type="button"
                      onClick={() => setWord(k)}
                      className="flex w-full items-center gap-2.5 rounded-lg px-2 py-1.5 text-left transition-colors hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
                    >
                      <span
                        className={cn(
                          'size-2 shrink-0 rounded-full',
                          a ? KIND[a.kind].dot : 'bg-muted-foreground/40'
                        )}
                        aria-hidden
                      />
                      <span
                        className={cn(
                          'font-devanagari text-lg font-semibold',
                          !p.known && 'text-muted-foreground'
                        )}
                      >
                        {p.text}
                      </span>
                      <span className="min-w-0 flex-1 truncate font-devanagari text-xs text-muted-foreground">
                        {a
                          ? `${a.lemma}${a.dhatu && a.krt ? ` ← ${a.dhatu} + ${a.krt}` : ''} · ${a.kind === 'avyaya' ? KIND.avyaya.label : padaTags(a)}`
                          : p.dictionary
                            ? 'in the dictionaries'
                            : 'unknown'}
                      </span>
                      <ChevronRightIcon
                        className="size-4 shrink-0 text-muted-foreground"
                        aria-hidden
                      />
                    </button>
                  </li>
                );
              })}
            </ul>
            {chunk.readings.length > 1 ? (
              <div className="flex flex-col gap-1 border-t pt-2">
                <p className="px-1.5 text-[11px] text-muted-foreground">
                  Ways to split it · {chunk.readings.length}
                </p>
                <ReadingChoices chunk={chunk} jev={jev} selected={selected} onSelect={onSelect} />
              </div>
            ) : null}
          </>
        )}
      </PopoverContent>
    </Popover>
  );
}

/** A single word's underline takes the colour of its class (verb, noun, indeclinable). */
const KIND_UNDERLINE = {
  tinanta: 'decoration-emerald-500/70',
  subanta: 'decoration-sky-500/70',
  avyaya: 'decoration-amber-500/70'
} as const;

/** The words as a gloss table: form, analysis, tags, and its derivation. */
function WordTable({
  rows
}: {
  rows: { chunk: ChunkSplit; part: Part; pick?: WordPick; first: boolean }[];
}) {
  const open = useOpenDerivation();
  return (
    <div className="overflow-x-auto rounded-2xl border bg-card shadow-sm">
      <table className="w-full text-sm">
        <thead className="border-b bg-muted/40 text-left text-[11px] tracking-wider text-muted-foreground uppercase">
          <tr>
            <th scope="col" className="px-3 py-2 font-semibold">
              Written
            </th>
            <th scope="col" className="px-3 py-2 font-semibold">
              Word
            </th>
            <th scope="col" className="px-3 py-2 font-semibold">
              Stem / root
            </th>
            <th scope="col" className="px-3 py-2 font-semibold">
              Grammar
            </th>
            <th scope="col" className="px-3 py-2 font-semibold">
              <span className="sr-only">Links</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.map(({ chunk, part, pick, first }, i) => {
            const { jevKey, jevP } = pickOf(pick);
            const a = part.analyses.find((x) => x.key === jevKey) ?? part.analyses[0];
            const kind = a ? KIND[a.kind] : null;
            const forms = a ? formsLink(a) : null;
            return (
              <tr key={i} className={cn('align-top', first && i > 0 && 'border-t')}>
                <td className="px-3 py-1.5 font-devanagari text-muted-foreground">
                  {first ? chunk.chunk : null}
                </td>
                <td
                  className={cn(
                    'px-3 py-1.5 font-devanagari text-base font-semibold',
                    !part.known && 'text-muted-foreground'
                  )}
                >
                  {part.text}
                </td>
                <td className="px-3 py-1.5 font-devanagari">
                  {a ? (
                    <span className="flex items-center gap-1.5">
                      <span
                        className={cn('size-1.5 shrink-0 rounded-full', kind!.dot)}
                        aria-hidden
                      />
                      {a.lemma}
                      {a.dhatu && a.krt ? (
                        <span className="text-muted-foreground">
                          ← {a.dhatu} + {a.krt}
                        </span>
                      ) : null}
                    </span>
                  ) : (
                    <span className="text-muted-foreground">
                      {part.dictionary ? 'in the dictionaries' : 'unknown'}
                    </span>
                  )}
                </td>
                <td className="px-3 py-1.5 font-devanagari text-muted-foreground">
                  {a ? (a.kind === 'avyaya' ? kind!.label : padaTags(a)) : null}
                  {jevP !== undefined && a?.key === jevKey ? (
                    <span className="ml-1.5 font-sans text-[11px] font-semibold text-violet-700 dark:text-violet-300">
                      Jev {Math.round(jevP * 100)}%
                    </span>
                  ) : null}
                </td>
                <td className="px-3 py-1.5">
                  <span className="flex justify-end gap-1">
                    {a?.word ? (
                      <Button
                        variant="ghost"
                        size="xs"
                        onClick={() =>
                          open({
                            spec: a.word!,
                            form: part.text,
                            context: `${a.lemma} · ${padaTags(a) || kind!.label}`
                          })
                        }
                      >
                        <GitBranchIcon aria-hidden /> Derivation
                      </Button>
                    ) : null}
                    {forms ? (
                      <Link
                        to={forms.to}
                        search={forms.search as never}
                        className={buttonVariants({ variant: 'ghost', size: 'xs' })}
                      >
                        <TableIcon aria-hidden /> {forms.label}
                      </Link>
                    ) : null}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
