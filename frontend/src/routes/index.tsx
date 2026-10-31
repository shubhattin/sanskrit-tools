import { useMemo } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { BookOpenTextIcon, SearchXIcon } from 'lucide-react';
import { dictionariesQuery, searchQuery } from '@/lib/api/client';
import type { SearchMode } from '@/lib/api/types';
import { SearchBar } from '@/components/dict/search-bar';
import { DictionaryGroupCard } from '@/components/dict/entry';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import { useDisplayText } from '@/components/script/display-script';

interface SearchSearch {
  q?: string;
  mode?: SearchMode;
  dicts?: string;
}

export const Route = createFileRoute('/')({
  validateSearch: (search: Record<string, unknown>): SearchSearch => ({
    q: typeof search.q === 'string' ? search.q : undefined,
    mode: search.mode === 'en' ? 'en' : search.mode === 'sa' ? 'sa' : undefined,
    dicts: typeof search.dicts === 'string' ? search.dicts : undefined
  }),
  component: SearchPage
});

const EXAMPLES: { label: string; q: string; mode: SearchMode }[] = [
  { label: 'अग्नि', q: 'अग्नि', mode: 'sa' },
  { label: 'agni', q: 'agni', mode: 'sa' },
  { label: 'dharma', q: 'dharma', mode: 'sa' },
  { label: 'bhavati', q: 'bhavati', mode: 'sa' },
  { label: 'fire (English)', q: 'fire', mode: 'en' }
];

function SearchPage() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const q = (search.q ?? '').trim();
  const mode: SearchMode = search.mode ?? 'sa';
  const dictSlugs = useMemo(
    () =>
      (search.dicts ?? '')
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean),
    [search.dicts]
  );

  const dictsQuery = useQuery(dictionariesQuery());
  const result = useQuery({
    ...searchQuery({ q, mode, dicts: search.dicts || undefined }),
    enabled: q.length > 0
  });

  const go = (next: { q: string; mode: SearchMode; dicts: string[] }) =>
    void navigate({
      to: '/',
      search: {
        q: next.q || undefined,
        mode: next.mode === 'sa' ? undefined : next.mode,
        dicts: next.dicts.length > 0 ? next.dicts.join(',') : undefined
      }
    });

  const totalHits = (result.data?.groups ?? []).reduce((n, g) => n + g.total, 0);

  return (
    <div className="flex flex-col gap-5">
      <SearchBar
        value={{ q: search.q ?? '', mode, dicts: dictSlugs }}
        dictionaries={dictsQuery.data ?? []}
        autoFocus={q.length === 0}
        onSubmit={go}
      />

      {q.length === 0 ? (
        <Landing dictionariesLoading={dictsQuery.isPending} />
      ) : result.isPending ? (
        <ResultsSkeleton />
      ) : result.isError ? (
        <QueryError message={result.error.message} onRetry={() => void result.refetch()} />
      ) : result.data ? (
        <Results
          q={q}
          mode={mode}
          totalHits={totalHits}
          data={result.data}
          onExample={(chip) => go({ q: chip, mode: 'sa', dicts: dictSlugs })}
        />
      ) : null}
    </div>
  );
}

function Landing({ dictionariesLoading }: { dictionariesLoading: boolean }) {
  const dictsQuery = useQuery(dictionariesQuery());
  const navigate = useNavigate();
  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-2 py-4 text-center sm:py-8">
        <p className="font-devanagari text-4xl font-bold sm:text-5xl">शब्दकोशः</p>
        <p className="mx-auto max-w-xl text-sm text-muted-foreground sm:text-base">
          Search across Sanskrit lexicons, thesauri, dhātu works and sūtra commentaries. Type in
          Devanagari, IAST, Harvard-Kyoto, ITRANS or SLP1 — or enable Lipi Lekhika typing and type
          phonetically.
        </p>
        <div className="mt-2 flex flex-wrap items-center justify-center gap-2">
          {EXAMPLES.map((ex) => (
            <Button
              key={ex.label}
              variant="outline"
              size="sm"
              className="font-devanagari"
              onClick={() =>
                void navigate({
                  to: '/',
                  search: {
                    q: ex.q,
                    mode: ex.mode === 'sa' ? undefined : ex.mode
                  }
                })
              }
            >
              {ex.label}
            </Button>
          ))}
        </div>
      </div>
      <section aria-label="Dictionaries" className="flex flex-col gap-3">
        <h2 className="text-sm font-semibold tracking-wide text-muted-foreground uppercase">
          Dictionaries
        </h2>
        {dictionariesLoading || !dictsQuery.data ? (
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-24" />
            ))}
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {dictsQuery.data.map((d) => (
              <Link
                key={d.slug}
                to="/dict/$slug"
                params={{ slug: d.slug }}
                search={{}}
                className="block"
              >
                <Card className="h-full transition-colors hover:border-primary/50">
                  <CardContent className="flex flex-col gap-1 p-4">
                    <div className="flex items-center gap-2">
                      <Badge variant="secondary" className="font-bold">
                        {d.short_name}
                      </Badge>
                      <span className="text-xs text-muted-foreground tabular-nums">
                        {d.entry_count.toLocaleString('en-IN')} entries
                      </span>
                    </div>
                    <p className="truncate text-sm font-semibold">{d.name}</p>
                    {d.description ? (
                      <p className="line-clamp-2 text-xs text-muted-foreground">{d.description}</p>
                    ) : null}
                  </CardContent>
                </Card>
              </Link>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function ResultsSkeleton() {
  return (
    <div className="flex flex-col gap-4" aria-label="Loading results">
      <Skeleton className="h-5 w-48" />
      <Skeleton className="h-56" />
      <Skeleton className="h-56" />
    </div>
  );
}

function QueryError({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyMedia variant="icon">
          <SearchXIcon aria-hidden />
        </EmptyMedia>
        <EmptyTitle>Search failed</EmptyTitle>
        <EmptyDescription>{message}</EmptyDescription>
      </EmptyHeader>
      <Button variant="outline" size="sm" onClick={onRetry}>
        Retry
      </Button>
    </Empty>
  );
}

function Results({
  q,
  mode,
  totalHits,
  data,
  onExample
}: {
  q: string;
  mode: SearchMode;
  totalHits: number;
  data: import('@/lib/api/types').SearchResponse;
  onExample: (chip: string) => void;
}) {
  const nothingFound = data.groups.length === 0 && data.forms.length === 0;
  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-muted-foreground" role="status">
        {totalHits > 0 ? (
          <>
            <strong className="text-foreground tabular-nums">{totalHits}</strong>{' '}
            {totalHits === 1 ? 'hit' : 'hits'} for{' '}
            <span className="font-devanagari font-semibold text-foreground">{q}</span>
            {mode === 'en' ? ' in English' : null}
            {data.slp1.length > 0 ? (
              <span className="ml-1 text-xs">(SLP1: {data.slp1.join(', ')})</span>
            ) : null}
          </>
        ) : (
          <>
            No results for{' '}
            <span className="font-devanagari font-semibold text-foreground">{q}</span>
          </>
        )}
      </p>

      {data.forms.length > 0 ? <FormsCard forms={data.forms} /> : null}

      {nothingFound ? (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <BookOpenTextIcon aria-hidden />
            </EmptyMedia>
            <EmptyTitle>No entries found</EmptyTitle>
            <EmptyDescription>
              Try a different spelling or transliteration (Devanagari, IAST, SLP1 all work), or
              search in English mode.
            </EmptyDescription>
          </EmptyHeader>
          {data.did_you_mean.length > 0 ? (
            <div className="flex flex-wrap items-center justify-center gap-2">
              <span className="text-sm text-muted-foreground">Did you mean:</span>
              {data.did_you_mean.map((chip) => (
                <DidYouMeanChip key={chip} chip={chip} onSelect={onExample} />
              ))}
            </div>
          ) : null}
        </Empty>
      ) : (
        <div className="flex gap-5">
          {data.groups.length > 1 ? <HitSidebar groups={data.groups} /> : null}
          <div className="flex min-w-0 flex-1 flex-col gap-5">
            {data.groups.map((g) => (
              <DictionaryGroupCard
                key={g.dictionary.slug}
                dictionary={g.dictionary}
                entries={g.entries}
                total={g.total}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function FormsCard({ forms }: { forms: import('@/lib/api/types').FormAnalysis[] }) {
  return (
    <Card>
      <CardContent className="flex flex-col gap-2 p-4">
        <h2 className="text-sm font-semibold tracking-wide text-muted-foreground uppercase">
          Inflected form → lemma
        </h2>
        <ul className="flex flex-col gap-1.5">
          {forms.map((f, i) => (
            <li
              key={`${f.form}-${f.lemma}-${i}`}
              className="flex flex-wrap items-baseline gap-x-2 text-sm"
            >
              <span className="font-devanagari text-base font-semibold">{f.form}</span>
              <span aria-hidden className="text-muted-foreground">
                →
              </span>
              <FormLemma lemma={f.lemma} />
              <span className="text-xs text-muted-foreground">{f.analysis}</span>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}

function FormLemma({ lemma }: { lemma: string }) {
  const display = useDisplayText(lemma);
  const navigate = useNavigate();
  return (
    <button
      type="button"
      className="font-devanagari text-base font-semibold text-primary underline underline-offset-2"
      onClick={() => void navigate({ to: '/', search: { q: lemma, mode: 'sa' } })}
    >
      {display}
    </button>
  );
}

function DidYouMeanChip({ chip, onSelect }: { chip: string; onSelect: (c: string) => void }) {
  const display = useDisplayText(chip);
  return (
    <Button variant="outline" size="sm" className="font-devanagari" onClick={() => onSelect(chip)}>
      {display}
    </Button>
  );
}

function HitSidebar({ groups }: { groups: import('@/lib/api/types').DictionaryHits[] }) {
  return (
    <aside aria-label="Dictionaries with hits" className="hidden w-52 shrink-0 lg:block">
      <nav className="sticky top-20 flex flex-col gap-1 rounded-xl border p-2">
        {groups.map((g) => (
          <a
            key={g.dictionary.slug}
            href={`#dict-${g.dictionary.slug}`}
            className="flex items-center justify-between gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-muted"
          >
            <span className="min-w-0 truncate">
              <span className="mr-1.5 font-bold text-muted-foreground">
                {g.dictionary.short_name}
              </span>
              <span className="sr-only">{g.dictionary.name}</span>
            </span>
            <span className="text-xs text-muted-foreground tabular-nums">{g.total}</span>
          </a>
        ))}
      </nav>
    </aside>
  );
}
