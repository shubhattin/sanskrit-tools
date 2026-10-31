import { useEffect, useRef, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { SearchIcon } from 'lucide-react';
import { Command, CommandItem, CommandList } from '@/components/ui/command';
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { suggestQuery } from '@/lib/api/client';
import type { Dictionary, SearchMode } from '@/lib/api/types';
import { useLipiInput } from '@/components/lipi/typing';
import { LipiToggle } from '@/components/lipi/toggle';
import { SearchSettingsPopover } from '@/components/dict/filter';
import { useDisplayText } from '@/components/script/display-script';

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
  dicts: string[];
}

export function SearchBar({
  value,
  dictionaries,
  autoFocus,
  onSubmit
}: {
  value: SearchBarValue;
  dictionaries: Dictionary[];
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
  const inputRef = useRef<HTMLInputElement>(null);
  const boxRef = useRef<HTMLDivElement>(null);

  const lipiProps = useLipiInput(text, setText);
  const debounced = useDebounced(text, 200);
  const suggest = useQuery({
    ...suggestQuery(debounced.trim()),
    enabled: open && debounced.trim().length > 0
  });

  // `/` focuses search from anywhere (unless already typing in a field).
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const typing = target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA');
      if (e.key === '/' && !typing) {
        e.preventDefault();
        inputRef.current?.focus();
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
    inputRef.current?.blur();
    onSubmit({ q, mode: value.mode, dicts: value.dicts });
  };

  const suggestions = suggest.data ?? [];

  return (
    <div ref={boxRef} className="relative">
      <form
        role="search"
        onSubmit={(e) => {
          e.preventDefault();
          submit(text.trim());
        }}
      >
        <InputGroup className="h-12 rounded-xl bg-background text-base shadow-xs">
          <InputGroupAddon>
            <SearchIcon aria-hidden />
          </InputGroupAddon>
          <InputGroupInput
            {...lipiProps}
            ref={inputRef}
            autoFocus={autoFocus}
            placeholder="Search — अग्नि, agni, dharma…"
            aria-label="Search dictionaries"
            autoComplete="off"
            spellCheck={false}
            onFocus={() => setOpen(true)}
            className="font-devanagari text-lg"
          />
          {!text ? (
            <InputGroupAddon align="inline-end">
              <Kbd>/</Kbd>
            </InputGroupAddon>
          ) : null}
          <InputGroupAddon align="inline-end">
            <div
              role="group"
              aria-label="Search mode"
              className="flex items-center gap-0.5 rounded-lg border p-0.5"
            >
              {(
                [
                  { v: 'sa', label: 'सं', title: 'Sanskrit headword' },
                  { v: 'en', label: 'En', title: 'English meaning' }
                ] as const
              ).map((m) => (
                <Button
                  key={m.v}
                  type="button"
                  variant={value.mode === m.v ? 'secondary' : 'ghost'}
                  size="sm"
                  className="h-6 px-2 text-xs"
                  title={m.title}
                  aria-pressed={value.mode === m.v}
                  onClick={() => onSubmit({ q: value.q, mode: m.v, dicts: value.dicts })}
                >
                  {m.label}
                </Button>
              ))}
            </div>
            <LipiToggle />
            <SearchSettingsPopover
              dictionaries={dictionaries}
              selected={value.dicts}
              onChange={(dicts) => onSubmit({ q: value.q, mode: value.mode, dicts })}
            />
          </InputGroupAddon>
        </InputGroup>
      </form>
      {open && debounced.trim().length > 0 ? (
        <div className="absolute inset-x-0 top-full z-50 mt-1.5 overflow-hidden rounded-xl border bg-popover shadow-lg">
          <Command label="Suggestions">
            <CommandList>
              {suggest.isPending ? (
                <p className="px-3 py-4 text-sm text-muted-foreground">Loading…</p>
              ) : suggestions.length === 0 ? (
                <p className="px-3 py-4 text-sm text-muted-foreground">No suggestions</p>
              ) : (
                suggestions.map((s) => (
                  <SuggestRow
                    key={`${s.headword}-${s.slp1}`}
                    headword={s.headword}
                    count={s.count}
                    onSelect={() => submit(s.headword)}
                  />
                ))
              )}
            </CommandList>
          </Command>
        </div>
      ) : null}
    </div>
  );
}

function SuggestRow({
  headword,
  count,
  onSelect
}: {
  headword: string;
  count: number;
  onSelect: () => void;
}) {
  const display = useDisplayText(headword);
  return (
    <CommandItem value={headword} onSelect={onSelect} className="font-devanagari text-base">
      <span className="min-w-0 flex-1 truncate">{display}</span>
      <span className="text-xs text-muted-foreground tabular-nums">{count}</span>
    </CommandItem>
  );
}
