import { useEffect, useRef, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { ArrowRightIcon, CornerDownLeftIcon, SearchIcon, XIcon } from 'lucide-react';
import { Command, CommandItem, CommandList } from '@/components/ui/command';
import { Kbd } from '@/components/ui/kbd';
import { Skeleton } from '@/components/ui/skeleton';
import { suggestQuery } from '@/lib/api/client';
import type { Dictionary, SearchMode } from '@/lib/api/types';
import type { SearchPrefs } from '@/lib/search-prefs';
import { useLipiInput } from '@/components/lipi/typing';
import { DictionaryPicker, SearchOptions } from '@/components/dict/filter';
import { cn } from '@/lib/utils';

function useDebounced(value: string, delayMs: number): string {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(t);
  }, [value, delayMs]);
  return debounced;
}

export interface SearchBarValue {
  q: string;
  mode: SearchMode;
}

const MODES = [
  { v: 'sa', label: 'संस्कृतम्', short: 'सं', title: 'Search Sanskrit headwords' },
  { v: 'en', label: 'English', short: 'En', title: 'Search English meanings' }
] as const;

export function SearchBar({
  value,
  dictionaries,
  selected,
  prefs,
  autoFocus,
  onSubmit
}: {
  value: SearchBarValue;
  dictionaries: Dictionary[];
  selected: string[];
  prefs: SearchPrefs;
  autoFocus?: boolean;
  onSubmit: (next: SearchBarValue) => void;
}) {
  const [text, setText] = useState(value.q);
  // Sync local text when the URL changes (e.g. a[data-q] navigation).
  // Render-time adjustment: the sanctioned pattern for syncing state to props.
  const [syncedQ, setSyncedQ] = useState(value.q);
  if (value.q !== syncedQ) {
    setSyncedQ(value.q);
    setText(value.q);
  }
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const inputRef = useRef<HTMLInputElement>(null);
  const boxRef = useRef<HTMLDivElement>(null);

  const { props: lipiProps, enabled: lipiOn } = useLipiInput(text, setText, value.mode === 'sa');
  const debounced = useDebounced(text, 180);
  const suggestOn = value.mode === 'sa' && open && debounced.trim().length > 0;
  const suggest = useQuery({ ...suggestQuery(debounced.trim()), enabled: suggestOn });
  const suggestions = suggest.data ?? [];

  // `/` focuses search from anywhere (unless already typing in a field).
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const typing =
        target &&
        (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable);
      if (e.key === '/' && !typing) {
        e.preventDefault();
        inputRef.current?.focus();
        inputRef.current?.select();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // Close the dropdown on outside click.
  useEffect(() => {
    const onDown = (e: PointerEvent) => {
      if (boxRef.current && !boxRef.current.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener('pointerdown', onDown);
    return () => window.removeEventListener('pointerdown', onDown);
  }, []);

  const submit = (q: string) => {
    setOpen(false);
    setActive(-1);
    inputRef.current?.blur();
    onSubmit({ q, mode: value.mode });
  };

  const showSuggest = suggestOn && (suggest.isPending || suggestions.length > 0);

  return (
    <div ref={boxRef} className="relative">
      <form
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          if (showSuggest && active >= 0 && suggestions[active]) {
            submit(suggestions[active].headword);
          } else submit(text.trim());
        }}
        className={cn(
          'group/search relative rounded-2xl border bg-card/90 shadow-lg shadow-black/5 backdrop-blur-sm transition-all duration-200',
          'focus-within:border-primary/50 focus-within:ring-4 focus-within:shadow-primary/10 focus-within:ring-primary/10',
          'dark:shadow-black/30'
        )}
      >
        <div className="flex items-center gap-2 px-3 sm:px-4">
          <SearchIcon
            className="size-5 shrink-0 text-muted-foreground transition-colors group-focus-within/search:text-primary"
            aria-hidden
          />
          <input
            {...lipiProps}
            onKeyDown={(e) => {
              if (showSuggest && suggestions.length > 0) {
                if (e.key === 'ArrowDown') {
                  e.preventDefault();
                  setActive((i) => (i + 1) % suggestions.length);
                  return;
                }
                if (e.key === 'ArrowUp') {
                  e.preventDefault();
                  setActive((i) => (i <= 0 ? suggestions.length - 1 : i - 1));
                  return;
                }
              }
              if (e.key === 'Escape') {
                setOpen(false);
                return;
              }
              lipiProps.onKeyDown(e);
            }}
            onChange={(e) => {
              setActive(-1);
              setOpen(true);
              lipiProps.onChange(e);
            }}
            ref={inputRef}
            autoFocus={autoFocus}
            placeholder={
              value.mode === 'sa' ? 'agni, dharma, अग्नि…' : 'English meaning — fire, king…'
            }
            aria-label="Search dictionaries"
            autoComplete="off"
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            enterKeyHint="search"
            onFocus={() => setOpen(true)}
            className="h-14 min-w-0 flex-1 bg-transparent font-devanagari text-lg outline-none placeholder:text-muted-foreground/70 sm:h-16 sm:text-xl"
          />
          {text ? (
            <button
              type="button"
              aria-label="Clear search"
              className="grid size-8 shrink-0 place-items-center rounded-full text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              onClick={() => {
                setText('');
                inputRef.current?.focus();
              }}
            >
              <XIcon className="size-4" aria-hidden />
            </button>
          ) : (
            <Kbd className="max-sm:hidden">/</Kbd>
          )}
          <button
            type="submit"
            aria-label="Search"
            className="grid size-9 shrink-0 place-items-center rounded-xl bg-gradient-to-br from-primary to-teal-500 text-primary-foreground shadow-md shadow-primary/25 transition-all hover:scale-105 hover:shadow-primary/40 active:scale-95 sm:size-10"
          >
            <ArrowRightIcon className="size-4 sm:size-5" aria-hidden />
          </button>
        </div>
        <div className="flex flex-wrap items-center gap-1.5 border-t border-dashed px-2.5 py-2 sm:px-3">
          <div
            role="radiogroup"
            aria-label="Search mode"
            className="relative grid grid-cols-2 rounded-full bg-muted p-0.5"
          >
            <span
              aria-hidden
              className={cn(
                'absolute inset-y-0.5 left-0.5 w-[calc(50%-2px)] rounded-full bg-background shadow-sm ring-1 ring-border transition-transform duration-300 ease-out',
                value.mode === 'en' && 'translate-x-full'
              )}
            />
            {MODES.map((m) => (
              <button
                key={m.v}
                type="button"
                role="radio"
                aria-checked={value.mode === m.v}
                title={m.title}
                onClick={() => onSubmit({ q: text.trim() || value.q, mode: m.v })}
                className={cn(
                  'relative z-10 h-7 px-3 text-xs font-medium transition-colors',
                  m.v === 'sa' && 'font-devanagari',
                  value.mode === m.v
                    ? 'text-foreground'
                    : 'text-muted-foreground hover:text-foreground'
                )}
              >
                <span className="max-sm:hidden">{m.label}</span>
                <span className="sm:hidden">{m.short}</span>
              </button>
            ))}
          </div>
          <DictionaryPicker dictionaries={dictionaries} selected={selected} prefs={prefs} />
          <SearchOptions prefs={prefs} mode={value.mode} />
          <span
            className="ml-auto hidden items-center gap-1.5 pr-1 text-[11px] leading-none text-muted-foreground sm:flex"
            title="Alt+X toggles Devanagari typing"
          >
            {lipiOn ? (
              <>
                <span className="inline-flex h-5 items-center gap-1 rounded-md bg-muted px-1.5 leading-none">
                  <span className="font-mono text-[11px]">a</span>
                  <span aria-hidden>→</span>
                  <span className="font-devanagari text-xs">अ</span>
                </span>
                <span className="leading-none">typing</span>
              </>
            ) : (
              'Latin typing'
            )}
            <Kbd className="ml-1">Alt X</Kbd>
          </span>
        </div>
      </form>
      {showSuggest ? (
        <div className="absolute inset-x-0 top-full z-50 mt-2 animate-in overflow-hidden rounded-xl border bg-popover shadow-xl duration-150 fade-in-0 slide-in-from-top-1">
          <Command
            label="Suggestions"
            shouldFilter={false}
            value={suggestions[active]?.headword ?? ''}
          >
            <CommandList>
              {suggest.isPending ? (
                <div
                  className="flex flex-col gap-1 p-2"
                  role="status"
                  aria-label="Loading suggestions"
                >
                  {[70, 55, 62, 45].map((w) => (
                    <div key={w} className="flex items-center gap-2 px-2 py-1.5">
                      <Skeleton className="size-3.5 rounded-full" />
                      <Skeleton className="h-4" style={{ width: `${w}%` }} />
                    </div>
                  ))}
                </div>
              ) : (
                suggestions.map((s, i) => (
                  <CommandItem
                    key={`${s.headword}-${s.slp1}`}
                    value={s.headword}
                    onSelect={() => submit(s.headword)}
                    onMouseEnter={() => setActive(i)}
                    className="font-devanagari text-base"
                  >
                    <SearchIcon className="size-3.5 text-muted-foreground" aria-hidden />
                    <span className="min-w-0 flex-1 truncate">{s.headword}</span>
                    {i === active ? (
                      <CornerDownLeftIcon className="size-3.5 text-muted-foreground" aria-hidden />
                    ) : null}
                    <span className="text-xs text-muted-foreground tabular-nums">{s.count}</span>
                  </CommandItem>
                ))
              )}
            </CommandList>
          </Command>
        </div>
      ) : null}
    </div>
  );
}
