import { WrenchIcon } from 'lucide-react';
import {
  TraceCollapsible,
  TraceCollapsibleContent,
  TraceCollapsibleTrigger,
  TraceIcon,
  TraceLine,
  TraceTitle
} from '@/components/ui/trace';

/** Fallback for a tool the UI doesn't know yet: a generic row with the raw I/O. */
export function UnknownToolStep({
  name,
  input,
  output,
  errorText,
  failed
}: {
  name: string;
  input: unknown;
  output: unknown;
  errorText?: string;
  failed: boolean;
}) {
  const renderJson = (value: unknown) => {
    let text: string;
    try {
      text = JSON.stringify(value, null, 2) ?? String(value);
    } catch {
      text = String(value);
    }
    return (
      <pre className="max-h-64 overflow-auto rounded-lg border border-border/70 bg-muted/40 p-2.5 font-mono text-[11px] break-words whitespace-pre-wrap text-foreground">
        {text}
      </pre>
    );
  };
  return (
    <TraceCollapsible>
      <TraceCollapsibleTrigger>
        <TraceLine status={failed ? 'error' : undefined}>
          <TraceIcon>
            <WrenchIcon />
          </TraceIcon>
          <TraceTitle>{failed ? `Tool failed · ${name}` : `Ran tool · ${name}`}</TraceTitle>
        </TraceLine>
      </TraceCollapsibleTrigger>
      <TraceCollapsibleContent className="flex flex-col gap-2">
        {failed && errorText ? (
          <p className="text-xs text-destructive">{errorText}</p>
        ) : (
          <>
            {input !== undefined ? renderJson(input) : null}
            {output !== undefined ? renderJson(output) : null}
          </>
        )}
      </TraceCollapsibleContent>
    </TraceCollapsible>
  );
}
