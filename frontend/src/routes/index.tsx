import { useEffect, useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import {
  ArrowRightIcon,
  BookOpenTextIcon,
  CheckIcon,
  LibraryBigIcon,
  PlusIcon,
  SearchXIcon,
  SparklesIcon,
  TableIcon
} from 'lucide-react';
import { searchString } from '@/lib/search-params';
import { dictionariesQuery, searchQuery } from '@/lib/api/client';
import type {
  Dictionary,
  DictionaryHits,
  FormAnalysis,
  SearchMode,
  SearchResponse
} from '@/lib/api/types';
import { DICT_KIND_LABELS, DICT_KIND_ORDER } from '@/lib/api/types';
import {
  resolveSelectedSlugs,
  setSearchPrefs,
  useSearchPrefs,
  type SearchPrefs
} from '@/lib/search-prefs';
import { SearchBar } from '@/components/dict/search-bar';
import { DictionaryGroupCard } from '@/components/dict/entry';
import { FormParadigmDialog, ParadigmSourceBadge } from '@/components/dict/paradigm';
import { DirectionTag, groupByKind } from '@/components/dict/filter';
import { KIND_ACCENT } from '@/components/dict/kind';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import { cn } from '@/lib/utils';

interface SearchSearch {
  q?: string;
  mode?: SearchMode;
}

export const Route = createFileRoute('/')({
  validateSearch: (search: Record<string, unknown>): SearchSearch => ({
    q: searchString(search.q),
    mode: search.mode === 'en' ? 'en' : search.mode === 'sa' ? 'sa' : undefined
  }),
  component: SearchPage
});

function SearchPage() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const q = (search.q ?? '').trim();
  const mode: SearchMode = search.mode ?? 'sa';
  const prefs = useSearchPrefs();

  const dictsQuery = useQuery(dictionariesQuery());
  const dictionaries = dictsQuery.data ?? [];
  // Until the list loads, send the stored slugs as is (the API ignores unknown ones).
  const selected = dictsQuery.data
    ? resolveSelectedSlugs(prefs, dictsQuery.data)
    : prefs.dicts === 'all'
      ? []
      : prefs.dicts;
  const isAll =
    prefs.dicts === 'all' || (dictionaries.length > 0 && selected.length === dictionaries.length);

  const result = useQuery({
    ...searchQuery({
      q,
      mode,
      dicts: isAll ? undefined : selected.join(','),
      forms: prefs.forms
    }),
    enabled: q.length > 0
  });

  const go = (next: { q: string; mode: SearchMode }) =>
    void navigate({
      to: '/',
      search: { q: next.q || undefined, mode: next.mode === 'sa' ? undefined : next.mode }
    });

  const bar = (
    <SearchBar
      value={{ q: search.q ?? '', mode }}
      dictionaries={dictionaries}
      selected={selected}
      prefs={prefs}
      autoFocus={q.length === 0}
      onSubmit={go}
    />
  );

  if (q.length === 0) {
    return <Landing bar={bar} dictionaries={dictsQuery.data} selected={selected} isAll={isAll} />;
  }

  return (
    <div className="grid gap-5 lg:grid-cols-[15rem_minmax(0,1fr)] lg:gap-8">
      <div className="relative z-[35] lg:col-start-2">{bar}</div>
      {result.isPending ? (
        <ResultsSkeleton />
      ) : result.isError ? (
        <div className="lg:col-start-2">
          <QueryError message={result.error.message} onRetry={() => void result.refetch()} />
        </div>
      ) : (
        <Results
          q={q}
          mode={mode}
          data={result.data}
          prefs={prefs}
          isAll={isAll}
          onPick={(chip) => go({ q: chip, mode: 'sa' })}
        />
      )}
    </div>
  );
}

/* ───────────────────────────── Landing ───────────────────────────── */

function Landing({
  bar,
  dictionaries,
  selected,
  isAll
}: {
  bar: React.ReactNode;
  dictionaries: Dictionary[] | undefined;
  selected: string[];
  isAll: boolean;
}) {
  return (
    <div className="flex flex-col gap-12 sm:gap-16">
      <section className="flex flex-col items-center gap-6 pt-6 text-center sm:pt-14">
        <div className="flex animate-in flex-col items-center gap-3 duration-700 fade-in-0 slide-in-from-bottom-3">
          <h1 className="hero-title font-devanagari text-5xl leading-[1.25] font-bold tracking-tight sm:text-7xl">
            शब्दकोशः
          </h1>
          <p className="max-w-xl text-sm leading-relaxed text-pretty text-muted-foreground sm:text-base">
            Search Sanskrit lexicons, thesauri, dhātu works and sūtra commentaries.
          </p>
        </div>
        <div className="relative z-[35] w-full max-w-3xl animate-in text-left delay-150 duration-700 fade-in-0 fill-mode-both slide-in-from-bottom-4">
          {bar}
        </div>
      </section>

      <section aria-labelledby="dicts-heading" className="flex flex-col gap-8">
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2 id="dicts-heading" className="flex items-center gap-2 text-lg font-semibold">
              <LibraryBigIcon className="size-5 text-primary" aria-hidden /> Library
            </h2>
            <p className="text-sm text-muted-foreground">
              {dictionaries
                ? isAll
                  ? 'Searching every work.'
                  : `Searching ${selected.length} of ${dictionaries.length} works — add more below or from “Dictionaries”.`
                : 'Loading…'}
            </p>
          </div>
        </div>
        {!dictionaries ? (
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {Array.from({ length: 8 }).map((_, i) => (
              <Skeleton key={i} className="h-36 rounded-2xl" />
            ))}
          </div>
        ) : (
          <DictionaryCatalog dictionaries={dictionaries} selected={selected} />
        )}
      </section>
    </div>
  );
}

function DictionaryCatalog({
  dictionaries,
  selected
}: {
  dictionaries: Dictionary[];
  selected: string[];
}) {
  const byKind = groupByKind(dictionaries);
  const selectedSet = new Set(selected);
  const toggle = (slug: string) => {
    const next = selectedSet.has(slug)
      ? selected.filter((s) => s !== slug)
      : dictionaries.filter((d) => d.slug === slug || selectedSet.has(d.slug)).map((d) => d.slug);
    if (next.length === 0) return;
    setSearchPrefs({ dicts: next.length === dictionaries.length ? 'all' : next });
  };
  // Global numbering in catalogue order (kind, then sort order).
  const ordinal = new Map(
    DICT_KIND_ORDER.flatMap((k) => byKind.get(k) ?? []).map((d, i) => [d.slug, i + 1])
  );
  return (
    <div className="flex flex-col gap-10">
      {DICT_KIND_ORDER.filter((k) => byKind.has(k)).map((kind) => {
        const group = byKind.get(kind)!;
        return (
          <div key={kind} className="flex flex-col gap-3">
            <h3 className="flex items-center gap-2 text-xs font-semibold tracking-widest text-muted-foreground uppercase">
              <span className={cn('size-2 rounded-full', KIND_ACCENT[kind].dot)} aria-hidden />
              {DICT_KIND_LABELS[kind]}
              <span className="font-normal tabular-nums">· {group.length}</span>
            </h3>
            <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {group.map((d) => {
                return (
                  <DictionaryCard
                    key={d.slug}
                    d={d}
                    index={ordinal.get(d.slug) ?? 0}
                    on={selectedSet.has(d.slug)}
                    last={selectedSet.has(d.slug) && selected.length === 1}
                    onToggle={() => toggle(d.slug)}
                  />
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function DictionaryCard({
  d,
  index,
  on,
  last,
  onToggle
}: {
  d: Dictionary;
  index: number;
  on: boolean;
  last: boolean;
  onToggle: () => void;
}) {
  const accent = KIND_ACCENT[d.kind];
  return (
    <article
      className={cn(
        'group relative flex flex-col overflow-hidden rounded-2xl border bg-card/80 p-4 transition-all duration-300',
        'hover:-translate-y-0.5 hover:border-primary/40 hover:shadow-lg hover:shadow-primary/5',
        on && 'border-primary/40 bg-primary/[0.03]'
      )}
    >
      <span
        aria-hidden
        className={cn(
          'absolute inset-x-0 top-0 h-0.5 bg-gradient-to-r opacity-0 transition-opacity group-hover:opacity-100',
          accent.stripe,
          on && 'opacity-100'
        )}
      />
      <div className="flex items-start gap-2">
        <span className="pt-0.5 text-[11px] text-muted-foreground/70 tabular-nums">
          {String(index).padStart(2, '0')}
        </span>
        <div className="flex min-w-0 flex-1 flex-col gap-1.5">
          <div className="flex flex-wrap items-center gap-1.5">
            <span
              className={cn(
                'rounded-md px-1.5 py-0.5 font-devanagari text-xs font-bold ring-1',
                accent.badge
              )}
            >
              {d.short_name}
            </span>
            <DirectionTag dictionary={d} />
            <span className="text-[11px] text-muted-foreground tabular-nums">
              {d.entry_count.toLocaleString('en-IN')}
            </span>
          </div>
          <Link
            to="/dict/$slug"
            params={{ slug: d.slug }}
            search={{}}
            className="text-sm leading-snug font-semibold after:absolute after:inset-0 hover:text-primary"
          >
            {d.name}
          </Link>
          {d.description ? (
            <p className="line-clamp-2 text-xs leading-relaxed text-muted-foreground">
              {d.description}
            </p>
          ) : null}
        </div>
      </div>
      <div className="relative z-10 mt-auto flex items-center justify-between pt-3">
        <button
          type="button"
          onClick={onToggle}
          disabled={last}
          title={last ? 'At least one dictionary stays selected' : undefined}
          className={cn(
            'inline-flex h-7 items-center gap-1 rounded-full px-2.5 text-xs font-medium transition-all disabled:cursor-not-allowed disabled:opacity-60',
            on
              ? 'bg-primary text-primary-foreground shadow-sm shadow-primary/30 hover:bg-primary/90'
              : 'border border-dashed text-muted-foreground hover:border-primary/50 hover:text-primary'
          )}
        >
          {on ? (
            <CheckIcon className="size-3.5" aria-hidden />
          ) : (
            <PlusIcon className="size-3.5" aria-hidden />
          )}
          {on ? 'In search' : 'Add to search'}
        </button>
        <span className="pointer-events-none flex items-center gap-1 text-xs text-muted-foreground opacity-0 transition-all group-hover:translate-x-0.5 group-hover:opacity-100">
          Browse <ArrowRightIcon className="size-3.5" aria-hidden />
        </span>
      </div>
    </article>
  );
}

/* ───────────────────────────── Results ───────────────────────────── */

function ResultsSkeleton() {
  return (
    <>
      <div className="hidden flex-col gap-2 lg:col-start-1 lg:row-span-2 lg:row-start-1 lg:flex">
        {Array.from({ length: 5 }).map((_, i) => (
          <Skeleton key={i} className="h-9 rounded-lg" />
        ))}
      </div>
      <div className="flex flex-col gap-4 lg:col-start-2" aria-label="Loading results">
        <Skeleton className="h-5 w-56" />
        <Skeleton className="h-64 rounded-2xl" />
        <Skeleton className="h-48 rounded-2xl" />
      </div>
    </>
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
  data,
  prefs,
  isAll,
  onPick
}: {
  q: string;
  mode: SearchMode;
  data: SearchResponse;
  prefs: SearchPrefs;
  isAll: boolean;
  onPick: (chip: string) => void;
}) {
  const groups = data.groups as DictionaryHits[];
  const totalHits = groups.reduce((n, g) => n + g.total, 0);
  const nothingFound = groups.length === 0 && data.forms.length === 0;
  const active = useActiveSection(groups.map((g) => `dict-${g.dictionary.slug}`));

  return (
    <>
      <aside
        aria-label="Dictionaries with hits"
        className="hidden lg:col-start-1 lg:row-span-2 lg:row-start-1 lg:block"
      >
        {groups.length > 0 ? (
          <nav className="sticky top-20 flex flex-col gap-0.5">
            <p className="px-2 pb-2 text-[11px] font-semibold tracking-widest text-muted-foreground uppercase">
              Results
            </p>
            {groups.map((g, i) => (
              <SectionLink
                key={g.dictionary.slug}
                g={g}
                index={i}
                active={active === `dict-${g.dictionary.slug}`}
              />
            ))}
          </nav>
        ) : null}
      </aside>

      <div className="flex min-w-0 flex-col gap-4 lg:col-start-2">
        <div
          className="flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-muted-foreground"
          role="status"
        >
          {totalHits > 0 ? (
            <span>
              <strong className="text-foreground tabular-nums">{totalHits}</strong>{' '}
              {totalHits === 1 ? 'hit' : 'hits'} in{' '}
              <strong className="text-foreground tabular-nums">{groups.length}</strong>{' '}
              {groups.length === 1 ? 'work' : 'works'} for{' '}
              <span className="font-devanagari font-semibold text-foreground">{q}</span>
              {mode === 'en' ? ' in English' : null}
            </span>
          ) : (
            <span>
              No dictionary entries for{' '}
              <span className="font-devanagari font-semibold text-foreground">{q}</span>
            </span>
          )}
          {data.slp1.length > 0 ? (
            <span className="rounded-md bg-muted px-1.5 py-0.5 font-mono text-xs">
              {data.slp1.join(' · ')}
            </span>
          ) : null}
        </div>

        {groups.length > 1 ? <MobileSectionStrip groups={groups} active={active} /> : null}

        {data.forms.length > 0 ? <FormsCard forms={data.forms as FormAnalysis[]} /> : null}

        {nothingFound ? (
          <Empty className="rounded-2xl border border-dashed">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <BookOpenTextIcon aria-hidden />
              </EmptyMedia>
              <EmptyTitle>No entries found</EmptyTitle>
              <EmptyDescription>
                Try another spelling (Devanagari, IAST or SLP1), widen the dictionaries, or switch
                to English mode.
              </EmptyDescription>
            </EmptyHeader>
            {data.did_you_mean.length > 0 ? (
              <div className="flex flex-wrap items-center justify-center gap-2">
                <span className="text-sm text-muted-foreground">Did you mean</span>
                {data.did_you_mean.map((chip) => (
                  <Button
                    key={chip}
                    variant="outline"
                    size="sm"
                    className="rounded-full font-devanagari"
                    onClick={() => onPick(chip)}
                  >
                    {chip}
                  </Button>
                ))}
              </div>
            ) : null}
            <div className="flex flex-wrap justify-center gap-2">
              {mode === 'sa' && !prefs.forms ? (
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => setSearchPrefs({ forms: true })}
                >
                  <SparklesIcon aria-hidden /> Look up inflected forms
                </Button>
              ) : null}
              {!isAll ? (
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => setSearchPrefs({ dicts: 'all' })}
                >
                  <LibraryBigIcon aria-hidden /> Search all dictionaries
                </Button>
              ) : null}
            </div>
          </Empty>
        ) : (
          <div className="flex flex-col gap-5">
            {groups.map((g, i) => (
              <div
                key={g.dictionary.slug}
                className="animate-in duration-500 fade-in-0 fill-mode-both slide-in-from-bottom-2"
                style={{ animationDelay: `${Math.min(i, 6) * 60}ms` }}
              >
                <DictionaryGroupCard
                  dictionary={g.dictionary}
                  entries={g.entries}
                  total={g.total}
                  index={groups.length > 1 ? i + 1 : undefined}
                />
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}

/** Which result section is currently at the top of the viewport. */
function useActiveSection(ids: string[]): string | null {
  const key = ids.join(',');
  const [active, setActive] = useState<string | null>(null);
  useEffect(() => {
    const list = key ? key.split(',') : [];
    if (list.length === 0) return;
    const visible = new Map<string, boolean>();
    const obs = new IntersectionObserver(
      (records) => {
        for (const r of records) visible.set(r.target.id, r.isIntersecting);
        const first = list.find((id) => visible.get(id));
        if (first) setActive(first);
      },
      { rootMargin: '-120px 0px -55% 0px' }
    );
    for (const id of list) {
      const el = document.getElementById(id);
      if (el) obs.observe(el);
    }
    return () => obs.disconnect();
  }, [key]);
  return active;
}

function SectionLink({ g, index, active }: { g: DictionaryHits; index: number; active: boolean }) {
  const accent = KIND_ACCENT[g.dictionary.kind];
  return (
    <a
      href={`#dict-${g.dictionary.slug}`}
      className={cn(
        'group flex items-center gap-2 rounded-lg px-2 py-1.5 text-sm transition-colors',
        active
          ? 'bg-primary/10 text-foreground'
          : 'text-muted-foreground hover:bg-muted hover:text-foreground'
      )}
      title={g.dictionary.name}
    >
      <span className="w-4 text-right text-[11px] tabular-nums opacity-70">{index + 1}</span>
      <span className={cn('size-1.5 shrink-0 rounded-full', accent.dot)} aria-hidden />
      <span className="min-w-0 flex-1 truncate font-devanagari font-medium">
        {g.dictionary.short_name}
      </span>
      <span
        className={cn(
          'rounded-full px-1.5 text-[11px] tabular-nums',
          active ? 'bg-primary text-primary-foreground' : 'bg-muted'
        )}
      >
        {g.total}
      </span>
    </a>
  );
}

function MobileSectionStrip({
  groups,
  active
}: {
  groups: DictionaryHits[];
  active: string | null;
}) {
  return (
    <nav
      aria-label="Jump to dictionary"
      className="sticky top-14 z-30 -mx-3 border-b border-border/60 bg-background/80 px-3 py-2 backdrop-blur-xl lg:hidden"
    >
      <div className="flex [scrollbar-width:none] gap-1.5 overflow-x-auto">
        {groups.map((g, i) => {
          const on = active === `dict-${g.dictionary.slug}`;
          return (
            <a
              key={g.dictionary.slug}
              href={`#dict-${g.dictionary.slug}`}
              className={cn(
                'flex shrink-0 items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs transition-colors',
                on
                  ? 'border-primary/50 bg-primary/10 text-foreground'
                  : 'bg-card text-muted-foreground'
              )}
            >
              <span className="text-[10px] tabular-nums opacity-60">{i + 1}</span>
              <span className="font-devanagari font-medium">{g.dictionary.short_name}</span>
              <span className="tabular-nums opacity-70">{g.total}</span>
            </a>
          );
        })}
      </div>
    </nav>
  );
}

function FormsCard({ forms }: { forms: FormAnalysis[] }) {
  const [openForm, setOpenForm] = useState<FormAnalysis | null>(null);
  const navigate = useNavigate();
  const goLemma = (lemma: string) => void navigate({ to: '/', search: { q: lemma } });
  return (
    <section className="relative animate-in overflow-hidden rounded-2xl border border-sky-500/25 bg-gradient-to-br from-sky-500/[0.07] via-card to-card p-4 duration-500 fade-in-0">
      <h2 className="mb-3 flex items-center gap-2 text-xs font-semibold tracking-widest text-sky-700 uppercase dark:text-sky-300">
        <SparklesIcon className="size-3.5" aria-hidden /> Inflected form → lemma
      </h2>
      <ul className="flex flex-col divide-y divide-border/60">
        {forms.map((f, i) => (
          <li
            key={`${f.form}-${f.lemma}-${i}`}
            className="flex flex-wrap items-center gap-x-2 gap-y-1.5 py-2 text-sm first:pt-0 last:pb-0"
          >
            <span className="font-devanagari text-lg font-semibold">{f.form}</span>
            <ArrowRightIcon className="size-4 text-muted-foreground" aria-hidden />
            <button
              type="button"
              className="font-devanagari text-lg font-semibold text-primary underline-offset-4 hover:underline"
              onClick={() => goLemma(f.lemma)}
            >
              {f.lemma}
            </button>
            {f.root ? (
              <span className="text-xs text-muted-foreground">
                from{' '}
                <button
                  type="button"
                  className="font-devanagari text-sm font-semibold text-primary underline-offset-4 hover:underline"
                  onClick={() => goLemma(f.root!)}
                >
                  {f.root}
                </button>
              </span>
            ) : null}
            <span className="flex flex-wrap gap-1">
              {f.analysis.split('; ').map((a) => (
                <span
                  key={a}
                  className="rounded-full bg-muted px-2 py-0.5 font-devanagari text-xs text-muted-foreground"
                >
                  {a}
                </span>
              ))}
            </span>
            <ParadigmSourceBadge source={f.source} />
            {f.paradigm_ids.length > 0 ? (
              <Button
                variant="outline"
                size="xs"
                className="ml-auto rounded-full"
                onClick={() => setOpenForm(f)}
              >
                <TableIcon aria-hidden /> Table
              </Button>
            ) : null}
          </li>
        ))}
      </ul>
      {openForm ? (
        <FormParadigmDialog
          key={openForm.paradigm_ids.join(',')}
          form={openForm}
          open={openForm !== null}
          onOpenChange={(v) => {
            if (!v) setOpenForm(null);
          }}
        />
      ) : null}
    </section>
  );
}
