import { useState, type ReactNode } from 'react';
import { useQuery } from '@tanstack/react-query';
import { ChevronDownIcon, type LucideIcon } from 'lucide-react';
import { dictionariesQuery } from '@/lib/api/client';
import { cn } from '@/lib/utils';
import {
  TraceCollapsible,
  TraceCollapsibleContent,
  TraceCollapsibleTrigger,
  TraceDescription,
  TraceIcon,
  TraceLine,
  TraceTitle
} from '@/components/ui/trace';
import { Button } from '@/components/ui/button';

/** Slug → short name (MW, Apte, …), falling back to the slug itself. */
export function useDictShortNames(): (slug: string) => string {
  const dicts = useQuery(dictionariesQuery());
  const map = new Map((dicts.data ?? []).map((d) => [d.slug, d.short_name]));
  return (slug: string) => map.get(slug) ?? slug;
}

export type ToolStepStatus = 'running' | 'done' | 'error' | 'interrupted';

export function toolStepStatus(state: string | undefined, streaming: boolean): ToolStepStatus {
  if (state === 'output-available') return 'done';
  if (state === 'output-error' || state === 'output-denied') return 'error';
  return streaming ? 'running' : 'interrupted';
}

function statusVerb(status: ToolStepStatus): 'running' | 'error' | undefined {
  if (status === 'running') return 'running';
  if (status === 'error' || status === 'interrupted') return 'error';
  return undefined;
}

/**
 * One compact collapsible trace row per tool call: spinner + present-tense label while
 * running, past tense when done, error styling on failure. Expanded: a compact view of
 * the output.
 */
export function ToolShell({
  icon: Icon,
  runningTitle,
  doneTitle,
  errorTitle,
  description,
  status,
  errorText,
  children
}: {
  icon: LucideIcon;
  runningTitle: string;
  doneTitle: string;
  errorTitle: string;
  description?: string;
  status: ToolStepStatus;
  errorText?: string;
  children?: ReactNode;
}) {
  if (status === 'running' || status === 'interrupted') {
    return (
      <TraceLine status={status === 'running' ? 'running' : 'error'}>
        <TraceIcon>
          <Icon />
        </TraceIcon>
        <TraceTitle>
          {status === 'running' ? runningTitle : `${doneTitle} — interrupted`}
        </TraceTitle>
        {description ? <TraceDescription>{description}</TraceDescription> : null}
      </TraceLine>
    );
  }

  const failed = status === 'error';
  return (
    <TraceCollapsible>
      <TraceCollapsibleTrigger>
        <TraceLine status={failed ? 'error' : undefined}>
          <TraceIcon>
            <Icon />
          </TraceIcon>
          <TraceTitle>{failed ? errorTitle : doneTitle}</TraceTitle>
          {description ? <TraceDescription>{description}</TraceDescription> : null}
        </TraceLine>
      </TraceCollapsibleTrigger>
      <TraceCollapsibleContent className="flex flex-col gap-2.5">
        {failed ? (
          <p className="text-xs text-destructive">{errorText || 'The tool call failed.'}</p>
        ) : (
          children
        )}
      </TraceCollapsibleContent>
    </TraceCollapsible>
  );
}

/** Long plain-text tool output: clamped with a Show more/less toggle. */
export function ClampedText({ text, lines = 6 }: { text: string; lines?: number }) {
  const [expanded, setExpanded] = useState(false);
  if (!text) return null;
  return (
    <div className="min-w-0">
      <p
        className="text-xs leading-relaxed break-words whitespace-pre-wrap text-muted-foreground"
        style={
          expanded
            ? undefined
            : {
                display: '-webkit-box',
                WebkitLineClamp: lines,
                WebkitBoxOrient: 'vertical',
                overflow: 'hidden'
              }
        }
      >
        {text}
      </p>
      {text.length > 220 ? (
        <Button
          variant="ghost"
          size="xs"
          className="mt-0.5 h-6 px-1.5 text-[11px]"
          onClick={() => setExpanded((v) => !v)}
          aria-expanded={expanded}
        >
          {expanded ? 'Show less' : 'Show more'}
          <ChevronDownIcon
            className={cn('size-3 transition-transform', expanded && 'rotate-180')}
          />
        </Button>
      ) : null}
    </div>
  );
}

export { statusVerb };
