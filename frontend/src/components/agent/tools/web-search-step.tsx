import { GlobeIcon } from 'lucide-react';
import { AgentMarkdown } from '../agent-markdown';
import type { AgentToolPart } from '../message-parts';
import { ToolShell, toolStepStatus } from './tool-shell';

const hostOf = (url: string): string => {
  try {
    return new URL(url).hostname.replace(/^www\./, '');
  } catch {
    return url;
  }
};

/** OpenAI web search: a factual summary with its source URLs, where it happened. */
export function WebSearchStep({
  part,
  streaming
}: {
  part: AgentToolPart<'web_search'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const query = part.input?.query ?? '';
  const count = done ? part.output.sources.length : 0;
  const description = [
    query,
    done && count > 0 ? `${count} ${count === 1 ? 'result' : 'results'}` : null
  ]
    .filter(Boolean)
    .join(' · ');

  return (
    <ToolShell
      icon={GlobeIcon}
      runningTitle="Searching the web"
      doneTitle="Searched the web"
      errorTitle="Web search failed"
      description={description || undefined}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? <WebSearchOutput part={part} /> : null}
    </ToolShell>
  );
}

export function WebSearchOutput({ part }: { part: AgentToolPart<'web_search'> }) {
  if (part.state !== 'output-available') return null;
  const { summary, sources } = part.output;
  return (
    <div className="flex min-w-0 flex-col gap-1.5">
      {summary ? (
        <AgentMarkdown className="text-xs leading-relaxed [&_p]:my-1">{summary}</AgentMarkdown>
      ) : null}
      {sources.length > 0 ? (
        <ul className="flex min-w-0 flex-col gap-1">
          {sources.map((source) => (
            <li key={source.url} className="min-w-0">
              <a
                href={source.url}
                target="_blank"
                rel="noreferrer"
                className="group flex min-w-0 items-center gap-2 text-xs no-underline"
              >
                <span className="min-w-0 flex-1 truncate text-foreground group-hover:underline">
                  {source.title || hostOf(source.url)}
                </span>
                <span className="shrink-0 text-muted-foreground/70">{hostOf(source.url)}</span>
              </a>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
