import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Link, useNavigate } from '@tanstack/react-router';
import { useQueries, useQuery } from '@tanstack/react-query';
import { paradigmQuery, paradigmsQuery } from '@/lib/api/client';
import {
  PARADIGM_SOURCE_LABELS,
  type FormAnalysis,
  type Paradigm,
  type ParadigmSource,
  type ParadigmSummary,
  type ParadigmTable
} from '@/lib/api/types';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog';
import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { cn } from '@/lib/utils';

export function ParadigmSourceBadge({ source }: { source: ParadigmSource }) {
  return <Badge variant="secondary">{PARADIGM_SOURCE_LABELS[source]}</Badge>;
}

type GridTableT = Extract<ParadigmTable, { rows: unknown }>;
type ListTableT = Extract<ParadigmTable, { list: unknown }>;

function isGridTable(t: ParadigmTable): t is GridTableT {
  return 'rows' in t;
}

/** Cell numbers parallel to `tables`: grid → rows of cell numbers; list → item numbers. */
type TableNums = { kind: 'grid'; rows: number[][] } | { kind: 'list'; items: number[] };

function numberTables(tables: ParadigmTable[]): TableNums[] {
  let n = 0;
  return tables.map((t) => {
    if (isGridTable(t)) {
      return { kind: 'grid', rows: t.rows.map((r) => r.cells.map(() => n++)) };
    }
    return { kind: 'list', items: t.list.map(() => n++) };
  });
}

const HIGHLIGHT_CELL = 'bg-primary/15 ring-1 ring-primary/40 rounded';

/**
 * Renders one grid table (declension row-labels × cols, conjugation puruṣa × vacana).
 * `corner` is the top-left header cell (declension label); pass null when the
 * label is shown as a caption instead (conjugation voices).
 */
function GridTable({
  table,
  nums,
  highlight,
  corner,
  firstNum,
  firstRefCb
}: {
  table: GridTableT;
  nums: Extract<TableNums, { kind: 'grid' }>;
  highlight: Set<number>;
  corner: string | null;
  firstNum: number | null;
  firstRefCb: (el: HTMLElement | null) => void;
}) {
  return (
    <div className="overflow-x-auto rounded-lg border">
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="bg-muted/60">
            <th
              scope="col"
              className="font-devanagari min-w-24 px-2 py-1.5 text-left font-semibold whitespace-nowrap"
            >
              {corner ?? ''}
            </th>
            {table.cols.map((c) => (
              <th
                key={c}
                scope="col"
                className="font-devanagari min-w-20 px-2 py-1.5 text-left font-semibold whitespace-nowrap"
              >
                {c}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {table.rows.map((row, ri) => (
            <tr key={ri} className="border-t">
              <th
                scope="row"
                className="font-devanagari px-2 py-1.5 text-left font-medium whitespace-nowrap text-muted-foreground"
              >
                {row.label}
              </th>
              {row.cells.map((forms, ci) => {
                const num = nums.rows[ri]?.[ci] ?? -1;
                const on = highlight.has(num);
                return (
                  <td
                    key={ci}
                    data-cell={num}
                    ref={num === firstNum ? firstRefCb : undefined}
                    className={cn(
                      'px-1 py-1 align-top',
                      on && 'bg-primary/15 ring-1 ring-primary/40'
                    )}
                  >
                    {forms.length === 0 ? (
                      <span className="text-muted-foreground">—</span>
                    ) : (
                      <div className="flex flex-col gap-0.5">
                        {forms.map((f) => (
                          <span key={f} className="font-devanagari tabular-nums">
                            {f}
                          </span>
                        ))}
                      </div>
                    )}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ListTable({
  table,
  nums,
  highlight,
  firstNum,
  firstRefCb
}: {
  table: ListTableT;
  nums: Extract<TableNums, { kind: 'list' }>;
  highlight: Set<number>;
  firstNum: number | null;
  firstRefCb: (el: HTMLElement | null) => void;
}) {
  const navigate = useNavigate();
  return (
    <dl className="flex flex-col gap-1.5">
      {table.list.map((item, i) => {
        const num = nums.items[i] ?? -1;
        const on = highlight.has(num);
        return (
          <div
            key={i}
            data-cell={num}
            ref={num === firstNum ? firstRefCb : undefined}
            className={cn(
              'flex flex-wrap items-baseline gap-x-2 rounded px-1 py-0.5',
              on && HIGHLIGHT_CELL
            )}
          >
            <dt className="font-devanagari shrink-0 text-sm font-semibold text-muted-foreground">
              {item.label}
            </dt>
            <dd className="font-devanagari min-w-0 text-[15px]">{item.text}</dd>
            <dd className="flex flex-wrap gap-1">
              {item.forms.map((f) => (
                <button
                  key={f}
                  type="button"
                  onClick={() => void navigate({ to: '/', search: { q: f } })}
                  className="font-devanagari rounded bg-muted px-1.5 py-px text-sm text-primary underline-offset-2 hover:underline"
                >
                  {f}
                </button>
              ))}
            </dd>
          </div>
        );
      })}
    </dl>
  );
}

/** Tables of one conjugation `group` (one tab): sections per lakāra title. */
function ConjugationGroup({
  tables,
  indexes,
  nums,
  highlight,
  firstNum,
  firstRefCb
}: {
  tables: ParadigmTable[];
  indexes: number[];
  nums: TableNums[];
  highlight: Set<number>;
  firstNum: number | null;
  firstRefCb: (el: HTMLElement | null) => void;
}) {
  const sections = useMemo(() => {
    const out: { title: string | null; idx: number[] }[] = [];
    for (const i of indexes) {
      const title = tables[i]?.title ?? null;
      const last = out[out.length - 1];
      if (last && last.title === title) last.idx.push(i);
      else out.push({ title, idx: [i] });
    }
    return out;
  }, [tables, indexes]);

  return (
    <div className="flex flex-col gap-6">
      {sections.map((s, si) => {
        const gridIdx = s.idx.filter((i) => nums[i]?.kind === 'grid');
        const listIdx = s.idx.filter((i) => nums[i]?.kind === 'list');
        return (
          <section key={`${s.title ?? ''}-${si}`} aria-label={s.title ?? 'forms'}>
            {s.title ? (
              <h4 className="font-devanagari mb-2 text-base font-semibold">{s.title}</h4>
            ) : null}
            {gridIdx.length > 0 ? (
              <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
                {gridIdx.map((i) => {
                  const t = tables[i];
                  const n = nums[i];
                  if (!t || !n || !isGridTable(t) || n.kind !== 'grid') return null;
                  return (
                    <figure key={i} className="flex min-w-0 flex-col gap-1">
                      {t.label ? (
                        <figcaption className="font-devanagari text-sm font-semibold">
                          {t.label}
                        </figcaption>
                      ) : null}
                      <GridTable
                        table={t}
                        nums={n}
                        highlight={highlight}
                        corner={null}
                        firstNum={firstNum}
                        firstRefCb={firstRefCb}
                      />
                    </figure>
                  );
                })}
              </div>
            ) : null}
            {listIdx.map((i) => {
              const t = tables[i];
              const n = nums[i];
              if (!t || !n || isGridTable(t) || n.kind !== 'list') return null;
              return (
                <ListTable
                  key={i}
                  table={t}
                  nums={n}
                  highlight={highlight}
                  firstNum={firstNum}
                  firstRefCb={firstRefCb}
                />
              );
            })}
          </section>
        );
      })}
    </div>
  );
}

/**
 * Renders a full paradigm. `highlight` lists cell numbers (running counter
 * from 0 over tables: grid cells row-major, list items) to emphasize; the
 * first highlighted cell scrolls into view and its tab opens initially.
 */
export function ParadigmView({
  paradigm,
  highlight = []
}: {
  paradigm: Paradigm;
  highlight?: number[];
}) {
  const tables = paradigm.tables;
  const nums = useMemo(() => numberTables(tables), [tables]);
  const highlightSet = useMemo(() => new Set(highlight), [highlight]);
  const firstNum = useMemo(
    () => (highlight.length > 0 ? Math.min(...highlight) : null),
    [highlight]
  );
  const firstRef = useRef<HTMLElement | null>(null);
  const firstRefCb = useCallback((el: HTMLElement | null) => {
    firstRef.current = el;
  }, []);

  // Groups in order of first appearance; null group renders as 'रूपाणि'.
  const groups = useMemo(() => {
    const out: { key: string; label: string; idx: number[] }[] = [];
    tables.forEach((t, i) => {
      const key = t.group ?? '';
      const last = out[out.length - 1];
      if (last && last.key === key) last.idx.push(i);
      else out.push({ key, label: t.group ?? 'रूपाणि', idx: [i] });
    });
    return out;
  }, [tables]);

  // Tab holding the first highlighted cell, else the first tab.
  const defaultTab = useMemo(() => {
    if (firstNum !== null) {
      const g = groups.find((gr) =>
        gr.idx.some((i) => {
          const n = nums[i];
          if (!n) return false;
          return n.kind === 'grid'
            ? n.rows.some((r) => r.includes(firstNum))
            : n.items.includes(firstNum);
        })
      );
      if (g) return g.key;
    }
    return groups[0]?.key ?? '';
  }, [groups, nums, firstNum]);

  const [tab, setTab] = useState(defaultTab);
  useEffect(() => {
    setTab(defaultTab);
  }, [defaultTab]);

  useEffect(() => {
    firstRef.current?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  }, [tab]);

  const highlightKey = highlight.join(',');
  useEffect(() => {
    firstRef.current?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [highlightKey]);

  if (tables.length === 0) {
    return <p className="text-sm text-muted-foreground">No tables in this paradigm.</p>;
  }

  if (groups.length <= 1) {
    const only = groups[0];
    if (!only) return null;
    return (
      <div className="flex flex-col gap-3">
        {only.idx.map((i) => {
          const t = tables[i];
          const n = nums[i];
          if (!t || !n) return null;
          if (isGridTable(t) && n.kind === 'grid') {
            return (
              <GridTable
                key={i}
                table={t}
                nums={n}
                highlight={highlightSet}
                corner={t.label}
                firstNum={firstNum}
                firstRefCb={firstRefCb}
              />
            );
          }
          if (!isGridTable(t) && n.kind === 'list') {
            return (
              <ListTable
                key={i}
                table={t}
                nums={n}
                highlight={highlightSet}
                firstNum={firstNum}
                firstRefCb={firstRefCb}
              />
            );
          }
          return null;
        })}
      </div>
    );
  }

  return (
    <Tabs value={tab} onValueChange={setTab} className="w-full">
      <TabsList className="h-auto max-w-full flex-wrap justify-start">
        {groups.map((g) => (
          <TabsTrigger key={g.key} value={g.key} className="font-devanagari">
            {g.label}
          </TabsTrigger>
        ))}
      </TabsList>
      {groups.map((g) => (
        <TabsContent key={g.key} value={g.key} className="mt-3">
          <ConjugationGroup
            tables={tables}
            indexes={g.idx}
            nums={nums}
            highlight={highlightSet}
            firstNum={firstNum}
            firstRefCb={firstRefCb}
          />
        </TabsContent>
      ))}
    </Tabs>
  );
}

function ParadigmDialogBody({
  paradigmId,
  highlight
}: {
  paradigmId: number;
  highlight: number[];
}) {
  const query = useQuery(paradigmQuery(paradigmId));
  const cell = highlight[0];

  if (query.isPending) {
    return (
      <div className="flex flex-col gap-2 p-1" aria-label="Loading paradigm">
        <Skeleton className="h-7 w-48" />
        <Skeleton className="h-40" />
      </div>
    );
  }

  if (query.isError) {
    return (
      <p className="p-1 text-sm text-destructive">Failed to load table: {query.error.message}</p>
    );
  }

  const p = query.data;
  return (
    <div className="flex min-h-0 flex-col gap-3">
      <DialogHeader className="pr-8">
        <DialogTitle className="font-devanagari flex flex-wrap items-baseline gap-x-3 text-2xl">
          {p.lemma}
          <span className="text-sm font-normal text-muted-foreground">
            {p.label ? <span className="font-devanagari">{p.label}</span> : null}
          </span>
        </DialogTitle>
        <DialogDescription className="flex flex-wrap items-center gap-2">
          <ParadigmSourceBadge source={p.source} />
          {p.root ? (
            <span>
              from{' '}
              <Link
                to="/"
                search={{ q: p.root }}
                className="font-devanagari text-primary underline underline-offset-2"
              >
                {p.root}
              </Link>
            </span>
          ) : null}
        </DialogDescription>
      </DialogHeader>
      <div className="min-h-0 flex-1 overflow-y-auto pr-1">
        <ParadigmView key={p.id} paradigm={p} highlight={highlight} />
      </div>
      <div className="flex justify-end border-t pt-2">
        <Link
          to="/paradigm/$id"
          params={{ id: String(p.id) }}
          search={cell !== undefined ? { cell } : {}}
          className="text-sm text-primary underline underline-offset-2"
        >
          Open full page
        </Link>
      </div>
    </div>
  );
}

/** Dialog showing one paradigm table (no highlight). Used by inflection sections. */
export function ParadigmDialog({
  paradigmId,
  highlight = [],
  open,
  onOpenChange
}: {
  paradigmId: number;
  highlight?: number[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[85vh] max-w-5xl flex-col">
        <ParadigmDialogBody paradigmId={paradigmId} highlight={highlight} />
      </DialogContent>
    </Dialog>
  );
}

/** Dialog for a search-hit inflected form: switch between its paradigms, highlight its cells. */
export function FormParadigmDialog({
  form,
  open,
  onOpenChange
}: {
  form: FormAnalysis;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const [sel, setSel] = useState(0);

  const results = useQueries({
    queries: form.paradigm_ids.map((id) => paradigmQuery(id))
  });
  const selectedId = form.paradigm_ids[Math.min(sel, form.paradigm_ids.length - 1)] ?? -1;
  const highlight = form.cells[Math.min(sel, form.cells.length - 1)] ?? [];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[85vh] max-w-5xl flex-col">
        {form.paradigm_ids.length > 1 ? (
          <div className="flex flex-wrap gap-1.5 pr-8" aria-label="Choose table">
            {form.paradigm_ids.map((id, i) => {
              const label = results[i]?.data?.label ?? `#${id}`;
              return (
                <Button
                  key={id}
                  variant={i === sel ? 'default' : 'outline'}
                  size="sm"
                  className="font-devanagari"
                  onClick={() => setSel(i)}
                >
                  {label}
                </Button>
              );
            })}
          </div>
        ) : null}
        <ParadigmDialogBody key={selectedId} paradigmId={selectedId} highlight={highlight} />
      </DialogContent>
    </Dialog>
  );
}

const SOURCE_ORDER: ParadigmSource[] = [
  'heritage_decl',
  'heritage_conj',
  'heritage_kridanta',
  'jnu_tinanta'
];

const KRIDANTA_LIMIT = 40;

function summaryButtonLabel(s: ParadigmSummary): string {
  if (s.source === 'heritage_kridanta') return s.label ? `${s.lemma} (${s.label})` : s.lemma;
  return s.label ?? s.lemma;
}

/**
 * "Inflection" card for entry/dhātu pages: summaries grouped by source.
 * Clicking a summary opens the full table dialog (no highlight).
 */
export function InflectionSection({ query }: { query: string }) {
  const q = query.trim();
  const result = useQuery({ ...paradigmsQuery(q), enabled: q.length > 0 });
  const [openId, setOpenId] = useState<number | null>(null);
  const [expandedFor, setExpandedFor] = useState<string | null>(null);
  const showAllKridanta = expandedFor === q;

  if (q.length === 0) return null;

  if (result.isPending) {
    return (
      <Card aria-label="Loading inflection">
        <CardContent className="flex flex-col gap-2 p-4">
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-9" />
        </CardContent>
      </Card>
    );
  }

  if (result.isError || !result.data || result.data.length === 0) return null;

  const bySource = new Map<ParadigmSource, ParadigmSummary[]>();
  for (const s of result.data) {
    const arr = bySource.get(s.source);
    if (arr) arr.push(s);
    else bySource.set(s.source, [s]);
  }
  const ordered = SOURCE_ORDER.filter((src) => bySource.has(src));

  return (
    <Card>
      <CardContent className="flex flex-col gap-4 p-4">
        <h2 className="text-sm font-semibold tracking-wide text-muted-foreground uppercase">
          Inflection
        </h2>
        {ordered.map((src) => {
          const items = bySource.get(src) ?? [];
          const shown =
            src === 'heritage_kridanta' && !showAllKridanta
              ? items.slice(0, KRIDANTA_LIMIT)
              : items;
          return (
            <section
              key={src}
              aria-label={PARADIGM_SOURCE_LABELS[src]}
              className="flex flex-col gap-2"
            >
              <h3 className="text-xs font-semibold text-muted-foreground">
                {PARADIGM_SOURCE_LABELS[src]} <span className="tabular-nums">({items.length})</span>
              </h3>
              <div className="flex flex-wrap gap-1.5">
                {shown.map((s) => (
                  <Button
                    key={s.id}
                    variant="outline"
                    size="sm"
                    className="font-devanagari"
                    onClick={() => setOpenId(s.id)}
                  >
                    {summaryButtonLabel(s)}
                  </Button>
                ))}
                {src === 'heritage_kridanta' && items.length > KRIDANTA_LIMIT ? (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setExpandedFor((v) => (v === q ? null : q))}
                  >
                    {showAllKridanta ? 'Show less' : `Show all ${items.length}`}
                  </Button>
                ) : null}
              </div>
            </section>
          );
        })}
        {openId !== null ? (
          <ParadigmDialog
            paradigmId={openId}
            open={openId !== null}
            onOpenChange={(v) => {
              if (!v) setOpenId(null);
            }}
          />
        ) : null}
      </CardContent>
    </Card>
  );
}
