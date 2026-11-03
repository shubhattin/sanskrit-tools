import { Fragment, useDeferredValue, useEffect, useRef, useState } from 'react';
import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { AudioWaveformIcon, SearchIcon } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import {
  DevaInput,
  DevaTextarea,
  ErrorNote,
  LoadingBlock,
  SectionLabel
} from '@/components/vidyut/controls';
import { searchString } from '@/lib/search-params';
import { chandasQuery, metersQuery, type MeterInfo, type MeterMatch } from '@/lib/vidyut/api';
import { devaNum } from '@/lib/vidyut/labels';
import { cn } from '@/lib/utils';

type Tab = 'scan' | 'meters';

interface ChandasSearch {
  tab?: Tab;
  /** The verse being scanned. */
  v?: string;
  /** Meter to show in the catalogue. */
  m?: string;
}

export const Route = createFileRoute('/vidyut/chandas')({
  validateSearch: (s: Record<string, unknown>): ChandasSearch => ({
    tab: s.tab === 'meters' ? 'meters' : undefined,
    v: searchString(s.v),
    m: searchString(s.m)
  }),
  component: ChandasTool
});

const SAMPLES = [
  {
    label: 'गीता १.१',
    text: 'धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः।\nमामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय॥'
  },
  {
    label: 'मेघदूतम् १',
    text: 'कश्चित्कान्ताविरहगुरुणा स्वाधिकारात्प्रमत्तः\nशापेनास्तङ्गमितमहिमा वर्षभोग्येण भर्तुः।'
  },
  {
    label: 'या कुन्देन्दु…',
    text: 'या कुन्देन्दुतुषारहारधवला या शुभ्रवस्त्रावृता\nया वीणावरदण्डमण्डितकरा या श्वेतपद्मासना।'
  },
  { label: 'मातः समस्त…', text: 'मातः समस्तजगतां मधुकैटभारेः' }
];

function ChandasTool() {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: Route.fullPath });
  const set = (patch: Partial<ChandasSearch>) =>
    void navigate({ search: (prev) => ({ ...prev, ...patch }) });
  const showMeter = (name: string) => set({ tab: 'meters', m: name });

  return (
    <Tabs
      value={search.tab ?? 'scan'}
      onValueChange={(t) => set({ tab: t === 'meters' ? 'meters' : undefined })}
    >
      <TabsList>
        <TabsTrigger value="scan">
          Scan a verse <span className="font-devanagari text-xs opacity-70">छन्दोज्ञानम्</span>
        </TabsTrigger>
        <TabsTrigger value="meters">
          Meters <span className="font-devanagari text-xs opacity-70">वृत्तानि</span>
        </TabsTrigger>
      </TabsList>
      <TabsContent value="scan" className="pt-2">
        <Scan verse={search.v} onScan={(v) => set({ v })} onMeter={showMeter} />
      </TabsContent>
      <TabsContent value="meters" className="pt-2">
        <Meters selected={search.m} onSelect={(m) => set({ m })} />
      </TabsContent>
    </Tabs>
  );
}

const FIT: Record<MeterMatch['fit'], string> = {
  full: 'whole verse fits',
  pada: 'whole pādas fit',
  prefix: 'the beginning fits'
};
const KIND: Record<MeterMatch['kind'], string> = {
  vrtta: 'vṛtta',
  jati: 'jāti',
  anushtubh: 'anuṣṭubh'
};

/** Laghu / guru mark. */
function Mark({ w, className }: { w: string; className?: string }) {
  return (
    <span
      className={cn(
        'font-devanagari',
        w === 'G' ? 'text-primary' : w === 'L' ? 'text-muted-foreground' : 'text-amber-600',
        className
      )}
    >
      {w === 'G' ? 'ऽ' : w === 'L' ? '।' : '×'}
    </span>
  );
}

function Scan({
  verse,
  onScan,
  onMeter
}: {
  verse?: string;
  onScan: (v: string) => void;
  onMeter: (name: string) => void;
}) {
  const [text, setText] = useState(verse ?? '');
  const [synced, setSynced] = useState(verse);
  if (synced !== verse) {
    setSynced(verse);
    setText(verse ?? '');
  }
  const q = useQuery({ ...chandasQuery(verse ?? ''), enabled: !!verse });

  return (
    <div className="flex flex-col gap-4">
      <form
        className="flex max-w-3xl flex-col gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          if (text.trim()) onScan(text.trim());
        }}
      >
        <DevaTextarea
          value={text}
          onChange={setText}
          label="Verse"
          placeholder={
            'One pāda or half-verse per line (daṇḍas also end a line)\nधर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः।'
          }
        />
        <div className="flex flex-wrap items-center gap-2">
          <Button type="submit" size="sm" disabled={!text.trim()}>
            <AudioWaveformIcon aria-hidden /> Scan
          </Button>
          <span className="ml-1 text-xs text-muted-foreground">Try</span>
          {SAMPLES.map((s) => (
            <button
              key={s.label}
              type="button"
              onClick={() => {
                setText(s.text);
                onScan(s.text);
              }}
              className="rounded-full border px-2.5 py-0.5 font-devanagari text-xs text-muted-foreground transition-colors hover:border-primary/50 hover:text-foreground"
            >
              {s.label}
            </button>
          ))}
        </div>
      </form>

      {!verse ? null : q.isPending ? (
        <LoadingBlock rows={3} label="Scanning" />
      ) : q.isError ? (
        <ErrorNote error={q.error} />
      ) : (
        <div className="flex flex-col gap-4">
          <div className="flex flex-wrap items-center gap-3 rounded-xl border bg-gradient-to-br from-primary/10 via-transparent to-sky-500/10 p-4">
            {q.data.best ? (
              <>
                <div className="flex flex-col">
                  <SectionLabel>Meter</SectionLabel>
                  <span className="font-devanagari text-2xl font-bold">{q.data.best.name}</span>
                </div>
                <Badge variant="secondary">{KIND[q.data.best.kind]}</Badge>
                <Badge variant="outline">{FIT[q.data.best.fit]}</Badge>
                {q.data.best.note ? <Badge variant="outline">{q.data.best.note}</Badge> : null}
                <Button
                  variant="ghost"
                  size="sm"
                  className="ml-auto"
                  onClick={() => onMeter(q.data.best!.name)}
                >
                  See its pattern →
                </Button>
              </>
            ) : (
              <p className="text-sm text-muted-foreground">
                No meter in the catalogue matches. The scansion below may still help.
              </p>
            )}
          </div>

          <div className="flex flex-col gap-3">
            {q.data.lines.map((l, i) => (
              <div key={i} className="overflow-x-auto rounded-xl border bg-card p-3">
                <div className="flex flex-wrap gap-1">
                  {l.syllables.map((s, j) => (
                    <span
                      key={j}
                      className={cn(
                        'flex min-w-9 flex-col items-center rounded-lg border px-1.5 py-1',
                        s.guru ? 'border-primary/40 bg-primary/5' : 'border-transparent bg-muted/50'
                      )}
                    >
                      <span className="font-devanagari text-base leading-tight">{s.text}</span>
                      <Mark w={s.guru ? 'G' : 'L'} className="text-xs leading-none" />
                    </span>
                  ))}
                </div>
                <p className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
                  <span>
                    gaṇas{' '}
                    <span className="font-devanagari text-sm text-foreground">
                      {l.ganas.join(' ')}
                    </span>
                  </span>
                  <span>
                    <span className="font-devanagari">{devaNum(l.syllables.length)}</span> akṣaras ·{' '}
                    <span className="font-devanagari">{devaNum(l.matras)}</span> mātrās
                  </span>
                </p>
              </div>
            ))}
          </div>

          {q.data.others.length > 0 ? (
            <div className="flex flex-wrap items-center gap-1.5 text-sm">
              <span className="text-xs text-muted-foreground">Also compatible</span>
              {q.data.others.map((o) => (
                <button
                  key={o.name}
                  type="button"
                  onClick={() => onMeter(o.name)}
                  className="rounded-full border px-2.5 py-0.5 font-devanagari text-xs hover:border-primary/50"
                  title={FIT[o.fit]}
                >
                  {o.name}
                </button>
              ))}
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
}

/** L/G pattern as ऽ / । marks, spaced by gaṇa (three syllables), with yati marked. */
function Pattern({ pattern, yati }: { pattern: string; yati: number[] }) {
  const marks = [...pattern];
  return (
    <span className="inline-flex flex-wrap items-center text-sm leading-none">
      {marks.map((w, i) => (
        <Fragment key={i}>
          {i > 0 && i % 3 === 0 ? <span className="w-2" aria-hidden /> : null}
          <Mark w={w} className="w-2.5 text-center" />
          {yati.includes(i + 1) && i + 1 < marks.length ? (
            <span
              className="mx-1.5 inline-block h-4 w-0.5 rounded-full bg-amber-500"
              title="yati (caesura)"
            />
          ) : null}
        </Fragment>
      ))}
    </span>
  );
}

function MeterRow({ m, selected }: { m: MeterInfo; selected: boolean }) {
  const ref = useRef<HTMLLIElement>(null);
  useEffect(() => {
    if (selected) ref.current?.scrollIntoView({ block: 'center', behavior: 'smooth' });
  }, [selected]);
  return (
    <li
      ref={ref}
      className={cn(
        'flex flex-col gap-1 border-b px-3 py-2 sm:flex-row sm:items-center sm:gap-4',
        selected && 'bg-primary/10 ring-1 ring-primary/40'
      )}
    >
      <span className="flex w-48 shrink-0 items-baseline gap-2">
        <span className="font-devanagari text-base font-semibold">{m.name}</span>
        <span className="text-[11px] text-muted-foreground">
          {m.kind === 'jati'
            ? 'jāti'
            : `${m.syllables[0] ?? ''}${m.padas.length > 1 ? '…' : ''} akṣaras`}
        </span>
      </span>
      {m.kind === 'jati' ? (
        <span className="text-xs text-muted-foreground">
          mātrās per pāda{' '}
          <span className="font-devanagari text-sm text-foreground">
            {m.matras.map(devaNum).join(' · ')}
          </span>
        </span>
      ) : (
        <span className="flex flex-col gap-1">
          {m.padas.map((p, i) => (
            <span key={i} className="flex flex-wrap items-center gap-x-3 gap-y-0.5">
              <Pattern pattern={p.pattern} yati={p.yati} />
              {p.ganas.length ? (
                <span className="font-devanagari text-xs text-muted-foreground">
                  {p.ganas.join(' ')}
                </span>
              ) : null}
            </span>
          ))}
          {m.kind === 'anushtubh' ? (
            <span className="text-[11px] text-muted-foreground">
              8 akṣaras a pāda; 5th laghu, 6th guru; 7th guru in odd pādas, laghu in even ones
            </span>
          ) : null}
        </span>
      )}
    </li>
  );
}

function Meters({ selected, onSelect }: { selected?: string; onSelect: (m?: string) => void }) {
  const q = useQuery(metersQuery());
  const [filter, setFilter] = useState('');
  const f = useDeferredValue(filter.trim());
  if (q.isPending) return <LoadingBlock rows={6} label="Loading meters" />;
  if (q.isError) return <ErrorNote error={q.error} />;
  const shown = f ? q.data.filter((m) => m.name.includes(f)) : q.data;
  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-3">
        <DevaInput
          value={filter}
          onChange={setFilter}
          label="Filter meters"
          placeholder="Filter: मन्दाक्रान्ता, इन्द्रवज्रा…"
          icon={<SearchIcon aria-hidden />}
          className="max-w-sm"
        />
        <span className="text-xs text-muted-foreground">
          <Mark w="G" /> guru · <Mark w="L" /> laghu · <Mark w="." /> either ·{' '}
          <span className="inline-block h-3 w-px bg-amber-500 align-middle" /> yati
        </span>
        {selected ? (
          <Button variant="ghost" size="sm" onClick={() => onSelect(undefined)}>
            Clear selection
          </Button>
        ) : null}
      </div>
      {shown.length === 0 ? (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <AudioWaveformIcon aria-hidden />
            </EmptyMedia>
            <EmptyTitle>No meter by that name</EmptyTitle>
            <EmptyDescription>{q.data.length} meters in the catalogue.</EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <ul className="overflow-hidden rounded-xl border bg-card">
          {shown.map((m, i) => (
            // A few names occur twice in the catalogue (variant patterns).
            <MeterRow key={`${m.name}-${i}`} m={m} selected={m.name === selected} />
          ))}
        </ul>
      )}
    </div>
  );
}
