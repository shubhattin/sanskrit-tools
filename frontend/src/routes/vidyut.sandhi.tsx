import { useDeferredValue, useEffect, useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { GitBranchIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { DevaInput, ErrorNote, LoadingBlock, Segmented } from '@/components/vidyut/controls';
import { useOpenDerivation } from '@/components/vidyut/derivation';
import { searchString } from '@/lib/search-params';
import { joinQuery, sandhiRulesQuery, splitQuery, type ChunkSplit } from '@/lib/vidyut/api';
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
          Join <span className="font-devanagari text-xs opacity-70">सन्धिः</span>
        </TabsTrigger>
        <TabsTrigger value="split">
          Split <span className="font-devanagari text-xs opacity-70">विच्छेदः</span>
        </TabsTrigger>
        <TabsTrigger value="rules">
          Rules <span className="font-devanagari text-xs opacity-70">नियमाः</span>
        </TabsTrigger>
      </TabsList>
      {tab !== 'rules' ? (
        <DevaInput
          value={text}
          onChange={setText}
          label={tab === 'join' ? 'Words to join' : 'Text to split'}
          placeholder={
            tab === 'join' ? 'Words, separated by spaces: रामः अस्ति' : 'A joined word: तथैव, देवेन्द्रः'
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
  const open = useOpenDerivation();
  if (!q) return <Hint>Type two or more words. The grammar's sandhi rules join them.</Hint>;
  if (!enabled) return <Hint>Add another word.</Hint>;
  if (res.isPending) return <LoadingBlock rows={1} label="Joining" />;
  if (res.isError) return <ErrorNote error={res.error} />;
  return (
    <div className="flex max-w-xl flex-col gap-2 rounded-xl border bg-card p-4">
      <p className="font-devanagari text-sm text-muted-foreground">{words.join(' + ')}</p>
      {res.data.map((r) => (
        <div key={r} className="flex flex-wrap items-center gap-3">
          <span className="font-devanagari text-2xl font-semibold">{r}</span>
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
    </div>
  );
}

function Hint({ children }: { children: React.ReactNode }) {
  return <p className="text-sm text-muted-foreground">{children}</p>;
}

function Part({ text, known }: { text: string; known: boolean }) {
  return known ? (
    <Link
      to="/"
      search={{ q: text }}
      className="font-devanagari font-semibold text-primary hover:underline"
      title="In the dictionaries — look it up"
    >
      {text}
    </Link>
  ) : (
    <span className="font-devanagari text-muted-foreground">{text}</span>
  );
}

function ChunkList({ chunk }: { chunk: ChunkSplit }) {
  const [all, setAll] = useState(false);
  const likely = chunk.splits.filter((s) => s.left_known && s.right_known);
  const shown = all ? chunk.splits : likely.length ? likely : chunk.splits.slice(0, 8);
  return (
    <section className="rounded-xl border bg-card" aria-label={chunk.chunk}>
      <h3 className="border-b px-3 py-2 font-devanagari text-lg font-semibold">{chunk.chunk}</h3>
      {chunk.splits.length === 0 ? (
        <p className="px-3 py-2 text-sm text-muted-foreground">No sandhi to undo here.</p>
      ) : (
        <ul className="grid grid-cols-1 gap-x-6 px-3 py-1 sm:grid-cols-2 lg:grid-cols-3">
          {shown.map((s) => (
            <li
              key={`${s.left}+${s.right}`}
              className={cn(
                'flex items-baseline gap-2 py-1 text-base',
                s.left_known && s.right_known && 'font-medium'
              )}
            >
              <Part text={s.left} known={s.left_known} />
              <span className="text-muted-foreground">+</span>
              <Part text={s.right} known={s.right_known} />
            </li>
          ))}
        </ul>
      )}
      {chunk.splits.length > shown.length || all ? (
        <Button variant="ghost" size="sm" className="m-1" onClick={() => setAll((v) => !v)}>
          {all ? 'Likely splits only' : `All ${chunk.splits.length} candidates`}
        </Button>
      ) : null}
    </section>
  );
}

function Split({ q }: { q: string }) {
  const res = useQuery({ ...splitQuery(q), enabled: !!q });
  if (!q)
    return (
      <Hint>
        Shows every way a joined word could have come from two words. Parts found in the
        dictionaries are highlighted; splits where both are found come first.
      </Hint>
    );
  if (res.isPending) return <LoadingBlock rows={2} label="Splitting" />;
  if (res.isError) return <ErrorNote error={res.error} />;
  return (
    <div className="flex flex-col gap-3">
      {res.data.map((c, i) => (
        <ChunkList key={`${c.chunk}-${i}`} chunk={c} />
      ))}
      <p className="text-xs text-muted-foreground">
        Purely phonetic: the second part is the rest of the word, which may itself hold more words.
      </p>
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
