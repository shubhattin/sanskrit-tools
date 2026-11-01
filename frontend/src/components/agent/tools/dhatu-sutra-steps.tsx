import { Link } from '@tanstack/react-router';
import { ScrollTextIcon, SearchIcon, SproutIcon } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import type { AgentToolPart } from '../message-parts';
import { ClampedText, ToolShell, toolStepStatus } from './tool-shell';

export function SearchDhatusStep({
  part,
  streaming
}: {
  part: AgentToolPart<'search_dhatus'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const query = part.input?.query ?? (part.input?.gana ? `gaṇa ${part.input.gana}` : '');
  const description = [query, done ? `${part.output.total}` : null]
    .filter((v) => v !== null && v !== '')
    .join(' · ');

  return (
    <ToolShell
      icon={SearchIcon}
      runningTitle="Searching dhātus"
      doneTitle="Searched dhātus"
      errorTitle="Dhātu search failed"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? (
        <ul className="flex min-w-0 flex-col gap-1.5">
          {part.output.dhatus.map((dhatu) => (
            <li key={dhatu.id} className="min-w-0 text-xs">
              <div className="flex flex-wrap items-baseline gap-x-2">
                <Link
                  to="/dhatu/$id"
                  params={{ id: String(dhatu.id) }}
                  className="font-devanagari text-sm font-semibold text-foreground hover:text-primary hover:underline"
                >
                  {dhatu.root}
                </Link>
                <span className="text-muted-foreground tabular-nums">{dhatu.number}</span>
                <span className="text-muted-foreground">gaṇa {dhatu.gana}</span>
              </div>
              {dhatu.artha ? <ClampedText text={dhatu.artha} lines={2} /> : null}
            </li>
          ))}
        </ul>
      ) : null}
    </ToolShell>
  );
}

export function GetDhatuStep({
  part,
  streaming
}: {
  part: AgentToolPart<'get_dhatu'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const id = part.input?.id;
  const description = done
    ? `dhātu ${part.output.dhatu.number} · ${part.output.dhatu.root}`
    : id !== undefined
      ? `dhātu ${id}`
      : undefined;

  return (
    <ToolShell
      icon={SproutIcon}
      runningTitle="Opening dhātu"
      doneTitle="Opened dhātu"
      errorTitle="Could not open dhātu"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? <GetDhatuOutput part={part} /> : null}
    </ToolShell>
  );
}

function GetDhatuOutput({ part }: { part: AgentToolPart<'get_dhatu'> }) {
  if (part.state !== 'output-available') return null;
  const { dhatu, works } = part.output;
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <div className="flex flex-wrap items-baseline gap-x-2 text-xs">
        <Link
          to="/dhatu/$id"
          params={{ id: String(dhatu.id) }}
          className="font-devanagari text-base font-semibold text-foreground hover:text-primary hover:underline"
        >
          {dhatu.root}
        </Link>
        <span className="text-muted-foreground tabular-nums">{dhatu.number}</span>
        <span className="text-muted-foreground">
          gaṇa {dhatu.gana}
          {dhatu.pada ? ` · ${dhatu.pada}` : ''}
          {dhatu.it ? ` · it ${dhatu.it}` : ''}
        </span>
      </div>
      {dhatu.artha ? <ClampedText text={dhatu.artha} lines={3} /> : null}
      {works.map((work) => (
        <div key={work.dict} className="flex min-w-0 flex-col gap-1">
          <Badge variant="secondary" className="self-start text-[11px]">
            {work.dict_name}
          </Badge>
          <ul className="flex min-w-0 flex-col gap-1.5">
            {work.entries.map((entry) => (
              <li key={entry.id} className="min-w-0">
                <Link
                  to="/entry/$id"
                  params={{ id: String(entry.id) }}
                  className="font-devanagari text-xs font-semibold text-foreground hover:text-primary hover:underline"
                >
                  {entry.headword}
                </Link>
                <ClampedText text={entry.text} lines={3} />
              </li>
            ))}
          </ul>
        </div>
      ))}
    </div>
  );
}

export function SearchSutrasStep({
  part,
  streaming
}: {
  part: AgentToolPart<'search_sutras'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const query = part.input?.query ?? '';
  const description = [query, done ? `${part.output.total}` : null].filter(Boolean).join(' · ');

  return (
    <ToolShell
      icon={SearchIcon}
      runningTitle="Searching sūtras"
      doneTitle="Searched sūtras"
      errorTitle="Sūtra search failed"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? (
        <ul className="flex min-w-0 flex-col gap-1">
          {part.output.sutras.map((sutra) => (
            <li key={sutra.id} className="flex min-w-0 flex-wrap items-baseline gap-x-2 text-xs">
              <Link
                to="/sutra/$id"
                params={{ id: sutra.id }}
                className="shrink-0 font-mono font-semibold text-primary tabular-nums hover:underline"
              >
                {sutra.id}
              </Link>
              <span className="min-w-0 flex-1 font-devanagari text-foreground">{sutra.text}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </ToolShell>
  );
}

export function GetSutraStep({
  part,
  streaming
}: {
  part: AgentToolPart<'get_sutra'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const id = part.input?.id ?? '';
  const commentaries =
    done && part.state === 'output-available'
      ? part.output.commentaries.map((c) => c.dict_name).join(', ')
      : null;
  const description = [done ? part.output.sutra.id : id, commentaries].filter(Boolean).join(' · ');

  return (
    <ToolShell
      icon={ScrollTextIcon}
      runningTitle="Opening sūtra"
      doneTitle="Opened sūtra"
      errorTitle="Could not open sūtra"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done && part.state === 'output-available' ? <GetSutraOutput part={part} /> : null}
    </ToolShell>
  );
}

function GetSutraOutput({ part }: { part: AgentToolPart<'get_sutra'> }) {
  if (part.state !== 'output-available') return null;
  const { sutra, prev, next, commentaries } = part.output;
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <div className="flex flex-wrap items-baseline gap-x-2 text-xs">
        <Link
          to="/sutra/$id"
          params={{ id: sutra.id }}
          className="font-mono font-semibold text-primary tabular-nums hover:underline"
        >
          {sutra.id}
        </Link>
        <span className="min-w-0 flex-1 font-devanagari text-sm text-foreground">{sutra.text}</span>
      </div>
      {prev || next ? (
        <p className="flex gap-3 text-xs text-muted-foreground">
          {prev ? (
            <Link
              to="/sutra/$id"
              params={{ id: prev }}
              className="hover:text-primary hover:underline"
            >
              ← {prev}
            </Link>
          ) : null}
          {next ? (
            <Link
              to="/sutra/$id"
              params={{ id: next }}
              className="hover:text-primary hover:underline"
            >
              {next} →
            </Link>
          ) : null}
        </p>
      ) : null}
      {commentaries.map((commentary, i) => (
        <div key={`${commentary.dict}-${i}`} className="flex min-w-0 flex-col gap-1">
          <Badge variant="secondary" className="self-start text-[11px]">
            {commentary.dict_name}
          </Badge>
          <ClampedText text={commentary.text} />
        </div>
      ))}
    </div>
  );
}
