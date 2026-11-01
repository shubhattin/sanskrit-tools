import { useQuery } from '@tanstack/react-query';
import { BookOpenIcon, InfoIcon } from 'lucide-react';
import { dictionariesQuery } from '@/lib/api/client';
import { AGENT_MODELS } from '@/lib/agent/models';
import type { AgentModelId, AgentUIMessage } from '@/lib/agent/types';
import { resolveSelectedSlugs, useSearchPrefs } from '@/lib/search-prefs';
import { DictionaryPicker } from '@/components/dict/filter';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { MobileHistoryButton } from './agent-sidebar';
import { chatTotals, formatCost, usageLine } from './chat-cost';
import { ContextButton } from './context-dialog';

const CAPABILITIES = [
  'Dictionary lookup across lexicons, thesauri and English meanings',
  'Inflected-form analysis (rāmeṇa → rāma, bhavati → bhū)',
  'Declension, conjugation and kṛdanta tables',
  'Dhātupāṭha roots and the works discussing them',
  'Aṣṭādhyāyī sūtras with commentaries (Kāśikā, Nyāsa, …)',
  'Book-order browsing of any dictionary',
  'Web search when the answer needs broader context'
];

function DictionariesButton() {
  const prefs = useSearchPrefs();
  const dictsQuery = useQuery(dictionariesQuery());
  const dictionaries = dictsQuery.data ?? [];
  const selected = dictsQuery.data
    ? resolveSelectedSlugs(prefs, dictsQuery.data)
    : prefs.dicts === 'all'
      ? []
      : prefs.dicts;
  const isAll =
    prefs.dicts === 'all' || (dictionaries.length > 0 && selected.length === dictionaries.length);
  const bySlug = new Map(dictionaries.map((d) => [d.slug, d]));

  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="outline"
            size="sm"
            type="button"
            aria-label={`Dictionaries: ${isAll ? 'all' : selected.length} selected`}
            className="gap-1.5 rounded-full text-xs font-medium"
          />
        }
      >
        <BookOpenIcon className="size-3.5 text-primary" aria-hidden />
        <span className="max-sm:hidden">Dictionaries</span>
        <span className="grid h-5 min-w-5 place-items-center rounded-full bg-foreground px-1.5 text-[11px] font-bold text-background tabular-nums">
          {isAll ? 'All' : selected.length}
        </span>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        className="w-[min(24rem,calc(100vw-1.5rem))] gap-3 rounded-xl p-4"
      >
        <div className="flex flex-col gap-1">
          <p className="text-sm font-semibold">Dictionaries</p>
          <p className="text-xs leading-relaxed text-muted-foreground">
            The agent searches these by default. It can look in any other dictionary when it needs
            to or when you ask.
          </p>
        </div>
        {dictionaries.length === 0 ? (
          <p className="text-xs text-muted-foreground">Loading…</p>
        ) : (
          <ul className="flex max-h-48 flex-col gap-1 overflow-y-auto overscroll-contain">
            {(isAll
              ? dictionaries
              : selected.map((slug) => bySlug.get(slug)).filter((d) => d !== undefined)
            ).map((d) => (
              <li key={d.slug} className="flex items-center gap-2 text-xs">
                <span className="shrink-0 rounded-md bg-muted px-1.5 py-0.5 font-devanagari font-bold">
                  {d.short_name}
                </span>
                <span className="min-w-0 flex-1 truncate text-muted-foreground">{d.name}</span>
              </li>
            ))}
          </ul>
        )}
        <div className="border-t pt-3">
          <DictionaryPicker dictionaries={dictionaries} selected={selected} prefs={prefs} />
        </div>
      </PopoverContent>
    </Popover>
  );
}

function InfoButton({ messages }: { messages: AgentUIMessage[] }) {
  const totals = chatTotals(messages);
  const tokens = usageLine({
    inputTokens: totals.inputTokens,
    outputTokens: totals.outputTokens,
    cachedInputTokens: totals.cachedInputTokens,
    reasoningTokens: totals.reasoningTokens
  });
  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="ghost"
            size="icon-sm"
            type="button"
            aria-label="What the agent can do"
            className="shrink-0 text-muted-foreground"
          />
        }
      >
        <InfoIcon className="size-4" />
      </PopoverTrigger>
      <PopoverContent
        align="end"
        className="w-[min(22rem,calc(100vw-1.5rem))] gap-3 rounded-xl p-4"
      >
        <div className="flex flex-col gap-1.5">
          <p className="text-sm font-semibold">What the agent can do</p>
          <ul className="flex flex-col gap-1 text-xs leading-relaxed text-muted-foreground">
            {CAPABILITIES.map((capability) => (
              <li key={capability} className="flex gap-1.5">
                <span aria-hidden className="text-primary">
                  •
                </span>
                <span>{capability}</span>
              </li>
            ))}
          </ul>
        </div>
        <div className="flex flex-col gap-1 border-t pt-3 text-xs">
          <p className="font-semibold">Cost of this chat</p>
          {totals.replies === 0 ? (
            <p className="text-muted-foreground">No replies yet.</p>
          ) : (
            <>
              <p>
                {formatCost(totals.total)} over {totals.replies - totals.untracked}{' '}
                {totals.replies - totals.untracked === 1 ? 'reply' : 'replies'}
              </p>
              {tokens ? <p className="text-muted-foreground tabular-nums">{tokens}</p> : null}
              {totals.webSearches > 0 ? (
                <p className="text-muted-foreground tabular-nums">
                  {totals.webSearches} web {totals.webSearches === 1 ? 'search' : 'searches'}
                </p>
              ) : null}
              {totals.untracked > 0 ? (
                <p className="text-muted-foreground">
                  {totals.untracked} older {totals.untracked === 1 ? 'reply has' : 'replies have'}{' '}
                  no cost recorded.
                </p>
              ) : null}
              <p className="text-muted-foreground/70">
                As charged by OpenRouter, cache discounts included.
              </p>
            </>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}

export function ChatHeader({
  title,
  model,
  messages
}: {
  title: string;
  model: AgentModelId;
  messages: AgentUIMessage[];
}) {
  return (
    <header className="flex items-center gap-2 border-b border-border/70 px-3 py-2">
      <MobileHistoryButton />
      <div className="min-w-0 flex-1">
        <h1 className="truncate text-sm font-semibold text-foreground">{title}</h1>
        <p className="truncate text-xs text-muted-foreground">{AGENT_MODELS[model].label}</p>
      </div>
      <DictionariesButton />
      <ContextButton />
      <InfoButton messages={messages} />
    </header>
  );
}
