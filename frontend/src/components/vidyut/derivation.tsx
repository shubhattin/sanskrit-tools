import { createContext, useContext, useState, type ReactNode } from 'react';
import { Link } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { GitBranchIcon } from 'lucide-react';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle
} from '@/components/ui/sheet';
import { Badge } from '@/components/ui/badge';
import { deriveQuery, type Derivation, type RuleRef, type WordSpec } from '@/lib/vidyut/api';
import { ErrorNote, LoadingBlock, Segmented } from './controls';
import { cn } from '@/lib/utils';

interface Pick {
  spec: WordSpec;
  /** The form the reader clicked (selects that result when there are several). */
  form?: string;
  /** What is being derived, e.g. "भू · लट् · प्रथमपुरुषः एकवचनम्". */
  context?: string;
}

const OpenCtx = createContext<(p: Pick) => void>(() => {});

/** Lets any form in the vidyut tools open its step-by-step prakriyā in a side sheet. */
export function DerivationProvider({ children }: { children: ReactNode }) {
  const [pick, setPick] = useState<Pick | null>(null);
  return (
    <OpenCtx.Provider value={setPick}>
      {children}
      <Sheet open={pick !== null} onOpenChange={(open) => !open && setPick(null)}>
        <SheetContent side="right" className="w-full gap-0 p-0 sm:max-w-xl">
          {pick ? <DerivationBody pick={pick} /> : null}
        </SheetContent>
      </Sheet>
    </OpenCtx.Provider>
  );
}

export function useOpenDerivation() {
  return useContext(OpenCtx);
}

/** A generated form; click shows how it is derived. */
export function FormButton({
  form,
  spec,
  context,
  className
}: {
  form: string;
  spec: WordSpec;
  context?: string;
  className?: string;
}) {
  const open = useOpenDerivation();
  return (
    <button
      type="button"
      onClick={() => open({ spec, form, context })}
      title="Show derivation"
      className={cn(
        '-mx-1 rounded-md px-1 text-left font-devanagari underline-offset-4 transition-colors hover:bg-primary/10 hover:text-primary focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
        className
      )}
    >
      {form}
    </button>
  );
}

const SOURCE: Record<RuleRef['source'], string> = {
  ashtadhyayi: '',
  varttika: 'वार्तिकम्',
  dhatupatha: 'धातुपाठः',
  unadipatha: 'उणादिपाठः',
  linganushasana: 'लिङ्गानुशासनम्',
  phit: 'फिट्सूत्रम्',
  kashika: 'काशिका',
  kaumudi: 'सिद्धान्तकौमुदी',
  other: ''
};

function RuleCode({ rule }: { rule: RuleRef }) {
  const cls =
    'inline-flex h-6 min-w-14 items-center justify-center rounded-md border px-1.5 font-mono text-xs tabular-nums';
  if (!rule.code) return <span className={cn(cls, 'text-muted-foreground')}>—</span>;
  if (rule.sutra_id && rule.source !== 'kaumudi') {
    return (
      <Link
        to="/sutra/$id"
        params={{ id: rule.sutra_id }}
        className={cn(cls, 'border-primary/30 text-primary hover:bg-primary/10')}
        title="Open sūtra with commentaries"
      >
        {rule.code}
      </Link>
    );
  }
  return <span className={cn(cls, 'text-muted-foreground')}>{rule.code}</span>;
}

/** Drops a step that repeats the previous one exactly (same rule, same result). */
function visibleSteps(d: Derivation) {
  const key = (i: number) => {
    const s = d.steps[i]!;
    return `${s.rule.source}|${s.rule.code}|${s.terms.map((t) => t.text).join('+')}`;
  };
  return d.steps.filter((_, i) => i === 0 || key(i) !== key(i - 1));
}

function DerivationBody({ pick }: { pick: Pick }) {
  const q = useQuery(deriveQuery(pick.spec));
  const results = q.data ?? [];
  const initial = Math.max(
    0,
    results.findIndex((d) => d.text === pick.form)
  );
  const [chosen, setChosen] = useState<number | null>(null);
  const idx = chosen ?? initial;
  const d = results[idx];

  return (
    <>
      <SheetHeader className="border-b bg-gradient-to-br from-primary/10 via-transparent to-sky-500/10 p-4 pr-12">
        <SheetTitle className="flex items-center gap-2 font-devanagari text-2xl font-bold">
          <GitBranchIcon className="size-5 shrink-0 text-primary" aria-hidden />
          {d?.text ?? pick.form ?? 'प्रक्रिया'}
        </SheetTitle>
        <SheetDescription className="font-devanagari">
          {pick.context ? `${pick.context} · ` : ''}
          <span className="font-sans">step-by-step prakriyā</span>
        </SheetDescription>
        {results.length > 1 ? (
          <Segmented
            size="sm"
            label="Result"
            value={String(idx)}
            onChange={(v) => setChosen(Number(v))}
            options={results.map((r, i) => ({
              id: String(i),
              label: <span className="font-devanagari">{r.text}</span>
            }))}
            className="mt-2"
          />
        ) : null}
      </SheetHeader>
      <div className="flex-1 overflow-y-auto p-4">
        {q.isPending ? (
          <LoadingBlock rows={8} label="Deriving" />
        ) : q.isError ? (
          <ErrorNote error={q.error} />
        ) : !d ? (
          <p className="text-sm text-muted-foreground">vidyut produced no derivation.</p>
        ) : (
          <>
            {d.optional_rules.length > 0 ? (
              <div className="mb-4 rounded-xl border border-amber-500/30 bg-amber-500/5 p-3 text-xs">
                <p className="mb-1.5 font-medium">
                  Optional rules behind this form
                  {results.length > 1 ? ' (why there are several)' : ''}:
                </p>
                <div className="flex flex-wrap gap-1.5">
                  {d.optional_rules.map((o, i) => (
                    <span key={i} className="inline-flex items-center gap-1">
                      <RuleCode rule={o.rule} />
                      <Badge variant={o.applied ? 'secondary' : 'outline'}>
                        {o.applied ? 'applied' : 'not applied'}
                      </Badge>
                    </span>
                  ))}
                </div>
              </div>
            ) : null}
            <ol className="relative flex flex-col">
              {visibleSteps(d).map((s, i) => (
                <li
                  key={i}
                  className="grid grid-cols-[auto_1fr] gap-x-3 border-b border-dashed py-2 last:border-0"
                >
                  <div className="pt-0.5">
                    <RuleCode rule={s.rule} />
                  </div>
                  <div className="min-w-0">
                    <p className="font-devanagari text-base leading-snug">
                      {s.terms.map((t, j) => (
                        <span key={j}>
                          {j > 0 ? <span className="px-1 text-muted-foreground">+</span> : null}
                          <span className={cn(t.changed && 'font-semibold text-primary')}>
                            {t.text}
                          </span>
                        </span>
                      ))}
                    </p>
                    {s.rule.text || SOURCE[s.rule.source] ? (
                      <p className="mt-0.5 font-devanagari text-sm text-muted-foreground">
                        {SOURCE[s.rule.source] ? (
                          <span className="mr-1.5 text-xs text-foreground/70">
                            {SOURCE[s.rule.source]}
                          </span>
                        ) : null}
                        {s.rule.url ? (
                          <a
                            href={s.rule.url}
                            target="_blank"
                            rel="noreferrer"
                            className="underline underline-offset-2"
                          >
                            {s.rule.text}
                          </a>
                        ) : (
                          s.rule.text
                        )}
                      </p>
                    ) : null}
                  </div>
                </li>
              ))}
            </ol>
          </>
        )}
      </div>
    </>
  );
}
