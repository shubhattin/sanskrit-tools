import { createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { dictionariesQuery } from '@/lib/api/client';
import { DICT_KIND_LABELS, DICT_KIND_ORDER } from '@/lib/api/types';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';

export const Route = createFileRoute('/about')({
  component: AboutPage
});

function AboutPage() {
  const dicts = useQuery(dictionariesQuery());

  return (
    <div className="flex max-w-3xl flex-col gap-6">
      <div className="flex flex-col gap-2">
        <h1 className="text-2xl font-bold sm:text-3xl">About</h1>
        <p className="sa-text text-[15px] text-muted-foreground">
          <span className="font-devanagari font-semibold text-foreground">शब्दकोशः</span> searches
          Sanskrit dictionaries in book order: lexicons (kośas), thesauri, dhātu works, sūtra
          commentaries and concordances. Type in Devanagari, IAST, Harvard-Kyoto, ITRANS, SLP1 or
          lipilekhika “Normal” — the server normalizes every input to the same keys.
        </p>
      </div>

      <section aria-label="Dictionaries" className="flex flex-col gap-3">
        <h2 className="text-lg font-semibold">Dictionaries</h2>
        {dicts.isPending ? (
          <div className="flex flex-col gap-3" aria-label="Loading dictionaries">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className="h-28" />
            ))}
          </div>
        ) : dicts.isError ? (
          <p className="text-sm text-destructive">
            Could not load the dictionary list: {dicts.error.message}
          </p>
        ) : (
          DICT_KIND_ORDER.map((kind) => {
            const group = dicts.data
              .filter((d) => d.kind === kind)
              .sort((a, b) => a.sort_order - b.sort_order);
            if (group.length === 0) return null;
            return (
              <div key={kind} className="flex flex-col gap-2">
                <h3 className="text-sm font-semibold tracking-wide text-muted-foreground uppercase">
                  {DICT_KIND_LABELS[kind]}
                </h3>
                {group.map((d) => (
                  <Card key={d.slug}>
                    <CardHeader className="flex flex-row flex-wrap items-center gap-2 pb-2">
                      <Badge variant="secondary" className="font-bold">
                        {d.short_name}
                      </Badge>
                      <span className="font-semibold">{d.name}</span>
                      <span className="ml-auto text-xs text-muted-foreground tabular-nums">
                        {d.entry_count.toLocaleString('en-IN')} entries
                      </span>
                    </CardHeader>
                    <CardContent className="flex flex-col gap-1 pt-0 text-sm">
                      {d.description ? (
                        <p className="text-muted-foreground">{d.description}</p>
                      ) : null}
                      {d.attribution ? (
                        <p className="text-xs text-muted-foreground">Source: {d.attribution}</p>
                      ) : null}
                    </CardContent>
                  </Card>
                ))}
              </div>
            );
          })
        )}
      </section>

      <section aria-label="Credits" className="flex flex-col gap-2">
        <h2 className="text-lg font-semibold">Credits</h2>
        <p className="text-sm text-muted-foreground">
          Dictionary data comes from the Arsha Drishti StarDict bundle and the sanskrit-coders
          stardict-sanskrit collection, the Cologne Digital Sanskrit Dictionaries (CDSL), Sanskrit
          Heritage (Gérard Huet) and JNU sources. See each dictionary’s attribution above for its
          origin.
        </p>
      </section>
    </div>
  );
}
