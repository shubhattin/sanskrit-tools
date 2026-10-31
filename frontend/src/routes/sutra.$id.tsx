import { Link, createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeftIcon, ArrowRightIcon } from 'lucide-react';
import { sutraQuery } from '@/lib/api/client';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty';
import { EntrySenses, EntryHeadword, SutraText } from '@/components/dict/entry';

export const Route = createFileRoute('/sutra/$id')({
  component: SutraDetailPage
});

function SutraDetailPage() {
  const { id } = Route.useParams();
  const detail = useQuery(sutraQuery(id));

  if (detail.isPending) {
    return (
      <div className="flex flex-col gap-3" aria-label="Loading sutra">
        <Skeleton className="h-10 w-40" />
        <Skeleton className="h-24" />
        <Skeleton className="h-48" />
      </div>
    );
  }

  if (detail.isError) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyTitle>Sūtra not found</EmptyTitle>
          <EmptyDescription>{detail.error.message}</EmptyDescription>
        </EmptyHeader>
        <Link to="/sutra" search={{}}>
          <Button variant="outline" size="sm">
            All sūtras
          </Button>
        </Link>
      </Empty>
    );
  }

  const { sutra, prev, next, commentaries } = detail.data;
  return (
    <div className="flex flex-col gap-4">
      <nav aria-label="Breadcrumb" className="text-sm text-muted-foreground">
        <Link to="/sutra" search={{}} className="hover:text-foreground hover:underline">
          Sūtras
        </Link>
        <span aria-hidden> / </span>
        <span>{sutra.id}</span>
      </nav>
      <Card>
        <CardContent className="flex flex-col items-center gap-2 p-6 text-center sm:p-8">
          <Badge variant="secondary" className="tabular-nums">
            {sutra.id}
          </Badge>
          <p className="text-3xl font-bold sm:text-4xl">
            <SutraText text={sutra.text} />
          </p>
          <p className="text-xs text-muted-foreground">
            Adhyāya {sutra.adhyaya} · Pāda {sutra.pada} · Sūtra {sutra.num}
          </p>
        </CardContent>
      </Card>
      <div className="flex items-center justify-between gap-2">
        {prev ? (
          <Link to="/sutra/$id" params={{ id: prev }}>
            <Button variant="outline" size="sm">
              <ArrowLeftIcon className="size-4" aria-hidden /> {prev}
            </Button>
          </Link>
        ) : (
          <span />
        )}
        {next ? (
          <Link to="/sutra/$id" params={{ id: next }}>
            <Button variant="outline" size="sm">
              {next} <ArrowRightIcon className="size-4" aria-hidden />
            </Button>
          </Link>
        ) : (
          <span />
        )}
      </div>
      {commentaries.length === 0 ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>No commentaries indexed</EmptyTitle>
            <EmptyDescription>No commentary entries reference this sūtra yet.</EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <Tabs defaultValue={commentaries[0]!.dictionary.slug} className="w-full">
          <TabsList className="h-auto flex-wrap">
            {commentaries.map((c) => (
              <TabsTrigger key={c.dictionary.slug} value={c.dictionary.slug}>
                {c.dictionary.short_name}
              </TabsTrigger>
            ))}
          </TabsList>
          {commentaries.map((c) => (
            <TabsContent key={c.dictionary.slug} value={c.dictionary.slug}>
              <Card>
                <CardHeader className="border-b py-3">
                  <p className="text-sm font-semibold">{c.dictionary.name}</p>
                </CardHeader>
                <CardContent className="flex flex-col gap-5 p-4 sm:p-5">
                  {c.entries.map((entry) => (
                    <article key={entry.id} className="flex flex-col gap-1.5">
                      <Link
                        to="/entry/$id"
                        params={{ id: String(entry.id) }}
                        className="w-fit text-xl font-semibold hover:text-primary hover:underline"
                      >
                        <EntryHeadword headword={entry.headword} />
                      </Link>
                      <EntrySenses senses={entry.senses} />
                    </article>
                  ))}
                </CardContent>
              </Card>
            </TabsContent>
          ))}
        </Tabs>
      )}
    </div>
  );
}
