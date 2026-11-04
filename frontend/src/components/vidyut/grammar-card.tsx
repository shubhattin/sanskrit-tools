// Dictionary search: what vidyut's grammar makes of a Sanskrit word the dictionaries' inflection
// tables don't cover (उद्भवनम्, गमयिष्यति), or how a joined word splits (तथैव).
import { Link } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { ArrowRightIcon, TableIcon, ZapIcon } from 'lucide-react';
import { padaQuery, type PadaInfo } from '@/lib/vidyut/api';
import { padaTags } from '@/lib/vidyut/labels';
import { KIND, formsLink } from './words';

const DEVA_WORD = /^[ऀ-ॿ‌‍]+$/;

export function GrammarCard({ q, onPick }: { q: string; onPick: (word: string) => void }) {
  const word = q.trim();
  const enabled = DEVA_WORD.test(word);
  const res = useQuery({ ...padaQuery(word), enabled, retry: false });
  if (!enabled || !res.data) return null;
  const { word: part, split } = res.data;
  // Only what adds something: not the word as its own stem.
  const seen = new Set<string>();
  const analyses = part.analyses
    .filter((a) => a.lemma !== part.text || a.kind === 'tinanta')
    .filter((a) => {
      const k = `${a.lemma}|${a.dhatu}|${padaTags(a)}`;
      if (seen.has(k)) return false;
      seen.add(k);
      return true;
    })
    .slice(0, 4);
  if (!analyses.length && !split) return null;
  return (
    <section className="relative animate-in overflow-hidden rounded-2xl border border-amber-500/25 bg-gradient-to-br from-amber-500/[0.07] via-card to-card p-4 duration-500 fade-in-0">
      <h2 className="mb-3 flex items-center gap-2 text-xs font-semibold tracking-widest text-amber-700 uppercase dark:text-amber-300">
        <ZapIcon className="size-3.5" aria-hidden /> Grammar · vidyut
      </h2>
      {analyses.length ? (
        <ul className="flex flex-col divide-y divide-border/60">
          {analyses.map((a) => (
            <AnalysisRow key={a.key} a={a} form={part.text} onPick={onPick} />
          ))}
        </ul>
      ) : split ? (
        <p className="flex flex-wrap items-baseline gap-x-2 gap-y-1 text-sm">
          <span className="font-devanagari text-lg font-semibold">{part.text}</span>
          <span className="text-muted-foreground">splits as</span>
          {split.parts.map((p, i) => (
            <span key={i} className="flex items-baseline gap-2">
              {i > 0 ? <span className="text-muted-foreground">+</span> : null}
              <button
                type="button"
                onClick={() => onPick(p.text)}
                className="font-devanagari text-lg font-semibold text-primary underline-offset-4 hover:underline"
              >
                {p.text}
              </button>
            </span>
          ))}
        </p>
      ) : null}
      <Link
        to="/vidyut/sandhi"
        search={{ tab: 'split', q: part.text }}
        className="mt-3 inline-flex items-center gap-1 text-xs font-medium text-amber-700 hover:underline dark:text-amber-300"
      >
        Derivations and more in Vidyut <ArrowRightIcon className="size-3" aria-hidden />
      </Link>
    </section>
  );
}

function AnalysisRow({
  a,
  form,
  onPick
}: {
  a: PadaInfo;
  form: string;
  onPick: (word: string) => void;
}) {
  const forms = formsLink(a);
  const kind = KIND[a.kind];
  return (
    <li className="flex flex-wrap items-center gap-x-2 gap-y-1.5 py-2 text-sm first:pt-0 last:pb-0">
      <span className="font-devanagari text-lg font-semibold">{form}</span>
      <ArrowRightIcon className="size-4 text-muted-foreground" aria-hidden />
      <button
        type="button"
        className="font-devanagari text-lg font-semibold text-primary underline-offset-4 hover:underline"
        onClick={() => onPick(a.lemma)}
      >
        {a.lemma}
      </button>
      {a.dhatu && a.krt ? (
        <span className="text-xs text-muted-foreground">
          from{' '}
          <button
            type="button"
            className="font-devanagari text-sm font-semibold text-primary underline-offset-4 hover:underline"
            onClick={() => onPick(a.dhatu!)}
          >
            {a.dhatu}
          </button>{' '}
          + <span className="font-devanagari">{a.krt}</span>
        </span>
      ) : null}
      <span className="inline-flex items-center gap-1.5 rounded-full bg-muted px-2 py-0.5 font-devanagari text-xs text-muted-foreground">
        <span className={`size-1.5 rounded-full ${kind.dot}`} aria-hidden />
        {a.kind === 'avyaya' ? kind.label : padaTags(a)}
      </span>
      {forms ? (
        <Link
          to={forms.to}
          search={forms.search as never}
          className="ml-auto inline-flex h-6 items-center gap-1 rounded-full border px-2.5 text-xs font-medium transition-colors hover:bg-muted"
        >
          <TableIcon className="size-3" aria-hidden /> {forms.label}
        </Link>
      ) : null}
    </li>
  );
}
