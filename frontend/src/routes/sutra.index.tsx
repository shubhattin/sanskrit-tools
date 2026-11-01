import { useDeferredValue, useEffect, useMemo, useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { BookOpenTextIcon, LanguagesIcon, SearchIcon, XIcon } from 'lucide-react';
import { searchString } from '@/lib/search-params';
import { allSutrasQuery } from '@/lib/api/client';
import type { Sutra } from '@/lib/api/types';
import { asciiDigits, devanagariCandidates } from '@/lib/translit';
import { useLipiInput } from '@/components/lipi/typing';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group';
import { Kbd } from '@/components/ui/kbd';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { Spinner } from '@/components/ui/spinner';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import { SutraText } from '@/components/dict/entry';

export const Route = createFileRoute('/sutra/')({
  validateSearch: (search: Record<string, unknown>): { q?: string } => ({
    q: searchString(search.q)
  }),
  component: SutraListPage
});

const PAGE = 150;

/** `1`, `1.1`, `1.1.1`, `1.1.001` (optionally with a trailing dot). */
const NUMBER_TOKEN = /^(\d)(?:\.(\d)(?:\.(\d{1,3}))?)?\.?$/;

interface ParsedQuery {
  number: { adhyaya: number; pada?: number; num?: number } | null;
  /** Non-number part of the query, as typed. */
  text: string;
}

/** Split the number part (Devanagari digits allowed) from the text part. */
function parseQuery(raw: string): ParsedQuery {
  let number: ParsedQuery['number'] = null;
  const rest: string[] = [];
  for (const token of asciiDigits(raw).trim().split(/\s+/).filter(Boolean)) {
    const m: RegExpExecArray | null = number ? null : NUMBER_TOKEN.exec(token);
    if (m) {
      number = {
        adhyaya: Number(m[1]),
        pada: m[2] ? Number(m[2]) : undefined,
        num: m[3] ? Number(m[3]) : undefined
      };
    } else rest.push(token);
  }
  return { number, text: rest.join(' ') };
}

/**
 * Comparison key: no spaces / danda / accents / ZWJ, and no virāma at word
 * ends (इको यण् matches इको यणचि across sandhi).
 */
function matchKey(text: string, query = false): string {
  let t = text
    .normalize('NFC')
    .replace(/\u200c|\u200d/g, '')
    .replace(/[\u0951-\u0954\u093d\u0964\u0965'’.,;:-]/g, '');
  if (query) t = t.replace(/्(?=\s|$)/g, '');
  return t.replace(/\s+/g, '');
}

function useTextCandidates(text: string): { keys: string[]; pending: boolean } {
  const t = text.trim();
  const q = useQuery({
    queryKey: ['translit-candidates', t],
    queryFn: () => devanagariCandidates(t),
    enabled: t.length > 0,
    staleTime: Infinity,
    // Keep the previous match while the next transliteration resolves (no flash of all rows).
    placeholderData: (prev) => prev
  });
  const keys = useMemo(
    () => [...new Set((q.data ?? []).map((c) => matchKey(c, true)).filter(Boolean))],
    [q.data]
  );
  return {
    keys: t.length > 0 ? keys : [],
    pending: t.length > 0 && (q.isPending || q.isPlaceholderData)
  };
}

function filterSutras(all: Sutra[], parsed: ParsedQuery, keys: string[]): Sutra[] {
  const n = parsed.number;
  let rows = n
    ? all.filter(
        (s) =>
          s.adhyaya === n.adhyaya &&
          (n.pada === undefined || s.pada === n.pada) &&
          (n.num === undefined || s.num === n.num)
      )
    : all;
  if (keys.length > 0) {
    const scored: { s: Sutra; starts: boolean }[] = [];
    for (const s of rows) {
      const k = matchKey(s.text);
      const hit = keys.find((q) => k.includes(q));
      if (hit) scored.push({ s, starts: k.startsWith(hit) });
    }
    // Sūtras starting with the query first, then book order (stable sort).
    rows = scored.sort((a, b) => Number(b.starts) - Number(a.starts)).map((x) => x.s);
  }
  return rows;
}

function SutraListPage() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const [text, setText] = useState(search.q ?? '');
  const [shown, setShown] = useState(PAGE);
  const { props: lipiProps, enabled, setEnabled } = useLipiInput(text, setText, false);

  const all = useQuery(allSutrasQuery());
  const deferred = useDeferredValue(text);
  const parsed = useMemo(() => parseQuery(deferred), [deferred]);
  const { keys, pending } = useTextCandidates(parsed.text);
  const results = useMemo(
    () => (all.data ? filterSutras(all.data, parsed, keys) : []),
    [all.data, parsed, keys]
  );

  // Reset paging when the query changes (render-time adjustment).
  const [pagedFor, setPagedFor] = useState(deferred);
  if (pagedFor !== deferred) {
    setPagedFor(deferred);
    setShown(PAGE);
  }

  // Keep ?q= in sync (replace, debounced) so the filter survives reloads / sharing.
  useEffect(() => {
    const t = setTimeout(() => {
      const q = text.trim() || undefined;
      if (q !== (search.q || undefined))
        void navigate({ to: '/sutra', search: { q }, replace: true });
    }, 400);
    return () => clearTimeout(t);
  }, [text, search.q, navigate]);

  const filtering = parsed.text.length > 0 || parsed.number !== null;

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-bold sm:text-3xl">Aṣṭādhyāyī Sūtras</h1>
      <div className="flex flex-wrap items-center gap-3">
        <form role="search" className="w-full max-w-xl" onSubmit={(e) => e.preventDefault()}>
          <InputGroup className="h-11 rounded-xl">
            <InputGroupAddon>
              <SearchIcon aria-hidden />
            </InputGroupAddon>
            <InputGroupInput
              {...lipiProps}
              placeholder={
                enabled
                  ? 'vRddhi, १.१, 1.1.3…'
                  : 'Number (1.1, १.१) or text (vRddhi, vṛddhi, वृद्धि)…'
              }
              aria-label="Filter sūtras"
              autoComplete="off"
              autoCapitalize="off"
              spellCheck={false}
              className="font-devanagari text-base"
            />
            {pending ? (
              <InputGroupAddon align="inline-end">
                <Spinner label="Transliterating" />
              </InputGroupAddon>
            ) : null}
            {text ? (
              <InputGroupAddon align="inline-end">
                <button
                  type="button"
                  aria-label="Clear"
                  className="grid size-7 place-items-center rounded-full text-muted-foreground hover:bg-muted hover:text-foreground"
                  onClick={() => setText('')}
                >
                  <XIcon className="size-4" aria-hidden />
                </button>
              </InputGroupAddon>
            ) : null}
            <InputGroupAddon align="inline-end">
              <Label
                className="flex cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-1 hover:bg-muted"
                title="Lipi Lekhika typing: Latin → Devanagari as you type (Alt+X)"
              >
                <LanguagesIcon className={enabled ? 'size-4 text-primary' : 'size-4'} aria-hidden />
                <Switch
                  size="sm"
                  checked={enabled}
                  onCheckedChange={setEnabled}
                  aria-label="Lipi Lekhika typing"
                />
              </Label>
            </InputGroupAddon>
          </InputGroup>
        </form>
        <p className="text-xs text-muted-foreground">
          Latin text is matched as Devanagari too · <Kbd>Alt</Kbd>+<Kbd>X</Kbd> typing
        </p>
      </div>

      {all.isPending ? (
        <div
          className="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3"
          aria-label="Loading sutras"
        >
          {Array.from({ length: 12 }).map((_, i) => (
            <Skeleton key={i} className="h-14 rounded-xl" />
          ))}
        </div>
      ) : all.isError ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>Could not load sūtras</EmptyTitle>
            <EmptyDescription>{all.error.message}</EmptyDescription>
          </EmptyHeader>
          <Button variant="outline" size="sm" onClick={() => void all.refetch()}>
            Retry
          </Button>
        </Empty>
      ) : results.length === 0 && !pending ? (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <BookOpenTextIcon aria-hidden />
            </EmptyMedia>
            <EmptyTitle>No sūtras found</EmptyTitle>
            <EmptyDescription>
              Try a number like 1.1.1, Devanagari, or lipilekhika Latin (vRddhi, iko yaN).
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <>
          <p className="text-sm text-muted-foreground tabular-nums" role="status">
            {filtering
              ? `${results.length} of ${all.data.length} sūtras`
              : `${all.data.length} sūtras`}
            {keys.length > 0 && /[A-Za-z]/.test(parsed.text) ? (
              <span className="ml-2 font-devanagari">· {keys.join(' / ')}</span>
            ) : null}
          </p>
          <ol className="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
            {results.slice(0, shown).map((s) => (
              <li key={s.id}>
                <Link to="/sutra/$id" params={{ id: s.id }} className="block h-full">
                  <Card className="h-full py-0 transition-all hover:-translate-y-px hover:border-primary/50 hover:shadow-md">
                    <CardContent className="flex items-baseline gap-3 p-3 sm:p-4">
                      <Badge variant="secondary" className="shrink-0 tabular-nums">
                        {s.id}
                      </Badge>
                      <SutraText text={s.text} className="text-lg" />
                    </CardContent>
                  </Card>
                </Link>
              </li>
            ))}
          </ol>
          {results.length > shown ? (
            <Button
              variant="outline"
              className="self-center rounded-full"
              onClick={() => setShown((n) => n + PAGE * 2)}
            >
              Show more ({results.length - shown} left)
            </Button>
          ) : null}
        </>
      )}
    </div>
  );
}
