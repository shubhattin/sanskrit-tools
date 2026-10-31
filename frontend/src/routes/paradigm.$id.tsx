import { Link, createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeftIcon } from 'lucide-react';
import { paradigmQuery } from '@/lib/api/client';
import { searchInt } from '@/lib/search-params';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty';
import { ParadigmSourceBadge, ParadigmView } from '@/components/dict/paradigm';

export const Route = createFileRoute('/paradigm/$id')({
  validateSearch: (search: Record<string, unknown>): { cell?: number } => ({
    cell: searchInt(search.cell)
  }),
  component: ParadigmPage
});

function ParadigmPage() {
  const { id } = Route.useParams();
  const search = Route.useSearch();
  const paradigmId = Number(id);
  const valid = Number.isInteger(paradigmId) && paradigmId > 0;
  const query = useQuery({ ...paradigmQuery(paradigmId), enabled: valid });

  if (!valid) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyTitle>Invalid table</EmptyTitle>
          <EmptyDescription>“{id}” is not a valid paradigm id.</EmptyDescription>
        </EmptyHeader>
        <Link to="/" search={{}}>
          <Button variant="outline" size="sm">
            Back to search
          </Button>
        </Link>
      </Empty>
    );
  }

  if (query.isPending) {
    return (
      <div className="flex flex-col gap-3" aria-label="Loading paradigm">
        <Skeleton className="h-10 w-56" />
        <Skeleton className="h-64" />
      </div>
    );
  }

  if (query.isError) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyTitle>Table not found</EmptyTitle>
          <EmptyDescription>{query.error.message}</EmptyDescription>
        </EmptyHeader>
        <Link to="/" search={{}}>
          <Button variant="outline" size="sm">
            Back to search
          </Button>
        </Link>
      </Empty>
    );
  }

  const p = query.data;
  const highlight = search.cell !== undefined ? [search.cell] : [];

  return (
    <div className="flex flex-col gap-4">
      <nav aria-label="Breadcrumb" className="text-sm text-muted-foreground">
        <Link to="/" search={{}} className="hover:text-foreground hover:underline">
          Search
        </Link>
        <span aria-hidden> / </span>
        <span>Inflection table</span>
      </nav>
      <header className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <h1 className="font-devanagari text-3xl font-bold sm:text-4xl">{p.lemma}</h1>
        {p.root ? (
          <span className="text-sm text-muted-foreground">
            from{' '}
            <Link
              to="/"
              search={{ q: p.root }}
              className="font-devanagari text-primary underline underline-offset-2"
            >
              {p.root}
            </Link>
          </span>
        ) : null}
        {p.label ? (
          <span className="font-devanagari text-lg text-muted-foreground">{p.label}</span>
        ) : null}
        <ParadigmSourceBadge source={p.source} />
      </header>
      <Card>
        <CardContent className="p-3 sm:p-4">
          <ParadigmView paradigm={p} highlight={highlight} />
        </CardContent>
      </Card>
      <div>
        <Link to="/" search={{}}>
          <Button variant="outline" size="sm">
            <ArrowLeftIcon className="size-4" aria-hidden />
            Back to search
          </Button>
        </Link>
      </div>
    </div>
  );
}
