import { useEffect, useRef, useState } from 'react';
import { CheckIcon, CopyIcon, InfoIcon } from 'lucide-react';
import { AGENT_MODELS } from '@/lib/agent/models';
import type { AgentMessageMetadata, AgentUIMessage } from '@/lib/agent/types';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Button } from '@/components/ui/button';

/** Dollars with enough precision for small amounts: $0.0042, $0.042, $1.23. */
export function formatCost(usd: number): string {
  if (usd === 0) return '$0';
  if (usd < 0.01) return `$${usd.toFixed(4)}`;
  if (usd < 1) return `$${usd.toFixed(3)}`;
  return `$${usd.toFixed(2)}`;
}

const compact = (n: number): string =>
  n >= 1_000_000
    ? `${(n / 1_000_000).toFixed(1)}M`
    : n >= 1_000
      ? `${(n / 1_000).toFixed(1)}k`
      : String(n);

export interface ChatTotals {
  replies: number;
  untracked: number;
  total: number;
  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;
  reasoningTokens: number;
  webSearches: number;
}

/** Sum of per-reply metadata over a chat (cost + tokens + web searches). */
export function chatTotals(messages: AgentUIMessage[]): ChatTotals {
  const totals: ChatTotals = {
    replies: 0,
    untracked: 0,
    total: 0,
    inputTokens: 0,
    outputTokens: 0,
    cachedInputTokens: 0,
    reasoningTokens: 0,
    webSearches: 0
  };
  for (const message of messages) {
    if (message.role !== 'assistant') continue;
    const meta = message.metadata;
    totals.replies += 1;
    if (meta?.cost === undefined) {
      totals.untracked += 1;
      continue;
    }
    totals.total += meta.cost;
    totals.inputTokens += meta.usage?.inputTokens ?? 0;
    totals.outputTokens += meta.usage?.outputTokens ?? 0;
    totals.cachedInputTokens += meta.usage?.cachedInputTokens ?? 0;
    totals.reasoningTokens += meta.usage?.reasoningTokens ?? 0;
    totals.webSearches += meta.webSearches ?? 0;
  }
  return totals;
}

export function usageLine(usage: AgentMessageMetadata['usage']): string | null {
  if (!usage) return null;
  const parts = [`${compact(usage.inputTokens ?? 0)} in`];
  if (usage.cachedInputTokens) parts.push(`${compact(usage.cachedInputTokens)} cached`);
  parts.push(`${compact(usage.outputTokens ?? 0)} out`);
  if (usage.reasoningTokens) parts.push(`${compact(usage.reasoningTokens)} reasoning`);
  return parts.join(' · ');
}

/** Under a finished reply: the model + cost info, beside the copy button. */
export function ReplyCost({ metadata }: { metadata: AgentMessageMetadata | undefined }) {
  if (metadata?.cost === undefined) return null;
  const tokens = usageLine(metadata.usage);
  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button
            variant="ghost"
            size="icon-xs"
            type="button"
            aria-label={`Reply cost ${formatCost(metadata.cost)}`}
            className="text-muted-foreground/60 hover:text-muted-foreground"
          />
        }
      >
        <InfoIcon className="size-3.5" />
      </PopoverTrigger>
      <PopoverContent align="start" className="flex w-60 flex-col gap-1 text-xs">
        <span>
          Cost {formatCost(metadata.cost)}
          {metadata.model ? ` · ${AGENT_MODELS[metadata.model]?.label ?? metadata.model}` : null}
        </span>
        {tokens ? <span className="text-muted-foreground tabular-nums">{tokens}</span> : null}
        {metadata.webSearches ? (
          <span className="text-muted-foreground tabular-nums">
            {metadata.webSearches} web {metadata.webSearches === 1 ? 'search' : 'searches'}
          </span>
        ) : null}
        <span className="text-muted-foreground/70">
          As charged by OpenRouter, cache discounts included.
        </span>
      </PopoverContent>
    </Popover>
  );
}

/** Under a finished reply: a copy button for the reply's markdown, beside the cost info. */
export function CopyReplyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);
  if (!text.trim()) return null;
  return (
    <button
      type="button"
      aria-label={copied ? 'Copied' : 'Copy reply as markdown'}
      title={copied ? 'Copied' : 'Copy reply as markdown'}
      onClick={() => {
        try {
          const clip = navigator.clipboard;
          if (!clip?.writeText) return;
          void clip
            .writeText(text)
            .then(() => {
              setCopied(true);
              window.clearTimeout(timer.current);
              timer.current = window.setTimeout(() => setCopied(false), 1500);
            })
            .catch(() => undefined);
        } catch {
          return;
        }
      }}
      className="inline-flex items-center justify-center rounded p-0.5 text-muted-foreground/60 transition-colors hover:text-muted-foreground"
    >
      {copied ? <CheckIcon className="size-3.5" /> : <CopyIcon className="size-3.5" />}
    </button>
  );
}
