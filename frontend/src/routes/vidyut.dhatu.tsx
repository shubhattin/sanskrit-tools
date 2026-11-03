import { useDeferredValue, useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { BookOpenIcon, PlusIcon, SearchIcon, XIcon } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { Button, buttonVariants } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Chip,
  DevaInput,
  ErrorNote,
  LoadingBlock,
  Segmented,
  SectionLabel
} from '@/components/vidyut/controls';
import { FormButton } from '@/components/vidyut/derivation';
import { FormsTable } from '@/components/vidyut/forms-table';
import { searchString } from '@/lib/search-params';
import {
  krdantaQuery,
  tinantaQuery,
  vRootQuery,
  vRootsQuery,
  type DhatuArgs,
  type KrdantaForm,
  type PadaId,
  type PrayogaId,
  type RootInfo,
  type SanadiId
} from '@/lib/vidyut/api';
import {
  GANAS,
  LAKARAS,
  LAKARA_LABEL,
  PADA_LABEL,
  PRAYOGAS,
  PURUSHAS,
  SANADIS,
  SANADI_LABEL,
  UPASARGAS,
  VACANAS,
  devaNum
} from '@/lib/vidyut/labels';
import { cn } from '@/lib/utils';

type Tab = 'tin' | 'krt';

interface DhatuSearch {
  code?: string;
  sanadi?: SanadiId[];
  upa?: string[];
  prayoga?: PrayogaId;
  tab?: Tab;
}

const SANADI_IDS = new Set(SANADIS.map((s) => s.id));
const UPA_SET = new Set<string>(UPASARGAS);

function list(v: unknown): unknown[] {
  return Array.isArray(v) ? v : typeof v === 'string' && v ? v.split(',') : [];
}

export const Route = createFileRoute('/vidyut/dhatu')({
  validateSearch: (s: Record<string, unknown>): DhatuSearch => {
    const sanadi = list(s.sanadi).filter((x): x is SanadiId => SANADI_IDS.has(x as SanadiId));
    const upa = list(s.upa).filter((x): x is string => typeof x === 'string' && UPA_SET.has(x));
    const prayoga = PRAYOGAS.find((p) => p.id === s.prayoga)?.id;
    return {
      code: searchString(s.code),
      sanadi: sanadi.length ? sanadi : undefined,
      upa: upa.length ? upa : undefined,
      prayoga: prayoga && prayoga !== 'kartari' ? prayoga : undefined,
      tab: s.tab === 'krt' ? 'krt' : undefined
    };
  },
  component: DhatuTool
});

function DhatuTool() {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: Route.fullPath });
  const set = (patch: Partial<DhatuSearch>) =>
    void navigate({ search: (prev) => ({ ...prev, ...patch }), replace: false });

  return (
    <div className="flex flex-col gap-4">
      <RootPicker
        hasRoot={!!search.code}
        onPick={(r) => set({ code: r.code, sanadi: undefined, upa: undefined })}
      />
      {search.code ? (
        <RootWorkspace
          code={search.code}
          sanadi={search.sanadi ?? []}
          upa={search.upa ?? []}
          prayoga={search.prayoga ?? 'kartari'}
          tab={search.tab ?? 'tin'}
          set={set}
        />
      ) : null}
    </div>
  );
}

// ---- root search ------------------------------------------------------------------------------

function RootPicker({ hasRoot, onPick }: { hasRoot: boolean; onPick: (r: RootInfo) => void }) {
  const [text, setText] = useState('');
  const [gana, setGana] = useState(0);
  const q = useDeferredValue(text.trim());
  const browsing = !hasRoot || q.length > 0 || gana > 0;
  const roots = useQuery({ ...vRootsQuery(q, gana || undefined), enabled: browsing });

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-col gap-2 sm:flex-row">
        <DevaInput
          value={text}
          onChange={setText}
          label="Find a root"
          placeholder={hasRoot ? 'Another root… (भू, एध्, 1.1)' : 'Find a root: भू, गम्, एध्, 1.1…'}
          icon={<SearchIcon aria-hidden />}
          className="sm:max-w-md"
          autoFocus={!hasRoot}
        />
        <Segmented
          size="sm"
          label="Gaṇa"
          value={String(gana)}
          onChange={(v) => setGana(Number(v))}
          options={[
            { id: '0', label: 'All' },
            ...GANAS.map((g, i) => ({
              id: String(i + 1),
              label: <span className="font-devanagari">{devaNum(i + 1)}</span>,
              hint: g
            }))
          ]}
          className="self-start"
        />
      </div>
      {browsing ? (
        roots.isPending ? (
          <LoadingBlock rows={2} label="Searching roots" />
        ) : roots.isError ? (
          <ErrorNote error={roots.error} />
        ) : roots.data.length === 0 ? (
          <p className="text-sm text-muted-foreground">No root found.</p>
        ) : (
          <ul className="grid max-h-80 grid-cols-1 gap-1.5 overflow-y-auto rounded-xl sm:grid-cols-2 lg:grid-cols-3">
            {roots.data.map((r) => (
              <li key={r.code}>
                <button
                  type="button"
                  onClick={() => {
                    setText('');
                    setGana(0);
                    onPick(r);
                  }}
                  className="flex w-full items-baseline gap-2 rounded-xl border bg-card px-3 py-2 text-left transition-colors hover:border-primary/50 hover:bg-primary/5"
                >
                  <span className="font-devanagari text-base font-semibold">{r.aupadeshika}</span>
                  <span className="font-devanagari text-sm text-muted-foreground">
                    {r.roots.join(' / ')} · {r.artha}
                  </span>
                  <span className="ml-auto font-mono text-[11px] text-muted-foreground tabular-nums">
                    {r.code}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )
      ) : null}
    </div>
  );
}

// ---- selected root ----------------------------------------------------------------------------

function RootWorkspace({
  code,
  sanadi,
  upa,
  prayoga,
  tab,
  set
}: {
  code: string;
  sanadi: SanadiId[];
  upa: string[];
  prayoga: PrayogaId;
  tab: Tab;
  set: (patch: Partial<DhatuSearch>) => void;
}) {
  const root = useQuery(vRootQuery(code));
  const dhatu: DhatuArgs = { code, sanadi, upasargas: upa };

  const toggleSanadi = (id: SanadiId) => {
    const next = sanadi.includes(id) ? sanadi.filter((x) => x !== id) : [...sanadi, id];
    set({ sanadi: next.length ? next : undefined });
  };
  const toggleUpa = (u: string) => {
    const next = upa.includes(u) ? upa.filter((x) => x !== u) : [...upa, u];
    set({ upa: next.length ? next : undefined });
  };

  if (root.isPending) return <LoadingBlock rows={3} label="Loading root" />;
  if (root.isError) return <ErrorNote error={root.error} />;
  const r = root.data.root;

  return (
    <>
      <Card className="py-0">
        <CardContent className="flex flex-col gap-3 p-4">
          <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
            <h1 className="font-devanagari text-3xl font-bold">{r.aupadeshika}</h1>
            <span className="font-devanagari text-lg text-muted-foreground">
              {r.roots.join(' / ')} · {r.artha}
            </span>
            <Badge variant="secondary" className="font-devanagari">
              {GANAS[r.gana - 1]} · {devaNum(`${r.gana}.${r.number}`)}
            </Badge>
            {root.data.dhatu_id ? (
              <Link
                to="/dhatu/$id"
                params={{ id: String(root.data.dhatu_id) }}
                className={cn(buttonVariants({ variant: 'ghost', size: 'sm' }), 'ml-auto')}
              >
                <BookOpenIcon aria-hidden /> Dhātu works
              </Link>
            ) : null}
          </div>
          <div className="flex flex-col gap-3 sm:flex-row sm:gap-8">
            <div className="flex flex-col gap-1.5">
              <SectionLabel>Sanādi</SectionLabel>
              <div className="flex flex-wrap gap-1.5">
                {SANADIS.map((s) => (
                  <Chip
                    key={s.id}
                    on={sanadi.includes(s.id)}
                    order={sanadi.length > 1 ? sanadi.indexOf(s.id) + 1 : undefined}
                    title={s.hint}
                    onClick={() => toggleSanadi(s.id)}
                  >
                    {s.label}
                    <span className="font-sans text-[11px] text-muted-foreground">{s.hint}</span>
                  </Chip>
                ))}
              </div>
            </div>
            <div className="flex flex-col gap-1.5">
              <SectionLabel>Upasarga</SectionLabel>
              <div className="flex flex-wrap items-center gap-1.5">
                {upa.map((u) => (
                  <Chip key={u} on onClick={() => toggleUpa(u)} title="Remove">
                    {u}
                    <XIcon className="size-3" aria-hidden />
                  </Chip>
                ))}
                <UpasargaPicker chosen={upa} onToggle={toggleUpa} />
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Tabs value={tab} onValueChange={(v) => set({ tab: v === 'krt' ? 'krt' : undefined })}>
        <TabsList>
          <TabsTrigger value="tin">
            Conjugation <span className="font-devanagari text-xs opacity-70">तिङन्तम्</span>
          </TabsTrigger>
          <TabsTrigger value="krt">
            Kṛdanta <span className="font-devanagari text-xs opacity-70">कृदन्तम्</span>
          </TabsTrigger>
        </TabsList>
        <TabsContent value="tin" className="pt-2">
          <Conjugations
            dhatu={dhatu}
            prayoga={prayoga}
            setPrayoga={(p) => set({ prayoga: p === 'kartari' ? undefined : p })}
          />
        </TabsContent>
        <TabsContent value="krt" className="pt-2">
          <Krdantas dhatu={dhatu} />
        </TabsContent>
      </Tabs>
    </>
  );
}

function UpasargaPicker({ chosen, onToggle }: { chosen: string[]; onToggle: (u: string) => void }) {
  return (
    <Popover>
      <PopoverTrigger render={<Button variant="outline" size="sm" className="h-7 rounded-full" />}>
        <PlusIcon aria-hidden /> Add
      </PopoverTrigger>
      <PopoverContent align="start" className="w-72">
        <p className="mb-2 text-xs text-muted-foreground">
          Pick in order: the first is outermost (उप + सम् + गम्).
        </p>
        <div className="flex flex-wrap gap-1.5">
          {UPASARGAS.map((u) => (
            <Chip
              key={u}
              on={chosen.includes(u)}
              order={chosen.includes(u) ? chosen.indexOf(u) + 1 : undefined}
              onClick={() => onToggle(u)}
            >
              {u}
            </Chip>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}

/** "प्र + भू + णिच्" — what a form's derivation starts from. */
function rootLabel(stems: string[], dhatu: DhatuArgs) {
  return [
    ...(dhatu.upasargas ?? []),
    stems[0] ?? '',
    ...(dhatu.sanadi ?? []).map((s) => SANADI_LABEL[s])
  ].join(' + ');
}

// ---- tiṅanta ------------------------------------------------------------------------------------

function Conjugations({
  dhatu,
  prayoga,
  setPrayoga
}: {
  dhatu: DhatuArgs;
  prayoga: PrayogaId;
  setPrayoga: (p: PrayogaId) => void;
}) {
  const q = useQuery(tinantaQuery(dhatu, prayoga));
  const [padaPick, setPadaPick] = useState<PadaId | null>(null);
  const tables = q.data?.tables ?? [];
  const pada = tables.find((t) => t.pada === padaPick) ?? tables[0];
  const base = q.data ? rootLabel([q.data.root.roots[0] ?? ''], dhatu) : '';

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <Segmented
          label="Prayoga"
          value={prayoga}
          onChange={setPrayoga}
          options={PRAYOGAS.map((p) => ({
            id: p.id,
            label: (
              <>
                <span className="font-devanagari">{p.label}</span>{' '}
                <span className="text-[11px] opacity-60">{p.hint}</span>
              </>
            )
          }))}
        />
        {tables.length > 1 ? (
          <Segmented
            label="Pada"
            value={pada?.pada ?? 'parasmaipada'}
            onChange={setPadaPick}
            options={tables.map((t) => ({
              id: t.pada,
              label: <span className="font-devanagari">{PADA_LABEL[t.pada]}</span>
            }))}
          />
        ) : pada ? (
          <Badge variant="outline" className="font-devanagari">
            {PADA_LABEL[pada.pada]}
          </Badge>
        ) : null}
        {q.data && q.data.stems.length > 0 && (dhatu.sanadi?.length || dhatu.upasargas?.length) ? (
          <span className="font-devanagari text-sm text-muted-foreground">
            {base} →{' '}
            {q.data.stems.map((s, i) => (
              <span key={s}>
                {i > 0 ? ' / ' : ''}
                <FormButton form={s} spec={{ kind: 'dhatu', dhatu }} context={base} />
              </span>
            ))}
          </span>
        ) : null}
      </div>
      {q.isPending ? (
        <LoadingBlock rows={4} label="Conjugating" />
      ) : q.isError ? (
        <ErrorNote error={q.error} />
      ) : pada ? (
        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
          {pada.lakaras.map((l) => (
            <FormsTable
              key={l.lakara}
              dense
              caption={
                <span className="flex items-baseline gap-2">
                  <span className="font-devanagari text-base">{LAKARA_LABEL[l.lakara]}</span>
                  <span className="text-xs font-normal text-muted-foreground">
                    {LAKARAS.find((x) => x.id === l.lakara)?.hint}
                  </span>
                </span>
              }
              rows={PURUSHAS.map((p) => p.label.replace('पुरुषः', ''))}
              cols={VACANAS.map((v) => v.label.replace('वचनम्', ''))}
              cells={l.forms}
              specFor={(ri, ci) => ({
                kind: 'tinanta',
                dhatu,
                prayoga,
                lakara: l.lakara,
                purusha: PURUSHAS[ri]!.id,
                vacana: VACANAS[ci]!.id,
                pada: pada.pada
              })}
              contextFor={(ri, ci) =>
                `${base} · ${LAKARA_LABEL[l.lakara]} · ${PURUSHAS[ri]!.label} ${VACANAS[ci]!.label}`
              }
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

// ---- kṛdanta ------------------------------------------------------------------------------------

function krtTitle(k: KrdantaForm) {
  const extra = [k.lakara === 'lrt' ? 'लृट्' : null, k.prayoga === 'karmani' ? 'कर्मणि' : null].filter(
    Boolean
  );
  return extra.length ? `${k.label} (${extra.join(', ')})` : k.label;
}

function Krdantas({ dhatu }: { dhatu: DhatuArgs }) {
  const q = useQuery(krdantaQuery(dhatu));
  const root = useQuery(vRootQuery(dhatu.code));
  const [showOther, setShowOther] = useState(false);
  const base = rootLabel(root.data?.root.roots ?? [], dhatu);
  if (q.isPending) return <LoadingBlock rows={5} label="Deriving kṛdantas" />;
  if (q.isError) return <ErrorNote error={q.error} />;

  return (
    <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
      {q.data.map((g) => {
        const other = g.key === 'other';
        const items = other && !showOther ? g.items.slice(0, 6) : g.items;
        return (
          <section
            key={g.key}
            className={cn('rounded-xl border bg-card', other && 'lg:col-span-2')}
            aria-label={g.label}
          >
            <h2 className="border-b px-3 py-2 text-sm font-semibold">{g.label}</h2>
            <ul className={cn('divide-y', other && 'lg:grid lg:grid-cols-2 lg:divide-y-0')}>
              {items.map((k) => {
                const stem = {
                  kind: 'krdanta' as const,
                  dhatu,
                  krt: k.krt,
                  lakara: k.lakara ?? null,
                  prayoga: k.prayoga ?? null
                };
                return (
                  <li
                    key={`${k.krt}-${k.lakara ?? ''}-${k.prayoga ?? ''}`}
                    className={cn(
                      'flex flex-wrap items-baseline gap-x-3 gap-y-1 px-3 py-2',
                      other && 'lg:border-b'
                    )}
                  >
                    <span className="w-28 shrink-0">
                      <span className="font-devanagari font-medium">{krtTitle(k)}</span>
                      {k.gloss ? (
                        <span className="block text-[11px] leading-tight text-muted-foreground">
                          {k.gloss}
                        </span>
                      ) : null}
                    </span>
                    <span className="flex min-w-0 flex-1 flex-wrap gap-x-2">
                      {k.forms.map((f) => (
                        <FormButton
                          key={f}
                          form={f}
                          spec={{ kind: 'stem', stem }}
                          context={`${base} + ${k.label}`}
                          className="text-base"
                        />
                      ))}
                    </span>
                    {k.lingas.length > 0 ? (
                      <Link
                        to="/vidyut/nama"
                        search={{ stem: k.forms[0], linga: k.lingas[0], from: stem }}
                        className="text-xs text-primary hover:underline"
                      >
                        Decline →
                      </Link>
                    ) : k.avyaya ? (
                      <span className="text-[11px] text-muted-foreground">avyaya</span>
                    ) : null}
                  </li>
                );
              })}
            </ul>
            {other && g.items.length > 6 ? (
              <Button
                variant="ghost"
                size="sm"
                className="m-1"
                onClick={() => setShowOther((v) => !v)}
              >
                {showOther ? 'Show fewer' : `Show all ${g.items.length}`}
              </Button>
            ) : null}
          </section>
        );
      })}
    </div>
  );
}
