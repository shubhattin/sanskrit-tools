import { useEffect, useRef, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { CheckIcon, ChevronRightIcon, CopyIcon, FileTextIcon } from 'lucide-react';
import { dictionariesQuery } from '@/lib/api/client';
import { agentContextQuery } from '@/lib/agent/client';
import type { AgentContext } from '@/lib/agent/types';
import { resolveSelectedSlugs, useSearchPrefs } from '@/lib/search-prefs';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog';
import { Skeleton } from '@/components/ui/skeleton';

/** The reader's preferred dictionary slugs — the same ones the chat sends with messages. */
export function usePreferredDictSlugs(): string[] {
  const prefs = useSearchPrefs();
  const dictsQuery = useQuery(dictionariesQuery());
  if (!dictsQuery.data) return prefs.dicts === 'all' ? [] : prefs.dicts;
  return resolveSelectedSlugs(prefs, dictsQuery.data);
}

/** Header button opening the Context dialog. Self-contained: works in any agent view. */
export function ContextButton() {
  const [open, setOpen] = useState(false);
  const dicts = usePreferredDictSlugs();
  return (
    <>
      <Button
        variant="outline"
        size="sm"
        type="button"
        aria-label="Context: what the model receives"
        className="gap-1.5 rounded-full text-xs font-medium"
        onClick={() => setOpen(true)}
      >
        <FileTextIcon className="size-3.5 text-primary" aria-hidden />
        <span className="max-sm:hidden">Context</span>
      </Button>
      <ContextDialog dicts={dicts} open={open} onOpenChange={setOpen} />
    </>
  );
}

function formatJson(value: unknown): string {
  try {
    return JSON.stringify(value, null, 2) ?? String(value);
  } catch {
    return String(value);
  }
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);
  return (
    <Button
      variant="ghost"
      size="xs"
      type="button"
      aria-label={copied ? 'Copied' : 'Copy system prompt'}
      className="gap-1 text-muted-foreground"
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
    >
      {copied ? <CheckIcon className="size-3" /> : <CopyIcon className="size-3" />}
      {copied ? 'Copied' : 'Copy'}
    </Button>
  );
}

function ToolRow({
  name,
  description,
  parameters,
  badge
}: {
  name: string;
  description?: string;
  parameters: unknown;
  badge?: string;
}) {
  return (
    <Collapsible className="rounded-lg border border-border/70">
      <CollapsibleTrigger className="group flex w-full cursor-pointer items-center gap-2 px-2.5 py-2 text-left outline-none focus-visible:ring-3 focus-visible:ring-ring/50">
        <ChevronRightIcon className="size-3.5 shrink-0 text-muted-foreground transition-transform group-aria-expanded:rotate-90" />
        <code className="shrink-0 font-mono text-xs font-semibold text-foreground">{name}</code>
        {badge ? (
          <Badge variant="secondary" className="shrink-0 text-[10px]">
            {badge}
          </Badge>
        ) : null}
        {description ? (
          <span className="min-w-0 flex-1 truncate text-xs text-muted-foreground">
            {description}
          </span>
        ) : null}
      </CollapsibleTrigger>
      <CollapsibleContent className="px-2.5 pb-2.5">
        <pre className="max-h-64 overflow-auto rounded-md bg-muted/40 p-2.5 font-mono text-[11px] break-words whitespace-pre-wrap text-foreground">
          {formatJson(parameters)}
        </pre>
      </CollapsibleContent>
    </Collapsible>
  );
}

export function ContextBody({ context }: { context: AgentContext }) {
  const chars = context.system_prompt.length;
  const approxTokens = Math.round(chars / 4);
  return (
    <div className="grid min-w-0 grid-cols-1 gap-5 md:grid-cols-2">
      <section className="flex min-w-0 flex-col gap-2">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-sm font-semibold text-foreground">System prompt</h2>
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground tabular-nums">
              {chars.toLocaleString()} chars · ≈{approxTokens.toLocaleString()} tokens
            </span>
            <CopyButton text={context.system_prompt} />
          </div>
        </div>
        <pre className="max-h-96 overflow-auto rounded-lg border border-border/70 bg-muted/40 p-3 font-mono text-xs leading-relaxed break-words whitespace-pre-wrap text-foreground md:max-h-[58dvh]">
          {context.system_prompt}
        </pre>
      </section>

      <div className="flex min-w-0 flex-col gap-5">
        <section className="flex min-w-0 flex-col gap-2">
          <h2 className="text-sm font-semibold text-foreground">
            Tools{' '}
            <span className="font-normal text-muted-foreground tabular-nums">
              ({context.tools.length + context.server_tools.length})
            </span>
          </h2>
          <ul className="flex min-w-0 flex-col gap-1.5">
            {context.tools.map((tool) => (
              <li key={tool.name} className="min-w-0">
                <ToolRow
                  name={tool.name}
                  description={tool.description}
                  parameters={tool.parameters}
                />
              </li>
            ))}
            {context.server_tools.map((tool) => (
              <li key={tool.type} className="min-w-0">
                <ToolRow name={tool.type} parameters={tool.parameters} badge="Run by OpenRouter" />
              </li>
            ))}
          </ul>
        </section>

        <section className="flex min-w-0 flex-col gap-1.5 border-t border-border/70 pt-3 text-xs text-muted-foreground">
          <p>Up to {context.max_tool_calls} tool calls per turn.</p>
          <ul className="flex min-w-0 flex-col gap-0.5">
            {context.models.map((model) => (
              <li
                key={model.id}
                className="flex min-w-0 flex-wrap items-baseline gap-x-1.5 tabular-nums"
              >
                <code className="font-mono font-semibold text-foreground">{model.id}</code>
                <span aria-hidden>→</span>
                <span className="min-w-0 break-all">{model.provider_id}</span>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}

export function ContextDialog({
  dicts,
  open,
  onOpenChange
}: {
  dicts: string[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const contextQuery = useQuery({ ...agentContextQuery(dicts), enabled: open });
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[85dvh] flex-col sm:max-w-5xl">
        <DialogHeader>
          <DialogTitle>Context</DialogTitle>
          <DialogDescription>Exactly what the model receives on every turn.</DialogDescription>
        </DialogHeader>
        <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain pr-1">
          {contextQuery.isPending ? (
            <div className="flex flex-col gap-3" aria-label="Loading context">
              <Skeleton className="h-5 w-32" />
              <Skeleton className="h-48 w-full" />
              <Skeleton className="h-5 w-24" />
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
            </div>
          ) : contextQuery.isError ? (
            <div className="flex flex-col gap-2 rounded-xl border border-destructive/30 bg-destructive/5 p-3 text-sm">
              <p className="text-destructive">
                {(contextQuery.error as Error)?.message || 'Could not load the context.'}
              </p>
              <Button
                size="xs"
                variant="outline"
                className="self-start"
                onClick={() => void contextQuery.refetch()}
              >
                Retry
              </Button>
            </div>
          ) : contextQuery.data ? (
            <ContextBody context={contextQuery.data} />
          ) : null}
        </div>
      </DialogContent>
    </Dialog>
  );
}
