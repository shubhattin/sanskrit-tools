import { useDeferredValue, useEffect, useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { ChevronRightIcon, GitBranchIcon, XIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import { WordCard } from '@/components/vidyut/words';
import {
  ChedaBadge,
  Legend,
  ReadingChoices,
  pickOf,
  useReading
} from '@/components/vidyut/reading';
import {
  DevaInput,
  ErrorNote,
  LoadingBlock,
  SectionLabel,
  Segmented
} from '@/components/vidyut/controls';
import { JevBadge, JevPending, isJevChoice, jevProbability } from '@/components/vidyut/jev';
import { useOpenDerivation } from '@/components/vidyut/derivation';
import { searchString } from '@/lib/search-params';
import {
  jevJoinQuery,
  joinQuery,
  sandhiRulesQuery,
  splitAtQuery,
  type ChunkSplit,
  type JevPick,
  type Reading,
  type WordPick
} from '@/lib/vidyut/api';
import { cn } from '@/lib/utils';

type Tab = 'join' | 'split' | 'rules';

interface SandhiSearch {
  tab?: Tab;
  q?: string;
}

export const Route = createFileRoute('/vidyut/sandhi')({
  validateSearch: (s: Record<string, unknown>): SandhiSearch => ({
    tab: s.tab === 'split' || s.tab === 'rules' ? s.tab : undefined,
    q: searchString(s.q)
  }),
  component: SandhiTool
});

function SandhiTool() {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: Route.fullPath });
  const set = (patch: Partial<SandhiSearch>, replace = false) =>
    void navigate({ search: (prev) => ({ ...prev, ...patch }), replace });
  const tab = search.tab ?? 'join';

  // Join and Split share the text, so a joined phrase can be split back and vice versa.
  const [text, setText] = useState(search.q ?? '');
  const [synced, setSynced] = useState(search.q);
  if (synced !== search.q) {
    setSynced(search.q);
    setText(search.q ?? '');
  }
  useEffect(() => {
    const t = setTimeout(() => {
      const q = text.trim() || undefined;
      if (q !== search.q) set({ q }, true);
    }, 400);
    return () => clearTimeout(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [text]);
  const q = (search.q ?? '').trim();

  return (
    <Tabs value={tab} onValueChange={(t) => set({ tab: t === 'join' ? undefined : (t as Tab) })}>
      <TabsList>
        <TabsTrigger value="join">
          Join <span className="font-devanagari text-xs">सन्धिः</span>
        </TabsTrigger>
        <TabsTrigger value="split">
          Split <span className="font-devanagari text-xs">विच्छेदः</span>
        </TabsTrigger>
        <TabsTrigger value="rules">
          Rules <span className="font-devanagari text-xs">नियमाः</span>
        </TabsTrigger>
      </TabsList>
      {tab !== 'rules' ? (
        <DevaInput
          value={text}
          onChange={setText}
          label={tab === 'join' ? 'Words to join' : 'Text to split'}
          placeholder={
            tab === 'join'
              ? 'Words, separated by spaces: रामः अस्ति'
              : 'Joined text: तथैव, धर्मक्षेत्रे कुरुक्षेत्रे'
          }
          className="mt-2 max-w-xl"
        />
      ) : null}
      <TabsContent value="join" className="pt-2">
        <Join q={q} />
      </TabsContent>
      <TabsContent value="split" className="pt-2">
        <Split q={q} />
      </TabsContent>
      <TabsContent value="rules" className="pt-2">
        <Rules />
      </TabsContent>
    </Tabs>
  );
}

function Join({ q }: { q: string }) {
  const words = q.split(/\s+/).filter(Boolean);
  const enabled = words.length >= 2;
  const res = useQuery({ ...joinQuery(words), enabled });
  const jev = useQuery({ ...jevJoinQuery(words), enabled: (res.data?.length ?? 0) > 1 });
  const open = useOpenDerivation();
  if (!q) return <Hint>Type two or more words. The grammar's sandhi rules join them.</Hint>;
  if (!enabled) return <Hint>Add another word.</Hint>;
  if (res.isPending) return <LoadingBlock rows={1} label="Joining" />;
  if (res.isError) return <ErrorNote error={res.error} />;
  return (
    <div className="flex max-w-xl flex-col gap-2 rounded-2xl border bg-card p-4 shadow-sm">
      <p className="font-devanagari text-sm text-muted-foreground">{words.join(' + ')}</p>
      {res.data.map((r) => (
        <div key={r} className="flex flex-wrap items-center gap-3">
          <span
            className={cn(
              'font-devanagari text-2xl font-semibold',
              res.data.length > 1 &&
                jev.data &&
                !isJevChoice(jev.data, r) &&
                'text-muted-foreground'
            )}
          >
            {r}
          </span>
          {isJevChoice(jev.data, r) ? <JevBadge p={jevProbability(jev.data, r)} /> : null}
          <Button
            variant="ghost"
            size="sm"
            onClick={() =>
              open({ spec: { kind: 'vakya', words }, form: r, context: words.join(' + ') })
            }
          >
            <GitBranchIcon aria-hidden /> How?
          </Button>
        </div>
      ))}
      {res.data.length > 1 && jev.isFetching ? <JevPending what="the spellings" /> : null}
    </div>
  );
}

function Hint({ children }: { children: React.ReactNode }) {
  return <p className="text-sm text-muted-foreground">{children}</p>;
}

/** A two-word candidate's part: dictionary / grammar words link to the dictionary. */
function PartText({ text, known }: { text: string; known: boolean }) {
  return known ? (
    <Link
      to="/"
      search={{ q: text }}
      className="font-devanagari font-semibold text-primary decoration-primary/40 underline-offset-4 hover:underline"
    >
      {text}
    </Link>
  ) : (
    <span className="font-devanagari text-muted-foreground">{text}</span>
  );
}

/** A reading's words as word cards, joined by +. */
function ReadingWords({
  reading,
  picks
}: {
  reading: Reading;
  /** Jev's analysis pick per word (same order). */
  picks?: (WordPick | undefined)[];
}) {
  return (
    <ol className="flex flex-wrap items-center gap-x-1.5 gap-y-2" aria-label="Words">
      {reading.parts.map((p, i) => {
        return (
          <li key={i} className="flex items-center gap-1.5">
            {i > 0 ? (
              <span className="text-lg text-muted-foreground" aria-hidden>
                +
              </span>
            ) : null}
            <WordCard part={p} {...pickOf(picks?.[i])} />
          </li>
        );
      })}
    </ol>
  );
}

/** The chunk's sounds, grouped by akṣara; picking one pins the junction there. */
function VarnaStrip({
  chunk,
  pinned,
  onPin
}: {
  chunk: ChunkSplit;
  pinned: number | null;
  onPin: (at: number | null) => void;
}) {
  return (
    <div className="flex flex-wrap items-center gap-1" role="group" aria-label="Sounds of the word">
      {chunk.aksharas.map((a, i) => (
        <span key={i} className="inline-flex rounded-lg border bg-muted/40 p-0.5">
          {a.varnas.map((v) => {
            const on = v.at === pinned;
            return (
              <button
                key={v.at}
                type="button"
                aria-pressed={on}
                aria-label={`Split where ${v.text} is`}
                onClick={() => onPin(on ? null : v.at)}
                className={cn(
                  'min-w-7 rounded-md px-1 py-0.5 font-devanagari text-base transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
                  on
                    ? 'bg-amber-500 text-white shadow-sm'
                    : 'hover:bg-amber-500/15 hover:text-amber-700 dark:hover:text-amber-300'
                )}
              >
                {v.text}
              </button>
            );
          })}
        </span>
      ))}
      {pinned !== null ? (
        <Button variant="ghost" size="xs" onClick={() => onPin(null)}>
          <XIcon aria-hidden /> Clear
        </Button>
      ) : null}
    </div>
  );
}

function PinnedResults({ chunk, at }: { chunk: string; at: number }) {
  const q = useQuery(splitAtQuery(chunk, at));
  if (q.isPending) return <LoadingBlock rows={1} label="Splitting there" />;
  if (q.isError) return <ErrorNote error={q.error} />;
  if (q.data.length === 0)
    return <Hint>No sandhi gives this sound. Try the sound where the two words meet.</Hint>;
  return (
    <ul className="flex flex-col gap-2">
      {q.data.slice(0, 6).map((s, i) => (
        <li key={i} className="rounded-xl bg-amber-500/5 p-2 ring-1 ring-amber-500/15">
          <ReadingWords reading={{ ...s.right, parts: [s.left, ...s.right.parts] }} />
        </li>
      ))}
    </ul>
  );
}

function ChunkCard({
  chunk,
  jev,
  jevLoading,
  selected,
  onSelect,
  picks,
  picksLoading
}: {
  chunk: ChunkSplit;
  jev?: JevPick;
  jevLoading: boolean;
  selected: number;
  onSelect: (i: number) => void;
  picks?: (WordPick | undefined)[];
  picksLoading: boolean;
}) {
  const [pinned, setPinned] = useState<number | null>(null);
  const [allPairs, setAllPairs] = useState(false);
  const pairs = allPairs ? chunk.splits : chunk.splits.slice(0, 12);
  const reading = chunk.readings[selected];
  const many = chunk.readings.length > 1;
  return (
    <section
      className="overflow-hidden rounded-2xl border bg-card shadow-sm"
      aria-label={`Split of ${chunk.chunk}`}
    >
      <header className="flex flex-wrap items-center gap-x-3 gap-y-1 border-b bg-gradient-to-r from-amber-500/10 via-transparent to-violet-500/10 px-4 py-2.5">
        <h2 className="font-devanagari text-xl font-bold">{chunk.chunk}</h2>
        {reading ? (
          <span className="flex items-center gap-1.5">
            {isJevChoice(jev, reading.label) ? (
              <JevBadge p={jevProbability(jev, reading.label)} />
            ) : null}
            {reading.cheda ? <ChedaBadge /> : null}
          </span>
        ) : null}
        <span className="ml-auto" aria-live="polite">
          {jevLoading ? (
            <JevPending what="the readings" />
          ) : picksLoading ? (
            <JevPending what="what each word is" />
          ) : null}
        </span>
      </header>

      <div className="flex flex-col gap-5 p-4">
        {reading ? <ReadingWords reading={reading} picks={picks} /> : null}

        {many ? (
          <div className="flex flex-col gap-1.5">
            <SectionLabel>Readings · {chunk.readings.length}</SectionLabel>
            <ReadingChoices chunk={chunk} jev={jev} selected={selected} onSelect={onSelect} />
          </div>
        ) : null}

        <Collapsible className="flex flex-col gap-2">
          <CollapsibleTrigger className="group flex items-center gap-1.5 self-start rounded-md text-xs font-semibold tracking-wide text-muted-foreground uppercase hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none">
            <ChevronRightIcon
              className="size-3.5 transition-transform group-data-panel-open:rotate-90"
              aria-hidden
            />
            Split it yourself
          </CollapsibleTrigger>
          <CollapsibleContent className="flex flex-col gap-4 pl-5">
            <div className="flex flex-col gap-1.5">
              <p className="text-xs text-muted-foreground">
                Pick the sound where two words meet (ऐ in तथैव, ओ in रामोऽस्ति): the splits there,
                with the rest read as words.
              </p>
              <VarnaStrip chunk={chunk} pinned={pinned} onPin={setPinned} />
              {pinned !== null ? <PinnedResults chunk={chunk.chunk} at={pinned} /> : null}
            </div>
            {chunk.splits.length > 0 ? (
              <div className="flex flex-col gap-1.5">
                <SectionLabel>Every two-word split · {chunk.splits.length}</SectionLabel>
                <ul className="grid grid-cols-1 gap-x-6 sm:grid-cols-2 lg:grid-cols-3">
                  {pairs.map((s) => (
                    <li key={`${s.left}+${s.right}`} className="flex items-baseline gap-1.5 py-0.5">
                      <PartText text={s.left} known={s.left_known} />
                      <span className="text-muted-foreground/70">+</span>
                      <PartText text={s.right} known={s.right_known} />
                    </li>
                  ))}
                </ul>
                {chunk.splits.length > 12 ? (
                  <Button
                    variant="ghost"
                    size="sm"
                    className="self-start"
                    onClick={() => setAllPairs((v) => !v)}
                  >
                    {allPairs ? 'Fewer' : `All ${chunk.splits.length}`}
                  </Button>
                ) : null}
              </div>
            ) : null}
          </CollapsibleContent>
        </Collapsible>
      </div>
    </section>
  );
}

function Split({ q }: { q: string }) {
  const r = useReading(q);
  if (!q)
    return (
      <Hint>
        Takes joined text apart: every way sandhi could have joined its words, checked against the
        dictionaries and every form Pāṇini’s grammar generates, then weighed by cheda and Jev. To
        read a whole verse word by word, use{' '}
        <Link
          to="/vidyut/cheda"
          className="font-medium text-primary underline-offset-4 hover:underline"
        >
          Cheda
        </Link>
        .
      </Hint>
    );
  if (r.split.isPending) return <LoadingBlock rows={3} label="Splitting" />;
  if (r.split.isError) return <ErrorNote error={r.split.error} />;
  return (
    <div className="flex flex-col gap-3">
      <Legend />
      {r.chunks.map((c, i) => (
        <ChunkCard
          key={`${c.chunk}-${i}`}
          chunk={c}
          jev={r.chunkPick(i)}
          jevLoading={r.readingsPending && c.readings.length > 1}
          selected={r.selected[i]!}
          onSelect={(k) => r.choose(i, c.readings[k]!.label)}
          picks={r.wordPicks(i)}
          picksLoading={r.wordsPending}
        />
      ))}
    </div>
  );
}

type RuleField = 'result' | 'first' | 'second';

function Rules() {
  const q = useQuery(sandhiRulesQuery());
  const [by, setBy] = useState<RuleField>('result');
  const [filter, setFilter] = useState('');
  const f = useDeferredValue(filter.trim());
  if (q.isPending) return <LoadingBlock rows={6} label="Loading rules" />;
  if (q.isError) return <ErrorNote error={q.error} />;
  const rows = f
    ? q.data.filter((r) =>
        by === 'result' ? r.result.includes(f) : by === 'first' ? r.first === f : r.second === f
      )
    : q.data;
  const shown = rows.slice(0, 300);
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <Segmented
          size="sm"
          label="Filter by"
          value={by}
          onChange={setBy}
          options={[
            { id: 'result', label: 'Result has' },
            { id: 'first', label: 'First ends in' },
            { id: 'second', label: 'Second starts with' }
          ]}
        />
        <DevaInput
          value={filter}
          onChange={setFilter}
          label="Sound"
          placeholder={by === 'result' ? 'ए, ओ, च्च…' : by === 'first' ? 'अ, स्, त्…' : 'इ, च, ग…'}
          className="max-w-56"
        />
        <span className="text-xs text-muted-foreground tabular-nums">
          {rows.length} of {q.data.length} rules
        </span>
      </div>
      <div className="grid grid-cols-2 gap-x-6 rounded-xl border bg-card px-3 py-1 sm:grid-cols-3 lg:grid-cols-4">
        {shown.map((r, i) => (
          <p
            key={i}
            className="flex items-baseline gap-1.5 border-b py-1 font-devanagari text-base"
          >
            <span>{r.first}</span>
            <span className="text-muted-foreground">+</span>
            <span>{r.second}</span>
            <span className="text-muted-foreground">→</span>
            <span className="font-semibold text-primary">{r.result}</span>
          </p>
        ))}
      </div>
      {rows.length > shown.length ? (
        <p className="text-xs text-muted-foreground">
          Showing the first {shown.length}; filter to narrow down.
        </p>
      ) : null}
    </div>
  );
}
