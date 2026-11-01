import { useState } from 'react';
import { Link } from '@tanstack/react-router';
import {
  ArrowRightIcon,
  CheckCheckIcon,
  InfoIcon,
  KeyboardIcon,
  LibraryBigIcon,
  RotateCcwIcon,
  SearchIcon,
  SlidersHorizontalIcon,
  SparklesIcon
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Kbd } from '@/components/ui/kbd';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { cn } from '@/lib/utils';
import type { Dictionary } from '@/lib/api/types';
import { DICT_KIND_LABELS, DICT_KIND_ORDER } from '@/lib/api/types';
import { DEFAULT_DICTS, setSearchPrefs, type SearchPrefs } from '@/lib/search-prefs';

export function groupByKind(dicts: Dictionary[]): Map<string, Dictionary[]> {
  const map = new Map<string, Dictionary[]>();
  for (const d of [...dicts].sort((a, b) => a.sort_order - b.sort_order)) {
    const list = map.get(d.kind) ?? [];
    list.push(d);
    map.set(d.kind, list);
  }
  return map;
}

/** Small "EN→SA" tag for English→Sanskrit works. */
export function DirectionTag({ dictionary }: { dictionary: Dictionary }) {
  if (dictionary.src_lang !== 'en') return null;
  return (
    <span
      title="English headwords with Sanskrit equivalents — search these in English mode"
      className="shrink-0 rounded border border-sky-500/30 bg-sky-500/10 px-1 text-[10px] font-semibold text-sky-700 dark:text-sky-300"
    >
      English→Sanskrit
    </span>
  );
}

/** Info button → popover with the full description of one work. */
export function DictionaryInfo({
  dictionary: d,
  side = 'right'
}: {
  dictionary: Dictionary;
  side?: 'right' | 'bottom';
}) {
  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="ghost"
            size="icon-xs"
            type="button"
            className="shrink-0 rounded-full text-muted-foreground hover:text-primary data-popup-open:bg-primary/10 data-popup-open:text-primary"
            aria-label={`About ${d.name}`}
          />
        }
      >
        <InfoIcon className="size-3.5" aria-hidden />
      </PopoverTrigger>
      <PopoverContent
        side={side}
        align="start"
        className="w-[min(20rem,calc(100vw-2rem))] gap-2 rounded-xl p-3.5"
      >
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="rounded-md bg-muted px-1.5 py-0.5 font-devanagari text-xs font-bold">
            {d.short_name}
          </span>
          <span className="text-[11px] text-muted-foreground">{DICT_KIND_LABELS[d.kind]}</span>
          <DirectionTag dictionary={d} />
        </div>
        <p className="text-sm leading-snug font-semibold">{d.name}</p>
        {d.description ? (
          <p className="text-xs leading-relaxed text-muted-foreground">{d.description}</p>
        ) : null}
        <p className="text-xs text-muted-foreground">
          <strong className="text-foreground tabular-nums">
            {d.entry_count.toLocaleString('en-IN')}
          </strong>{' '}
          entries
        </p>
        {d.attribution ? (
          <p className="border-t pt-2 text-[11px] leading-relaxed text-muted-foreground">
            {d.attribution}
          </p>
        ) : null}
        <Link
          to="/dict/$slug"
          params={{ slug: d.slug }}
          search={{}}
          className="inline-flex items-center gap-1 self-start text-xs font-medium text-primary hover:underline"
        >
          Browse in book order <ArrowRightIcon className="size-3.5" aria-hidden />
        </Link>
      </PopoverContent>
    </Popover>
  );
}

const triggerClass =
  'h-8 gap-1.5 rounded-full border-border/70 bg-background/60 px-3 text-xs font-medium shadow-none hover:border-primary/50 hover:bg-primary/5 data-popup-open:border-primary/60 data-popup-open:bg-primary/10 dark:bg-transparent';

export function DictionaryPicker({
  dictionaries,
  selected,
  prefs
}: {
  dictionaries: Dictionary[];
  /** Resolved selection (known slugs only). */
  selected: string[];
  prefs: SearchPrefs;
}) {
  const [filter, setFilter] = useState('');
  const total = dictionaries.length;
  const isAll = prefs.dicts === 'all' || (total > 0 && selected.length === total);
  const selectedSet = new Set(selected);

  const commit = (next: string[]) => {
    if (next.length === 0) return; // at least one dictionary stays selected
    setSearchPrefs({ dicts: next.length === total ? 'all' : next });
  };
  const toggle = (slug: string) =>
    commit(
      selectedSet.has(slug)
        ? selected.filter((s) => s !== slug)
        : dictionaries.filter((d) => d.slug === slug || selectedSet.has(d.slug)).map((d) => d.slug)
    );
  const toggleGroup = (group: Dictionary[], on: boolean) => {
    const slugs = new Set(group.map((d) => d.slug));
    commit(
      dictionaries
        .filter((d) => (slugs.has(d.slug) ? on : selectedSet.has(d.slug)))
        .map((d) => d.slug)
    );
  };

  const f = filter.trim().toLowerCase();
  const visible = f
    ? dictionaries.filter(
        (d) =>
          d.name.toLowerCase().includes(f) ||
          d.short_name.toLowerCase().includes(f) ||
          d.slug.includes(f)
      )
    : dictionaries;
  const byKind = groupByKind(visible);

  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="outline"
            size="sm"
            type="button"
            className={triggerClass}
            aria-label={`Dictionaries: ${isAll ? 'all' : selected.length} selected`}
          />
        }
      >
        <LibraryBigIcon className="size-3.5 text-primary" aria-hidden />
        <span>Dictionaries</span>
        <span
          className={cn(
            'grid h-5 min-w-5 place-items-center rounded-full px-1.5 text-[11px] font-bold tabular-nums',
            'bg-primary text-primary-foreground'
          )}
        >
          {isAll ? 'All' : selected.length}
        </span>
      </PopoverTrigger>
      <PopoverContent
        align="start"
        className="w-[min(26rem,calc(100vw-1.5rem))] gap-0 overflow-hidden rounded-xl p-0"
      >
        <div className="flex items-center justify-between gap-2 border-b bg-muted/40 px-3 py-2.5">
          <div>
            <p className="text-sm font-semibold">Dictionaries</p>
            <p className="text-xs text-muted-foreground">
              {isAll ? `All ${total}` : `${selected.length} of ${total}`} searched · saved on this
              device
            </p>
          </div>
          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="xs"
              title="Monier-Williams + Apte"
              onClick={() => setSearchPrefs({ dicts: [...DEFAULT_DICTS] })}
            >
              <RotateCcwIcon aria-hidden /> Default
            </Button>
            <Button variant="ghost" size="xs" onClick={() => setSearchPrefs({ dicts: 'all' })}>
              <CheckCheckIcon aria-hidden /> All
            </Button>
          </div>
        </div>
        <div className="border-b px-3 py-2">
          <div className="flex items-center gap-2 rounded-md bg-muted/60 px-2">
            <SearchIcon className="size-3.5 text-muted-foreground" aria-hidden />
            <input
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Filter dictionaries…"
              aria-label="Filter dictionaries"
              className="h-8 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
            />
          </div>
        </div>
        <div className="flex max-h-[min(60vh,26rem)] flex-col gap-3 overflow-y-auto overscroll-contain p-2">
          {dictionaries.length === 0 ? (
            <div
              className="flex flex-col gap-1.5 p-1"
              role="status"
              aria-label="Loading dictionaries"
            >
              {Array.from({ length: 6 }).map((_, i) => (
                <Skeleton key={i} className="h-10 rounded-lg" />
              ))}
            </div>
          ) : byKind.size === 0 ? (
            <p className="px-2 py-6 text-center text-sm text-muted-foreground">No match</p>
          ) : null}
          {DICT_KIND_ORDER.filter((k) => byKind.has(k)).map((kind) => {
            const group = byKind.get(kind)!;
            const on = group.filter((d) => selectedSet.has(d.slug)).length;
            return (
              <div key={kind} className="flex flex-col gap-0.5">
                <div className="flex items-center justify-between px-2 pb-1">
                  <p className="text-[11px] font-semibold tracking-wider text-muted-foreground uppercase">
                    {DICT_KIND_LABELS[kind]}
                  </p>
                  <button
                    type="button"
                    className="text-[11px] font-medium text-primary hover:underline"
                    onClick={() => toggleGroup(group, on < group.length)}
                  >
                    {on < group.length ? 'Select all' : 'Clear'}
                  </button>
                </div>
                {group.map((d) => {
                  const checked = selectedSet.has(d.slug);
                  const last = checked && selected.length === 1;
                  return (
                    <div
                      key={d.slug}
                      className={cn(
                        'flex items-center gap-1 rounded-lg pr-1 transition-colors hover:bg-muted',
                        checked && 'bg-primary/5'
                      )}
                    >
                      <Label className="flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 py-1.5 pl-2 text-sm font-normal">
                        <Checkbox
                          checked={checked}
                          disabled={last}
                          onCheckedChange={() => toggle(d.slug)}
                          aria-label={d.name}
                        />
                        <span className="min-w-0 flex-1">
                          <span className="flex items-center gap-1.5">
                            <span className="font-devanagari font-semibold">{d.short_name}</span>
                            <DirectionTag dictionary={d} />
                          </span>
                          <span className="block truncate text-xs text-muted-foreground">
                            {d.name}
                          </span>
                        </span>
                        <span className="text-[11px] text-muted-foreground tabular-nums">
                          {d.entry_count.toLocaleString('en-IN')}
                        </span>
                      </Label>
                      <DictionaryInfo dictionary={d} />
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </PopoverContent>
    </Popover>
  );
}

export function SearchOptions({ prefs, mode }: { prefs: SearchPrefs; mode: 'sa' | 'en' }) {
  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="outline"
            size="sm"
            type="button"
            className={triggerClass}
            aria-label="Search options"
          />
        }
      >
        <SlidersHorizontalIcon className="size-3.5 text-primary" aria-hidden />
        <span>Options</span>
        {prefs.forms ? (
          <span className="size-1.5 rounded-full bg-primary" aria-label="inflections on" />
        ) : null}
      </PopoverTrigger>
      <PopoverContent
        align="start"
        className="w-[min(22rem,calc(100vw-1.5rem))] gap-0 overflow-hidden rounded-xl p-0"
      >
        <div className="border-b bg-muted/40 px-3 py-2.5">
          <p className="text-sm font-semibold">Search options</p>
          <p className="text-xs text-muted-foreground">Saved on this device</p>
        </div>
        <div className="flex flex-col gap-1 p-2">
          <Label
            className={cn(
              'flex cursor-pointer items-start gap-3 rounded-lg p-2 font-normal hover:bg-muted',
              mode === 'en' && 'opacity-60'
            )}
          >
            <Checkbox
              className="mt-0.5"
              checked={prefs.forms}
              onCheckedChange={(v) => setSearchPrefs({ forms: v === true })}
            />
            <span className="flex flex-col gap-0.5">
              <span className="flex items-center gap-1.5 text-sm font-medium">
                <SparklesIcon className="size-3.5 text-amber-500" aria-hidden />
                Find inflected forms
              </span>
              <span className="text-xs leading-relaxed text-muted-foreground">
                Analyse declined and conjugated words —{' '}
                <span className="font-devanagari">रामेण</span> →{' '}
                <span className="font-devanagari">राम</span>,{' '}
                <span className="font-devanagari">गतेन</span> →{' '}
                <span className="font-devanagari">गम्</span> — with their paradigm tables.
                {mode === 'en' ? ' (Sanskrit mode only.)' : null}
              </span>
            </span>
          </Label>
        </div>
        <div className="flex flex-col gap-1.5 border-t bg-muted/30 px-3 py-2.5 text-xs text-muted-foreground">
          <p className="flex items-center gap-1.5 font-medium text-foreground">
            <KeyboardIcon className="size-3.5" aria-hidden /> Typing
          </p>
          <p>
            In Sanskrit mode Latin letters turn into Devanagari as you type (
            <span className="font-mono">agni</span> → <span className="font-devanagari">अग्नि</span>
            ). <Kbd>Alt</Kbd>+<Kbd>X</Kbd> switches it off for the current search.
          </p>
          <p>
            <Kbd>/</Kbd> focuses the search box from anywhere.
          </p>
        </div>
      </PopoverContent>
    </Popover>
  );
}
