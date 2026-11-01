import { Link } from '@tanstack/react-router';
import { TableIcon, TablePropertiesIcon } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { ParadigmView } from '@/components/dict/paradigm';
import { PARADIGM_SOURCE_LABELS, type Paradigm, type ParadigmSource } from '@/lib/api/types';
import type { AgentToolPart } from '../message-parts';
import { ToolShell, toolStepStatus } from './tool-shell';

function isParadigmSource(value: string): value is ParadigmSource {
  return (
    value === 'heritage_decl' ||
    value === 'heritage_kridanta' ||
    value === 'heritage_conj' ||
    value === 'jnu_tinanta'
  );
}

export function FindParadigmsStep({
  part,
  streaming
}: {
  part: AgentToolPart<'find_paradigms'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const query = part.input?.query ?? '';
  const description = [query, done ? `${part.output.total} tables` : null]
    .filter(Boolean)
    .join(' · ');

  return (
    <ToolShell
      icon={TablePropertiesIcon}
      runningTitle="Finding paradigms"
      doneTitle="Found paradigms"
      errorTitle="Paradigm search failed"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? (
        <ul className="flex min-w-0 flex-col gap-1">
          {part.output.paradigms.map((p) => (
            <li key={p.id} className="flex flex-wrap items-baseline gap-x-2 text-xs">
              <Link
                to="/paradigm/$id"
                params={{ id: String(p.id) }}
                className="font-devanagari text-sm font-semibold text-foreground hover:text-primary hover:underline"
              >
                {p.lemma}
              </Link>
              {p.label ? (
                <span className="font-devanagari text-muted-foreground">{p.label}</span>
              ) : null}
              {isParadigmSource(p.source) ? (
                <Badge variant="secondary" className="text-[11px]">
                  {PARADIGM_SOURCE_LABELS[p.source]}
                </Badge>
              ) : null}
              {p.root ? (
                <span className="text-muted-foreground">
                  from <span className="font-devanagari">{p.root}</span>
                </span>
              ) : null}
            </li>
          ))}
        </ul>
      ) : null}
    </ToolShell>
  );
}

export function GetParadigmStep({
  part,
  streaming
}: {
  part: AgentToolPart<'get_paradigm'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const id = part.input?.id;
  const description = done
    ? [part.output.lemma, part.output.label].filter(Boolean).join(' · ')
    : id !== undefined
      ? `table ${id}`
      : undefined;

  return (
    <ToolShell
      icon={TableIcon}
      runningTitle="Opening inflection table"
      doneTitle="Inflection table"
      errorTitle="Could not open inflection table"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? <GetParadigmOutput part={part} /> : null}
    </ToolShell>
  );
}

function GetParadigmOutput({ part }: { part: AgentToolPart<'get_paradigm'> }) {
  if (part.state !== 'output-available') return null;
  const { output } = part;
  const paradigm: Paradigm = {
    id: output.id,
    source: isParadigmSource(output.source) ? output.source : 'heritage_decl',
    lemma: output.lemma,
    root: output.root,
    label: output.label,
    tables: output.tables
  };
  return (
    <div className="flex min-w-0 flex-col gap-2">
      <div className="flex flex-wrap items-baseline gap-x-2 text-xs">
        <Link
          to="/paradigm/$id"
          params={{ id: String(output.id) }}
          className="font-devanagari text-sm font-semibold text-foreground hover:text-primary hover:underline"
        >
          {output.lemma}
        </Link>
        {output.label ? (
          <span className="font-devanagari text-muted-foreground">{output.label}</span>
        ) : null}
        <Badge variant="secondary" className="text-[11px]">
          {PARADIGM_SOURCE_LABELS[paradigm.source]}
        </Badge>
        {output.filtered ? (
          <span className="text-muted-foreground">(filtered to the asked form)</span>
        ) : null}
      </div>
      <ParadigmView paradigm={paradigm} />
    </div>
  );
}
