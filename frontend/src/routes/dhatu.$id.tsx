import { Link, createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { dhatuQuery } from '@/lib/api/client';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty';
import { EntryHeadword, EntrySenses, DhatuHeadword } from '@/components/dict/entry';
import { InflectionSection } from '@/components/dict/paradigm';

export const Route = createFileRoute('/dhatu/$id')({
  component: DhatuDetailPage
});

function DhatuDetailPage() {
  const { id } = Route.useParams();
  const dhatuId = Number(id);
  const valid = Number.isInteger(dhatuId) && dhatuId > 0;
  const detail = useQuery({ ...dhatuQuery(dhatuId), enabled: valid });

  if (!valid) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyTitle>Invalid dhātu</EmptyTitle>
          <EmptyDescription>“{id}” is not a valid dhātu id.</EmptyDescription>
        </EmptyHeader>
        <Link to="/dhatu" search={{}}>
          <Button variant="outline" size="sm">
            All dhātus
          </Button>
        </Link>
      </Empty>
    );
  }

  if (detail.isPending) {
    return (
      <div className="flex flex-col gap-3" aria-label="Loading dhātu">
        <Skeleton className="h-10 w-48" />
        <Skeleton className="h-32" />
      </div>
    );
  }

  if (detail.isError) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyTitle>Dhātu not found</EmptyTitle>
          <EmptyDescription>{detail.error.message}</EmptyDescription>
        </EmptyHeader>
        <Link to="/dhatu" search={{}}>
          <Button variant="outline" size="sm">
            All dhātus
          </Button>
        </Link>
      </Empty>
    );
  }

  const { dhatu, works } = detail.data;
  return (
    <div className="flex flex-col gap-4">
      <nav aria-label="Breadcrumb" className="text-sm text-muted-foreground">
        <Link to="/dhatu" search={{}} className="hover:text-foreground hover:underline">
          Dhātus
        </Link>
        <span aria-hidden> / </span>
        <span>{dhatu.number}</span>
      </nav>
      <Card>
        <CardContent className="flex flex-col gap-3 p-5 sm:p-6">
          <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
            <Badge variant="secondary" className="tabular-nums">
              {dhatu.number}
            </Badge>
            <h1 className="text-3xl font-bold">
              <DhatuHeadword text={dhatu.upadesha} />
            </h1>
          </div>
          <dl className="grid grid-cols-1 gap-x-8 gap-y-1.5 text-[15px] sm:grid-cols-2">
            <div className="flex gap-2">
              <dt className="shrink-0 font-semibold text-muted-foreground">Root</dt>
              <dd>
                <DhatuHeadword text={dhatu.root} />
              </dd>
            </div>
            <div className="flex gap-2">
              <dt className="shrink-0 font-semibold text-muted-foreground">Meaning</dt>
              <dd>
                <DhatuHeadword text={dhatu.artha} />
              </dd>
            </div>
            <div className="flex gap-2">
              <dt className="shrink-0 font-semibold text-muted-foreground">Gaṇa</dt>
              <dd className="tabular-nums">{dhatu.gana}</dd>
            </div>
            {dhatu.pada ? (
              <div className="flex gap-2">
                <dt className="shrink-0 font-semibold text-muted-foreground">Pada</dt>
                <dd>{dhatu.pada}</dd>
              </div>
            ) : null}
            {dhatu.it ? (
              <div className="flex gap-2">
                <dt className="shrink-0 font-semibold text-muted-foreground">It</dt>
                <dd className="font-devanagari">{dhatu.it}</dd>
              </div>
            ) : null}
            {dhatu.extra ? (
              <div className="flex gap-2 sm:col-span-2">
                <dt className="shrink-0 font-semibold text-muted-foreground">Note</dt>
                <dd className="font-devanagari">{dhatu.extra}</dd>
              </div>
            ) : null}
          </dl>
        </CardContent>
      </Card>
      <InflectionSection query={dhatu.root} />
      {works.length === 0 ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>No work entries indexed</EmptyTitle>
            <EmptyDescription>No dhātu-work entries reference this dhātu yet.</EmptyDescription>
          </EmptyHeader>
        </Empty>
      ) : (
        <Tabs defaultValue={works[0]!.dictionary.slug} className="w-full">
          <TabsList className="h-auto flex-wrap">
            {works.map((w) => (
              <TabsTrigger key={w.dictionary.slug} value={w.dictionary.slug}>
                {w.dictionary.short_name}
              </TabsTrigger>
            ))}
          </TabsList>
          {works.map((w) => (
            <TabsContent key={w.dictionary.slug} value={w.dictionary.slug}>
              <Card>
                <CardHeader className="border-b py-3">
                  <p className="text-sm font-semibold">{w.dictionary.name}</p>
                </CardHeader>
                <CardContent className="flex flex-col gap-5 p-4 sm:p-5">
                  {w.entries.map((entry) => (
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
