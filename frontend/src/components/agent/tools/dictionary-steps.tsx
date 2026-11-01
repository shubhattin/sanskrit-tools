import { Link } from '@tanstack/react-router';
import { BookOpenTextIcon, BookOpenIcon, LightbulbIcon, LibraryBigIcon } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { MatchBadge } from '@/components/dict/sense';
import type { MatchKind } from '@/lib/api/types';
import type { AgentToolPart } from '../message-parts';
import { ClampedText, ToolShell, toolStepStatus, useDictShortNames } from './tool-shell';

function isMatchKind(value: string): value is MatchKind {
  return (
    value === 'exact' ||
    value === 'form' ||
    value === 'prefix' ||
    value === 'fuzzy' ||
    value === 'fulltext'
  );
}

export function SearchDictionariesStep({
  part,
  streaming
}: {
  part: AgentToolPart<'search_dictionaries'>;
  streaming: boolean;
}) {
  const shortName = useDictShortNames();
  const status = toolStepStatus(part.state, streaming);
  const query = part.input?.query ?? '';
  const done = status === 'done' && part.state === 'output-available';
  const entryCount = done ? part.output.results.reduce((n, g) => n + g.entries.length, 0) : 0;
  const searched = done
    ? part.output.searched === 'all'
      ? 'all dictionaries'
      : part.output.searched.map(shortName).join(', ')
    : null;
  const description = [query, searched, done ? `${entryCount} entries` : null]
    .filter(Boolean)
    .join(' · ');

  return (
    <ToolShell
      icon={LibraryBigIcon}
      runningTitle="Searching dictionaries"
      doneTitle="Searched dictionaries"
      errorTitle="Dictionary search failed"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? <SearchDictionariesOutput part={part} /> : null}
    </ToolShell>
  );
}

function SearchDictionariesOutput({ part }: { part: AgentToolPart<'search_dictionaries'> }) {
  if (part.state !== 'output-available') return null;
  const { results, forms, did_you_mean: didYouMean } = part.output;
  return (
    <div className="flex min-w-0 flex-col gap-3">
      {results.map((group) => (
        <div key={group.dict} className="flex min-w-0 flex-col gap-1.5">
          <div className="flex flex-wrap items-center gap-1.5 text-xs">
            <Badge variant="secondary" className="font-devanagari font-bold">
              {group.dict_name}
            </Badge>
            <span className="text-muted-foreground tabular-nums">
              {group.total} {group.total === 1 ? 'hit' : 'hits'}
            </span>
          </div>
          <ul className="flex min-w-0 flex-col gap-2">
            {group.entries.map((entry) => (
              <li key={entry.id} className="min-w-0">
                <div className="flex flex-wrap items-baseline gap-x-2">
                  <Link
                    to="/entry/$id"
                    params={{ id: String(entry.id) }}
                    className="font-devanagari text-sm font-semibold text-foreground hover:text-primary hover:underline"
                  >
                    {entry.headword}
                  </Link>
                  {isMatchKind(entry.match) ? <MatchBadge match={entry.match} /> : null}
                </div>
                {entry.text ? <ClampedText text={entry.text} lines={3} /> : null}
              </li>
            ))}
          </ul>
        </div>
      ))}
      {forms.length > 0 ? (
        <div className="flex min-w-0 flex-col gap-1">
          <p className="text-[11px] font-semibold tracking-wider text-muted-foreground uppercase">
            Inflected forms
          </p>
          <ul className="flex min-w-0 flex-col gap-1">
            {forms.map((form, i) => (
              <li key={`${form.form}-${form.lemma}-${i}`} className="text-xs">
                <span className="font-devanagari font-semibold text-foreground">{form.form}</span>
                <span className="text-muted-foreground"> → </span>
                <span className="font-devanagari text-foreground">{form.lemma}</span>
                <span className="text-muted-foreground"> · {form.analysis}</span>
                {form.paradigm_ids.length > 0 ? (
                  <>
                    {' '}
                    <Link
                      to="/paradigm/$id"
                      params={{ id: String(form.paradigm_ids[0]) }}
                      className="text-primary hover:underline"
                    >
                      Table
                    </Link>
                  </>
                ) : null}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      {didYouMean.length > 0 ? (
        <p className="text-xs text-muted-foreground">
          Did you mean{' '}
          {didYouMean.map((w, i) => (
            <span key={w}>
              {i > 0 ? ', ' : ''}
              <span className="font-devanagari font-medium text-foreground">{w}</span>
            </span>
          ))}
          ?
        </p>
      ) : null}
    </div>
  );
}

export function GetEntryStep({
  part,
  streaming
}: {
  part: AgentToolPart<'get_entry'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const id = part.input?.id;
  const description = done
    ? `${part.output.headword} (${part.output.dict_name})`
    : id !== undefined
      ? `entry ${id}`
      : undefined;

  return (
    <ToolShell
      icon={BookOpenTextIcon}
      runningTitle="Opening entry"
      doneTitle="Opened entry"
      errorTitle="Could not open entry"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? <GetEntryOutput part={part} /> : null}
    </ToolShell>
  );
}

function GetEntryOutput({ part }: { part: AgentToolPart<'get_entry'> }) {
  if (part.state !== 'output-available') return null;
  const { output } = part;
  return (
    <div className="flex min-w-0 flex-col gap-1.5">
      <div className="flex flex-wrap items-baseline gap-x-2">
        <Link
          to="/entry/$id"
          params={{ id: String(output.id) }}
          className="font-devanagari text-base font-semibold text-foreground hover:text-primary hover:underline"
        >
          {output.headword}
        </Link>
        <Badge variant="secondary" className="text-[11px]">
          {output.dict_name}
        </Badge>
      </div>
      <ClampedText text={output.text} />
      <div className="flex flex-wrap gap-2 text-xs">
        {output.sutra_id ? (
          <Link
            to="/sutra/$id"
            params={{ id: output.sutra_id }}
            className="text-primary hover:underline"
          >
            Sūtra {output.sutra_id}
          </Link>
        ) : null}
        {output.dhatu_id !== null ? (
          <Link
            to="/dhatu/$id"
            params={{ id: String(output.dhatu_id) }}
            className="text-primary hover:underline"
          >
            Dhātu #{output.dhatu_id}
          </Link>
        ) : null}
      </div>
    </div>
  );
}

export function SuggestHeadwordsStep({
  part,
  streaming
}: {
  part: AgentToolPart<'suggest_headwords'>;
  streaming: boolean;
}) {
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const prefix = part.input?.prefix ?? '';
  const description = [prefix, done ? `${part.output.items.length} suggestions` : null]
    .filter(Boolean)
    .join(' · ');

  return (
    <ToolShell
      icon={LightbulbIcon}
      runningTitle="Suggesting headwords"
      doneTitle="Suggested headwords"
      errorTitle="Headword suggestions failed"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done ? (
        <ul className="flex flex-wrap gap-1.5">
          {part.output.items.map((item) => (
            <li
              key={item.headword}
              className="rounded-full bg-muted px-2 py-0.5 font-devanagari text-xs"
              title={`${item.count} entries`}
            >
              {item.headword}
              <span className="ml-1 text-muted-foreground tabular-nums">{item.count}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </ToolShell>
  );
}

export function BrowseDictionaryStep({
  part,
  streaming
}: {
  part: AgentToolPart<'browse_dictionary'>;
  streaming: boolean;
}) {
  const shortName = useDictShortNames();
  const status = toolStepStatus(part.state, streaming);
  const done = status === 'done' && part.state === 'output-available';
  const dict = part.input?.dict ?? '';
  const headword = part.input?.headword ?? '';
  const description =
    done && part.state === 'output-available'
      ? `${shortName(part.output.dict)} near ${headword} · ${part.output.items.length} entries`
      : `${shortName(dict)} near ${headword}`;

  return (
    <ToolShell
      icon={BookOpenIcon}
      runningTitle="Browsing dictionary"
      doneTitle={`Browsed ${shortName(done && part.state === 'output-available' ? part.output.dict : dict)}`}
      errorTitle="Dictionary browse failed"
      description={description}
      status={status}
      errorText={status === 'error' ? part.errorText : undefined}
    >
      {done && part.state === 'output-available' ? (
        <ul className="flex min-w-0 flex-col gap-0.5">
          {part.output.items.map((item) => {
            const center = item.seq === part.output.center;
            return (
              <li key={item.id}>
                <Link
                  to="/entry/$id"
                  params={{ id: String(item.id) }}
                  className={
                    center
                      ? 'block rounded-md bg-primary/10 px-2 py-0.5 font-devanagari text-xs font-semibold text-foreground'
                      : 'block rounded-md px-2 py-0.5 font-devanagari text-xs text-muted-foreground hover:bg-muted hover:text-foreground hover:underline'
                  }
                >
                  {item.headword}
                </Link>
              </li>
            );
          })}
        </ul>
      ) : null}
    </ToolShell>
  );
}
