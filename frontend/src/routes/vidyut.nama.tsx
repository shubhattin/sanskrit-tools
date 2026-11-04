import { useEffect, useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { PlusIcon, ShapesIcon, XIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import { DevaInput, ErrorNote, LoadingBlock, Segmented } from '@/components/vidyut/controls';
import { FormButton } from '@/components/vidyut/derivation';
import { FormSuggestions } from '@/components/vidyut/words';
import { FormsTable } from '@/components/vidyut/forms-table';
import { searchString } from '@/lib/search-params';
import {
  samasaQuery,
  stryantaQuery,
  subantaQuery,
  taddhitaQuery,
  type LingaId,
  type SamasaKind,
  type SamasaPada,
  type StemArgs,
  type VibhaktiId
} from '@/lib/vidyut/api';
import { LINGAS, LINGA_LABEL, SAMASAS, VACANAS, VIBHAKTIS } from '@/lib/vidyut/labels';
import { cn } from '@/lib/utils';

type Tab = 'sup' | 'stri' | 'tad' | 'sam';
const TABS: Tab[] = ['sup', 'stri', 'tad', 'sam'];

interface NamaSearch {
  stem?: string;
  linga?: LingaId;
  tab?: Tab;
  /** A derived stem (kṛdanta, taddhitānta, samāsa) the other tools sent here to decline. */
  from?: StemArgs;
}

const DERIVED = new Set(['krdanta', 'taddhitanta', 'samasa']);

export const Route = createFileRoute('/vidyut/nama')({
  validateSearch: (s: Record<string, unknown>): NamaSearch => {
    const from = s.from as StemArgs | undefined;
    return {
      stem: searchString(s.stem),
      linga: LINGAS.find((l) => l.id === s.linga)?.id,
      tab: TABS.includes(s.tab as Tab) && s.tab !== 'sup' ? (s.tab as Tab) : undefined,
      from: from && typeof from === 'object' && DERIVED.has(from.kind) ? from : undefined
    };
  },
  component: NamaTool
});

function NamaTool() {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: Route.fullPath });
  const set = (patch: Partial<NamaSearch>, replace = false) =>
    void navigate({ search: (prev) => ({ ...prev, ...patch }), replace });

  // The field edits a plain stem; typing replaces a derived one. URL follows after a pause.
  const [text, setText] = useState(search.stem ?? '');
  const [syncedStem, setSyncedStem] = useState(search.stem);
  if (syncedStem !== search.stem) {
    setSyncedStem(search.stem);
    setText(search.stem ?? '');
  }
  useEffect(() => {
    const t = setTimeout(() => {
      const stem = text.trim() || undefined;
      if (stem !== search.stem) set({ stem, from: undefined }, true);
    }, 450);
    return () => clearTimeout(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [text]);

  const stem = search.stem;
  const linga = search.linga ?? 'pum';
  const stemArgs: StemArgs | null = search.from ?? (stem ? { kind: 'basic', stem } : null);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
        <DevaInput
          value={text}
          onChange={setText}
          label="Stem (prātipadika)"
          placeholder="Stem: राम, मति, राजन्, मनस्…"
          className="sm:max-w-sm"
          autoFocus={!stem}
        />
        <Segmented
          label="Liṅga"
          value={linga}
          onChange={(l) => set({ linga: l === 'pum' ? undefined : l })}
          options={LINGAS.map((l) => ({
            id: l.id,
            label: <span className="font-devanagari">{l.label}</span>
          }))}
        />
      </div>
      {search.from && stem ? (
        <p className="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
          Derived stem
          <Badge variant="secondary" className="font-devanagari text-sm">
            {stem} ← {derivedLabel(search.from)}
          </Badge>
          <button
            type="button"
            className="inline-flex items-center gap-1 text-xs hover:text-foreground"
            onClick={() => set({ from: undefined })}
          >
            <XIcon className="size-3" aria-hidden /> treat as a plain stem
          </button>
        </p>
      ) : null}

      {stem && !search.from ? <FormSuggestions text={stem} tool="nama" /> : null}

      {!stemArgs || !stem ? (
        <Empty className="border border-dashed">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <ShapesIcon aria-hidden />
            </EmptyMedia>
            <EmptyTitle>Type a stem</EmptyTitle>
            <EmptyDescription>
              The dictionary form without case ending: <span className="font-devanagari">राम</span>,{' '}
              <span className="font-devanagari">मति</span>,{' '}
              <span className="font-devanagari">राजन्</span>,{' '}
              <span className="font-devanagari">मनस्</span>.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <Tabs
          value={search.tab ?? 'sup'}
          onValueChange={(v) => set({ tab: v === 'sup' ? undefined : (v as Tab) })}
        >
          <TabsList className="h-auto flex-wrap">
            <TabsTrigger value="sup">
              Declension <span className="font-devanagari text-xs">सुबन्तम्</span>
            </TabsTrigger>
            <TabsTrigger value="stri">
              Feminine <span className="font-devanagari text-xs">स्त्रीप्रत्ययः</span>
            </TabsTrigger>
            <TabsTrigger value="tad">
              Taddhita <span className="font-devanagari text-xs">तद्धितः</span>
            </TabsTrigger>
            <TabsTrigger value="sam">
              Compound <span className="font-devanagari text-xs">समासः</span>
            </TabsTrigger>
          </TabsList>
          <TabsContent value="sup" className="pt-2">
            <Declension stem={stemArgs} linga={linga} />
          </TabsContent>
          <TabsContent value="stri" className="pt-2">
            <Feminine stem={stemArgs} />
          </TabsContent>
          <TabsContent value="tad" className="pt-2">
            <Taddhitas stem={stem} />
          </TabsContent>
          <TabsContent value="sam" className="pt-2">
            <Compound key={stem} first={stem} />
          </TabsContent>
        </Tabs>
      )}
    </div>
  );
}

function derivedLabel(s: StemArgs): string {
  switch (s.kind) {
    case 'krdanta':
      return 'kṛdanta';
    case 'taddhitanta':
      return `${s.stem} + taddhita`;
    case 'samasa':
      return s.padas.map((p) => p.stem).join(' + ');
    default:
      return '';
  }
}

function Declension({ stem, linga }: { stem: StemArgs; linga: LingaId }) {
  const q = useQuery(subantaQuery(stem, linga));
  if (q.isPending) return <LoadingBlock rows={6} label="Declining" />;
  if (q.isError) return <ErrorNote error={q.error} />;
  const name = q.data.stems.join(' / ');
  return (
    <FormsTable
      className="max-w-3xl"
      caption={
        <span className="font-devanagari text-base">
          {name}{' '}
          <span className="text-sm font-normal text-muted-foreground">· {LINGA_LABEL[linga]}</span>
        </span>
      }
      rows={VIBHAKTIS.map((v) => v.label)}
      cols={VACANAS.map((v) => v.label)}
      cells={q.data.forms}
      specFor={(ri, ci) => ({
        kind: 'subanta',
        stem,
        linga,
        vibhakti: VIBHAKTIS[ri]!.id,
        vacana: VACANAS[ci]!.id
      })}
      contextFor={(ri, ci) =>
        `${name} · ${LINGA_LABEL[linga]} · ${VIBHAKTIS[ri]!.label} ${VACANAS[ci]!.label}`
      }
    />
  );
}

function Feminine({ stem }: { stem: StemArgs }) {
  const q = useQuery(stryantaQuery(stem));
  if (q.isPending) return <LoadingBlock rows={1} label="Deriving feminine stems" />;
  if (q.isError) return <ErrorNote error={q.error} />;
  return (
    <div className="flex max-w-2xl flex-col gap-3">
      {q.data.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          vidyut forms no feminine stem from this one.
        </p>
      ) : (
        <ul className="flex flex-col divide-y rounded-xl border bg-card">
          {q.data.map((f) => (
            <li key={f} className="flex items-baseline gap-3 px-3 py-2">
              <FormButton
                form={f}
                spec={{ kind: 'stryanta', stem }}
                context="स्त्रीप्रत्ययः"
                className="text-lg"
              />
              <Link
                to="/vidyut/nama"
                search={{ stem: f, linga: 'stri' }}
                className="ml-auto text-xs text-primary hover:underline"
              >
                Decline →
              </Link>
            </li>
          ))}
        </ul>
      )}
      <p className="text-xs text-muted-foreground">
        Declining a masculine stem in स्त्रीलिङ्गम् also adds the feminine suffix on the way (अज → अजा).
      </p>
    </div>
  );
}

function Taddhitas({ stem }: { stem: string }) {
  const q = useQuery(taddhitaQuery(stem));
  const [all, setAll] = useState(false);
  if (q.isPending) return <LoadingBlock rows={5} label="Deriving taddhitas" />;
  if (q.isError) return <ErrorNote error={q.error} />;
  const common = q.data.filter((t) => t.gloss);
  const rest = q.data.filter((t) => !t.gloss);
  const shown = all ? [...common, ...rest] : common;
  return (
    <div className="flex flex-col gap-2">
      <ul className="grid grid-cols-1 gap-x-6 rounded-xl border bg-card px-3 md:grid-cols-2">
        {shown.map((t) => {
          const spec: StemArgs = { kind: 'taddhitanta', stem, taddhita: t.taddhita };
          return (
            <li key={t.taddhita} className="flex flex-wrap items-baseline gap-x-3 border-b py-2">
              <span className="w-32 shrink-0">
                <span className="font-devanagari font-medium">{t.label}</span>
                {t.gloss ? (
                  <span className="block text-[11px] leading-tight text-muted-foreground">
                    {t.gloss}
                  </span>
                ) : null}
              </span>
              <span className="flex flex-1 flex-wrap gap-x-2">
                {t.forms.map((f) => (
                  <FormButton
                    key={f}
                    form={f}
                    spec={{ kind: 'stem', stem: spec }}
                    context={`${stem} + ${t.label}`}
                    className="text-base"
                  />
                ))}
              </span>
              <Link
                to="/vidyut/nama"
                search={{ stem: t.forms[0], from: spec, linga: t.linga }}
                className="text-xs text-primary hover:underline"
              >
                Decline →
              </Link>
            </li>
          );
        })}
      </ul>
      {rest.length > 0 ? (
        <Button variant="ghost" size="sm" className="self-start" onClick={() => setAll((v) => !v)}>
          {all ? 'Common suffixes only' : `All suffixes that apply (${q.data.length})`}
        </Button>
      ) : null}
    </div>
  );
}

const VIBHAKTI_ITEMS = Object.fromEntries(VIBHAKTIS.slice(0, 7).map((v) => [v.id, v.label]));

function Compound({ first }: { first: string }) {
  const [kind, setKind] = useState<SamasaKind>('tatpurusha');
  const [members, setMembers] = useState<SamasaPada[]>([
    { stem: first, vibhakti: 'sasthi', linga: 'pum' },
    { stem: '', vibhakti: 'prathama', linga: 'pum' }
  ]);
  const [asked, setAsked] = useState<{ padas: SamasaPada[]; kind: SamasaKind } | null>(null);
  const q = useQuery({
    ...samasaQuery(asked?.padas ?? [], asked?.kind ?? 'tatpurusha'),
    enabled: asked !== null
  });
  const update = (i: number, patch: Partial<SamasaPada>) =>
    setMembers((ms) => ms.map((m, j) => (j === i ? { ...m, ...patch } : m)));
  const ready = members.every((m) => m.stem.trim());
  const usesCase = kind === 'tatpurusha';

  return (
    <div className="flex max-w-3xl flex-col gap-3">
      <Segmented
        size="sm"
        label="Samāsa type"
        value={kind}
        onChange={setKind}
        options={SAMASAS.map((s) => ({
          id: s.id,
          hint: s.hint,
          label: <span className="font-devanagari">{s.label}</span>
        }))}
      />
      <ol className="flex flex-col gap-2">
        {members.map((m, i) => (
          <li key={i} className="flex items-center gap-2">
            <span className="w-5 text-right text-xs text-muted-foreground tabular-nums">
              {i + 1}
            </span>
            <MemberInput
              value={m.stem}
              onChange={(v) => update(i, { stem: v })}
              placeholder={i === 0 && kind === 'avyayibhava' ? 'Indeclinable: उप, यथा…' : 'Stem'}
            />
            {usesCase && i < members.length - 1 ? (
              <Select
                items={VIBHAKTI_ITEMS}
                value={m.vibhakti ?? 'prathama'}
                onValueChange={(v) => update(i, { vibhakti: (v ?? 'prathama') as VibhaktiId })}
              >
                <SelectTrigger className="h-10 w-32 rounded-xl font-devanagari" aria-label="Case">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {Object.entries(VIBHAKTI_ITEMS).map(([id, label]) => (
                    <SelectItem key={id} value={id} className="font-devanagari">
                      {label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : null}
            {members.length > 2 ? (
              <Button
                variant="ghost"
                size="icon-sm"
                aria-label="Remove member"
                onClick={() => setMembers((ms) => ms.filter((_, j) => j !== i))}
              >
                <XIcon aria-hidden />
              </Button>
            ) : null}
          </li>
        ))}
      </ol>
      <div className="flex gap-2">
        {members.length < 6 ? (
          <Button
            variant="outline"
            size="sm"
            onClick={() =>
              setMembers((ms) => [...ms, { stem: '', vibhakti: 'prathama', linga: 'pum' }])
            }
          >
            <PlusIcon aria-hidden /> Member
          </Button>
        ) : null}
        <Button
          size="sm"
          disabled={!ready}
          onClick={() =>
            setAsked({
              kind,
              padas: members.map((m) => ({
                ...m,
                stem: m.stem.trim(),
                vibhakti: usesCase ? m.vibhakti : 'prathama'
              }))
            })
          }
        >
          Form compound
        </Button>
      </div>
      {asked ? (
        q.isPending ? (
          <LoadingBlock rows={1} label="Forming compound" />
        ) : q.isError ? (
          <ErrorNote error={q.error} />
        ) : (
          <div className="flex flex-col gap-2 rounded-xl border bg-card p-3">
            <div className="flex flex-wrap items-baseline gap-3">
              <span className="text-xs text-muted-foreground">Stem</span>
              {q.data.stems.map((s) => (
                <FormButton
                  key={s}
                  form={s}
                  spec={{ kind: 'stem', stem: samasaStem(asked) }}
                  context="समासः"
                  className="text-xl font-semibold"
                />
              ))}
              <Link
                to="/vidyut/nama"
                search={{
                  stem: q.data.stems[0],
                  from: samasaStem(asked),
                  linga: q.data.words[0]?.linga
                }}
                className="ml-auto text-xs text-primary hover:underline"
              >
                Decline →
              </Link>
            </div>
            <div className="flex flex-wrap gap-x-5 gap-y-1">
              {q.data.words.map((w) => (
                <span key={w.linga} className="flex items-baseline gap-1.5">
                  <span className="font-devanagari text-xs text-muted-foreground">
                    {LINGAS.find((l) => l.id === w.linga)?.short}
                  </span>
                  {w.forms.map((f) => (
                    <FormButton
                      key={f}
                      form={f}
                      spec={{
                        kind: 'subanta',
                        stem: samasaStem(asked),
                        linga: w.linga,
                        vibhakti: 'prathama',
                        vacana: asked.kind === 'dvandva' ? 'dvi' : 'eka'
                      }}
                      context={`समासः · ${LINGA_LABEL[w.linga]}`}
                    />
                  ))}
                </span>
              ))}
            </div>
          </div>
        )
      ) : null}
    </div>
  );
}

function samasaStem(a: { padas: SamasaPada[]; kind: SamasaKind }): StemArgs {
  return { kind: 'samasa', padas: a.padas, samasa_type: a.kind };
}

function MemberInput({
  value,
  onChange,
  placeholder
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
}) {
  return (
    <DevaInput
      value={value}
      onChange={onChange}
      label="Compound member"
      placeholder={placeholder}
      className={cn('max-w-64 flex-1')}
    />
  );
}
