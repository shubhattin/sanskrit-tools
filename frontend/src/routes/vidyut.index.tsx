import { useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { ArrowRightIcon, HeartHandshakeIcon, ScanTextIcon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { DevaTextarea } from '@/components/vidyut/controls';
import { CHEDA_SAMPLES } from '@/components/vidyut/reading';
import { cn } from '@/lib/utils';
import { AmbudaCredit, TOOLS } from '@/components/vidyut/tools';

export const Route = createFileRoute('/vidyut/')({
  component: VidyutIndex
});

/** A few ready-made starting points per tool. */
type ToolPath = Exclude<(typeof TOOLS)[number]['to'], '/vidyut/cheda'>;

const EXAMPLES: Record<ToolPath, { label: string; search: object }[]> = {
  '/vidyut/dhatu': [
    { label: 'भू', search: { code: '01.0001' } },
    { label: 'कृ', search: { code: '08.0010' } },
    { label: 'गम्', search: { code: '01.1137' } },
    { label: 'प्र + नी', search: { code: '01.1049', upa: ['प्र'] } }
  ],
  '/vidyut/nama': [
    { label: 'राम', search: { stem: 'राम', linga: 'pum' } },
    { label: 'मति', search: { stem: 'मति', linga: 'stri' } },
    { label: 'राजन्', search: { stem: 'राजन्', linga: 'pum' } },
    { label: 'मनस्', search: { stem: 'मनस्', linga: 'napumsaka' } }
  ],
  '/vidyut/chandas': [
    {
      label: 'गीता १.१',
      search: { v: 'धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः।\nमामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय॥' }
    },
    { label: 'मातः समस्त…', search: { v: 'मातः समस्तजगतां मधुकैटभारेः' } },
    { label: 'Meters', search: { tab: 'meters' } }
  ],
  '/vidyut/sandhi': [
    { label: 'रामः + अस्ति', search: { tab: 'join', q: 'रामः अस्ति' } },
    { label: 'तथैव', search: { tab: 'split', q: 'तथैव' } },
    { label: 'पाण्डवाश्चैव', search: { tab: 'split', q: 'पाण्डवाश्चैव' } },
    { label: 'Rules', search: { tab: 'rules' } }
  ]
};

/** Per-tool accent (literal class names so Tailwind keeps them). */
const THEME: Record<
  ToolPath,
  { glyph: string; tile: string; glow: string; chip: string; text: string }
> = {
  '/vidyut/dhatu': {
    glyph: 'धा',
    tile: 'from-emerald-400 to-teal-600 shadow-emerald-500/30',
    glow: 'from-emerald-500/12 hover:border-emerald-500/40',
    chip: 'border-emerald-500/30 bg-emerald-500/8 hover:bg-emerald-500/15',
    text: 'text-emerald-700 dark:text-emerald-400'
  },
  '/vidyut/nama': {
    glyph: 'ना',
    tile: 'from-sky-400 to-indigo-600 shadow-sky-500/30',
    glow: 'from-sky-500/12 hover:border-sky-500/40',
    chip: 'border-sky-500/30 bg-sky-500/8 hover:bg-sky-500/15',
    text: 'text-sky-700 dark:text-sky-400'
  },
  '/vidyut/chandas': {
    glyph: 'छ',
    tile: 'from-fuchsia-400 to-violet-600 shadow-violet-500/30',
    glow: 'from-violet-500/12 hover:border-violet-500/40',
    chip: 'border-violet-500/30 bg-violet-500/8 hover:bg-violet-500/15',
    text: 'text-violet-700 dark:text-violet-400'
  },
  '/vidyut/sandhi': {
    glyph: 'स',
    tile: 'from-amber-400 to-orange-600 shadow-orange-500/30',
    glow: 'from-amber-500/12 hover:border-amber-500/40',
    chip: 'border-amber-500/30 bg-amber-500/8 hover:bg-amber-500/15',
    text: 'text-amber-800 dark:text-amber-400'
  }
};

function VidyutIndex() {
  return (
    <div className="flex flex-col gap-4">
      <ChedaCard />
      <div className="grid gap-4 md:grid-cols-2">
        {TOOLS.filter((t) => t.to !== '/vidyut/cheda').map(
          ({ to, label, deva, icon: Icon, blurb }) => {
            const t = THEME[to];
            return (
              <article
                key={to}
                className={cn(
                  'group relative flex flex-col gap-4 overflow-hidden rounded-2xl border bg-gradient-to-br via-card to-card p-5 shadow-sm transition-all duration-300 hover:-translate-y-0.5 hover:shadow-lg',
                  t.glow
                )}
              >
                <span
                  aria-hidden
                  className={cn(
                    'pointer-events-none absolute -right-2 -bottom-8 font-devanagari text-[8rem] leading-none font-black opacity-[0.07] transition-transform duration-500 select-none group-hover:scale-110 group-hover:-rotate-6',
                    t.text
                  )}
                >
                  {t.glyph}
                </span>
                <Link to={to} className="flex items-center gap-3 after:absolute after:inset-0">
                  <span
                    className={cn(
                      'grid size-12 place-items-center rounded-2xl bg-gradient-to-br text-white shadow-lg transition-transform duration-300 group-hover:scale-105 group-hover:-rotate-3',
                      t.tile
                    )}
                  >
                    <Icon className="size-6" aria-hidden />
                  </span>
                  <span className="flex flex-col leading-tight">
                    <span className="text-lg font-semibold">{label}</span>
                    <span className={cn('font-devanagari text-sm font-medium', t.text)}>
                      {deva}
                    </span>
                  </span>
                </Link>
                <p className="relative text-sm leading-relaxed text-muted-foreground">{blurb}</p>
                <div className="relative z-10 mt-auto flex flex-wrap items-center gap-1.5">
                  {EXAMPLES[to].map((e) => (
                    <Link
                      key={e.label}
                      to={to}
                      search={e.search as never}
                      className={cn(
                        'rounded-full border px-2.5 py-0.5 font-devanagari text-xs text-foreground/80 transition-colors',
                        t.chip
                      )}
                    >
                      {e.label}
                    </Link>
                  ))}
                  <span
                    className={cn(
                      'ml-auto inline-flex items-center gap-1 text-xs font-medium',
                      t.text
                    )}
                    aria-hidden
                  >
                    Open
                    <ArrowRightIcon className="size-3.5 transition-transform group-hover:translate-x-0.5" />
                  </span>
                </div>
              </article>
            );
          }
        )}
      </div>

      <div className="flex items-start gap-3 rounded-2xl border bg-card/60 p-4">
        <HeartHandshakeIcon className="mt-0.5 size-5 shrink-0 text-rose-500" aria-hidden />
        <AmbudaCredit className="text-sm" />
      </div>
    </div>
  );
}

/** The reader, up front: paste a verse and go. */
function ChedaCard() {
  const navigate = useNavigate();
  const [text, setText] = useState('');
  const read = (q: string) => void navigate({ to: '/vidyut/cheda', search: { q } });
  return (
    <section
      aria-labelledby="cheda-card-title"
      className="relative grid gap-5 overflow-hidden rounded-2xl border bg-gradient-to-br from-rose-500/10 via-card to-card p-5 shadow-sm md:grid-cols-[minmax(0,2fr)_minmax(0,3fr)]"
    >
      <span
        aria-hidden
        className="pointer-events-none absolute -bottom-10 -left-2 font-devanagari text-[9rem] leading-none font-black text-rose-700 opacity-[0.06] select-none dark:text-rose-400"
      >
        छे
      </span>
      <div className="relative flex flex-col gap-3">
        <Link to="/vidyut/cheda" className="group flex items-center gap-3 self-start">
          <span className="grid size-12 place-items-center rounded-2xl bg-gradient-to-br from-rose-400 to-pink-600 text-white shadow-lg shadow-rose-500/30 transition-transform duration-300 group-hover:scale-105 group-hover:-rotate-3">
            <ScanTextIcon className="size-6" aria-hidden />
          </span>
          <span className="flex flex-col leading-tight">
            <span id="cheda-card-title" className="text-lg font-semibold group-hover:underline">
              Cheda
            </span>
            <span className="font-devanagari text-sm font-medium text-rose-700 dark:text-rose-400">
              पदच्छेदः
            </span>
          </span>
        </Link>
        <p className="text-sm leading-relaxed text-muted-foreground">
          Paste a śloka or a sentence. Its sandhi is undone and every word tagged: stem or root,
          case or tense, number. cheda and Jev read the text as a whole, and each word opens its
          derivation and forms.
        </p>
      </div>
      <form
        className="relative flex flex-col gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          if (text.trim()) read(text.trim());
        }}
      >
        <DevaTextarea
          value={text}
          onChange={setText}
          label="Text to read"
          rows={3}
          placeholder={'धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः।\nमामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय॥'}
        />
        <div className="flex flex-wrap items-center gap-1.5">
          <Button type="submit" size="sm" disabled={!text.trim()}>
            <ScanTextIcon aria-hidden /> Read
          </Button>
          <span className="ml-1 text-xs text-muted-foreground">or try</span>
          {CHEDA_SAMPLES.map((s) => (
            <Link
              key={s.label}
              to="/vidyut/cheda"
              search={{ q: s.text }}
              className="rounded-full border border-rose-500/30 bg-rose-500/8 px-2.5 py-0.5 font-devanagari text-xs text-foreground/80 transition-colors hover:bg-rose-500/15"
            >
              {s.label}
            </Link>
          ))}
        </div>
      </form>
    </section>
  );
}
