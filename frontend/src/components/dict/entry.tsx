import { useState } from 'react';
import { Link } from '@tanstack/react-router';
import { ChevronDownIcon } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { cn } from '@/lib/utils';
import type { Dictionary, EntryHit } from '@/lib/api/types';
import { useDisplayText, useDisplayTexts } from '@/components/script/display-script';
import { MatchBadge, SenseHtml } from './sense';

/** Entries longer than this (HTML chars) start collapsed. */
const COLLAPSE_CHARS = 1400;

function DataList({ data }: { data: Record<string, unknown> }) {
  const rows = Object.entries(data);
  if (rows.length === 0) return null;
  return (
    <dl className="mt-3 grid grid-cols-1 gap-x-6 gap-y-1 rounded-lg bg-muted/60 p-3 text-sm sm:grid-cols-2">
      {rows.map(([k, v]) => (
        <div key={k} className="flex gap-2">
          <dt className="shrink-0 font-semibold text-muted-foreground">{k}</dt>
          <dd className="min-w-0 break-words">{formatDataValue(v)}</dd>
        </div>
      ))}
    </dl>
  );
}

function formatDataValue(v: unknown): string {
  if (v === null || v === undefined) return '—';
  if (typeof v === 'string' || typeof v === 'number' || typeof v === 'boolean') return String(v);
  try {
    return JSON.stringify(v);
  } catch {
    return String(v);
  }
}

function SutraDhatuLinks({ entry }: { entry: EntryHit | import('@/lib/api/types').Entry }) {
  if (!entry.sutra_id && !entry.dhatu_id) return null;
  return (
    <div className="mt-2 flex flex-wrap gap-2 text-sm">
      {entry.sutra_id ? (
        <Link
          to="/sutra/$id"
          params={{ id: entry.sutra_id }}
          className="text-primary underline underline-offset-2"
        >
          Sūtra {entry.sutra_id}
        </Link>
      ) : null}
      {entry.dhatu_id ? (
        <Link
          to="/dhatu/$id"
          params={{ id: String(entry.dhatu_id) }}
          className="text-primary underline underline-offset-2"
        >
          Dhātu #{entry.dhatu_id}
        </Link>
      ) : null}
    </div>
  );
}

export function EntryHeadword({ headword, className }: { headword: string; className?: string }) {
  const display = useDisplayText(headword);
  return <span className={cn('font-devanagari', className)}>{display}</span>;
}

/**
 * Renders each sense as its own block (never slicing HTML strings). Long
 * entries are clipped with CSS and a fade until expanded.
 */
export function EntrySenses({ senses }: { senses: string[] }) {
  const totalChars = senses.reduce((n, s) => n + s.length, 0);
  const collapsible = totalChars > COLLAPSE_CHARS;
  const [expanded, setExpanded] = useState(false);
  const clipped = collapsible && !expanded;
  return (
    <div>
      <div
        className={cn(
          'relative',
          clipped &&
            'max-h-64 overflow-hidden [mask-image:linear-gradient(to_bottom,black_70%,transparent)]'
        )}
      >
        {senses.length === 1 ? (
          <SenseHtml html={senses[0]} />
        ) : (
          <ol className="space-y-1.5">
            {senses.map((html, i) => (
              <li key={i} className="flex gap-2.5">
                <span className="mt-[0.45rem] shrink-0 text-xs text-muted-foreground tabular-nums">
                  {i + 1}.
                </span>
                <SenseHtml html={html} className="min-w-0 flex-1" />
              </li>
            ))}
          </ol>
        )}
      </div>
      {collapsible ? (
        <Button
          variant="ghost"
          size="sm"
          className="mt-1 h-7 px-2 text-xs"
          onClick={() => setExpanded((v) => !v)}
          aria-expanded={expanded}
        >
          {expanded ? 'Show less' : 'Show more'}
          <ChevronDownIcon
            className={cn('size-3.5 transition-transform', expanded && 'rotate-180')}
          />
        </Button>
      ) : null}
    </div>
  );
}

/**
 * One dictionary's hit group on the search page.
 */
export function DictionaryGroupCard({
  dictionary,
  entries,
  total,
  showMatch = true
}: {
  dictionary: Dictionary;
  entries: EntryHit[];
  total: number;
  showMatch?: boolean;
}) {
  const headwords = useDisplayTexts(entries.map((e) => e.headword));
  return (
    <section id={`dict-${dictionary.slug}`} aria-label={dictionary.name} className="scroll-mt-24">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between gap-2 border-b py-3">
          <div className="flex min-w-0 items-center gap-2">
            <Badge variant="secondary" className="shrink-0 font-bold">
              {dictionary.short_name}
            </Badge>
            <h2 className="truncate text-sm font-semibold sm:text-base">{dictionary.name}</h2>
          </div>
          <span className="shrink-0 text-xs text-muted-foreground tabular-nums">
            {total} {total === 1 ? 'hit' : 'hits'}
          </span>
        </CardHeader>
        <CardContent className="flex flex-col gap-5 p-4 sm:p-5">
          {entries.map((entry, i) => (
            <article key={entry.id} className="flex flex-col gap-1.5">
              <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                <Link
                  to="/entry/$id"
                  params={{ id: String(entry.id) }}
                  className="font-devanagari text-xl font-semibold hover:text-primary hover:underline sm:text-2xl"
                >
                  {headwords[i] ?? entry.headword}
                </Link>
                {showMatch ? <MatchBadge match={entry.match} /> : null}
              </div>
              {entry.snippet ? (
                <SenseHtml html={entry.snippet} className="text-sm text-muted-foreground" />
              ) : null}
              <EntrySenses senses={entry.senses} />
              {entry.data ? <DataList data={entry.data} /> : null}
              <SutraDhatuLinks entry={entry} />
            </article>
          ))}
        </CardContent>
      </Card>
    </section>
  );
}

export function SnippetOnlyCard({
  dictionary,
  entries
}: {
  dictionary: Dictionary;
  entries: EntryHit[];
}) {
  return <DictionaryGroupCard dictionary={dictionary} entries={entries} total={entries.length} />;
}

export function SutraText({ text, className }: { text: string; className?: string }) {
  const display = useDisplayText(text);
  return <span className={`font-devanagari ${className ?? ''}`}>{display}</span>;
}

export function DhatuHeadword({ text, className }: { text: string; className?: string }) {
  const display = useDisplayText(text);
  return <span className={`font-devanagari ${className ?? ''}`}>{display}</span>;
}
