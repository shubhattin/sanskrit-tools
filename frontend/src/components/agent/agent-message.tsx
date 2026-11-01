import { isToolUIPart } from 'ai';
import { BotIcon } from 'lucide-react';
import type { AgentUIMessage } from '@/lib/agent/types';
import { Trace, TraceIcon, TraceLine, TraceTitle } from '@/components/ui/trace';
import { AgentMarkdown } from './agent-markdown';
import { CopyReplyButton, ReplyCost } from './chat-cost';
import { groupParts, messageText, type AgentPart } from './message-parts';
import { ReasoningStep } from './tools/reasoning-step';
import {
  BrowseDictionaryStep,
  GetEntryStep,
  SearchDictionariesStep,
  SuggestHeadwordsStep
} from './tools/dictionary-steps';
import { FindParadigmsStep, GetParadigmStep } from './tools/paradigm-steps';
import {
  GetDhatuStep,
  GetSutraStep,
  SearchDhatusStep,
  SearchSutrasStep
} from './tools/dhatu-sutra-steps';
import { UnknownToolStep } from './tools/web-sources';
import { WebSearchStep } from './tools/web-search-step';

function StepRow({ part, streaming }: { part: AgentPart; streaming: boolean }) {
  if (part.type === 'reasoning') {
    return <ReasoningStep text={part.text} running={streaming && part.state === 'streaming'} />;
  }
  if (part.type === 'dynamic-tool') {
    const failed = part.state === 'output-error';
    return (
      <UnknownToolStep
        name={part.toolName}
        input={'input' in part ? part.input : undefined}
        output={'output' in part ? part.output : undefined}
        errorText={'errorText' in part ? part.errorText : undefined}
        failed={failed}
      />
    );
  }
  if (isToolUIPart(part)) return <ToolStep part={part} streaming={streaming} />;
  return null;
}

/** One typed `tool-*` part: a bespoke row per known tool, generic fallback otherwise. */
function ToolStep({ part, streaming }: { part: AgentPart; streaming: boolean }) {
  switch (part.type) {
    case 'tool-web_search':
      return <WebSearchStep part={part} streaming={streaming} />;
    case 'tool-search_dictionaries':
      return <SearchDictionariesStep part={part} streaming={streaming} />;
    case 'tool-get_entry':
      return <GetEntryStep part={part} streaming={streaming} />;
    case 'tool-suggest_headwords':
      return <SuggestHeadwordsStep part={part} streaming={streaming} />;
    case 'tool-browse_dictionary':
      return <BrowseDictionaryStep part={part} streaming={streaming} />;
    case 'tool-find_paradigms':
      return <FindParadigmsStep part={part} streaming={streaming} />;
    case 'tool-get_paradigm':
      return <GetParadigmStep part={part} streaming={streaming} />;
    case 'tool-search_dhatus':
      return <SearchDhatusStep part={part} streaming={streaming} />;
    case 'tool-get_dhatu':
      return <GetDhatuStep part={part} streaming={streaming} />;
    case 'tool-search_sutras':
      return <SearchSutrasStep part={part} streaming={streaming} />;
    case 'tool-get_sutra':
      return <GetSutraStep part={part} streaming={streaming} />;
    default:
      break;
  }
  if (!isToolUIPart(part)) return null;
  const failed = part.state === 'output-error';
  return (
    <UnknownToolStep
      name={part.type.slice('tool-'.length)}
      input={'input' in part ? part.input : undefined}
      output={'output' in part ? part.output : undefined}
      errorText={'errorText' in part ? part.errorText : undefined}
      failed={failed}
    />
  );
}

export function ThinkingLine() {
  return (
    <TraceLine status="running">
      <TraceIcon>
        <BotIcon />
      </TraceIcon>
      <TraceTitle>Working…</TraceTitle>
    </TraceLine>
  );
}

export function UserMessage({ message }: { message: AgentUIMessage }) {
  return (
    <div className="flex justify-end">
      <div className="max-w-[85%] rounded-2xl rounded-br-md border border-border/60 bg-muted px-3.5 py-2 text-sm leading-relaxed break-words whitespace-pre-wrap text-foreground">
        {messageText(message)}
      </div>
    </div>
  );
}

/** One agent turn: step traces and replies, in the order they happened. */
export function AssistantMessage({
  message,
  streaming
}: {
  message: AgentUIMessage;
  streaming: boolean;
}) {
  const groups = groupParts(message.parts);
  // Between steps (a tool finished, the next step has not streamed yet) show that work continues.
  const lastPart = message.parts.filter((part) => part.type !== 'step-start').at(-1);
  const waiting =
    streaming &&
    (!lastPart ||
      (isToolUIPart(lastPart) &&
        (lastPart.state === 'output-available' || lastPart.state === 'output-error')) ||
      (lastPart.type === 'reasoning' && lastPart.state === 'done'));

  return (
    <div className="flex w-full min-w-0 flex-col gap-3">
      {groups.map((group, index) => {
        const isLast = index === groups.length - 1;
        switch (group.kind) {
          case 'steps':
            return (
              <Trace key={group.key}>
                {group.parts.map((part, partIndex) => (
                  <StepRow key={partIndex} part={part} streaming={streaming} />
                ))}
              </Trace>
            );
          case 'text':
            return (
              <AgentMarkdown key={group.key} streaming={streaming && isLast}>
                {group.text}
              </AgentMarkdown>
            );
        }
      })}
      {waiting ? <ThinkingLine /> : null}
      {!streaming ? (
        <div className="flex items-center gap-0.5">
          <ReplyCost metadata={message.metadata} />
          <CopyReplyButton text={messageText(message)} />
        </div>
      ) : null}
    </div>
  );
}
