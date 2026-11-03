import { Link, createFileRoute } from '@tanstack/react-router';
import { ArrowRightIcon } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { AmbudaCredit, TOOLS } from '@/components/vidyut/tools';

export const Route = createFileRoute('/vidyut/')({
  component: VidyutIndex
});

/** A few ready-made starting points per tool. */
const EXAMPLES: Record<(typeof TOOLS)[number]['to'], { label: string; search: object }[]> = {
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
    { label: 'Scan a verse', search: {} },
    { label: 'Meters', search: { tab: 'meters' } }
  ],
  '/vidyut/sandhi': [
    { label: 'रामः + अस्ति', search: { tab: 'join', q: 'रामः अस्ति' } },
    { label: 'तथैव', search: { tab: 'split', q: 'तथैव' } }
  ]
};

function VidyutIndex() {
  return (
    <div className="flex flex-col gap-6">
      <p className="max-w-2xl text-sm text-muted-foreground">
        Pāṇinian word generation, meter and sandhi, computed on the spot. Every generated form can
        be opened to see its derivation, sūtra by sūtra. Type in Devanagari, or type romanized (bhU,
        rAma) and it turns into Devanagari as you go.
      </p>
      <div className="grid gap-3 md:grid-cols-2">
        {TOOLS.map(({ to, label, deva, icon: Icon, blurb }) => (
          <Card
            key={to}
            className="group relative py-0 transition-all hover:-translate-y-px hover:border-primary/40 hover:shadow-md"
          >
            <CardContent className="flex flex-col gap-3 p-4 sm:p-5">
              <Link to={to} className="flex items-center gap-3 after:absolute after:inset-0">
                <span className="grid size-10 place-items-center rounded-xl bg-primary/10 text-primary">
                  <Icon className="size-5" aria-hidden />
                </span>
                <span className="flex flex-col leading-tight">
                  <span className="font-semibold">{label}</span>
                  <span className="font-devanagari text-sm text-muted-foreground">{deva}</span>
                </span>
                <ArrowRightIcon
                  className="ml-auto size-4 text-muted-foreground transition-transform group-hover:translate-x-0.5"
                  aria-hidden
                />
              </Link>
              <p className="text-sm text-muted-foreground">{blurb}</p>
              <div className="relative z-10 flex flex-wrap gap-1.5">
                {EXAMPLES[to].map((e) => (
                  <Link
                    key={e.label}
                    to={to}
                    search={e.search as never}
                    className="rounded-full border px-2.5 py-0.5 font-devanagari text-xs text-muted-foreground transition-colors hover:border-primary/50 hover:text-foreground"
                  >
                    {e.label}
                  </Link>
                ))}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
      <AmbudaCredit className="max-w-2xl" />
    </div>
  );
}
