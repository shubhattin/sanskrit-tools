import { Link, createFileRoute } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeftIcon, ArrowRightIcon } from 'lucide-react';
import { browseQuery, dictionariesQuery, entryQuery } from '@/lib/api/client';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import { EntryHeadword, EntrySenses } from '@/components/dict/entry';
import { InflectionSection } from '@/components/dict/paradigm';

export const Route = createFileRoute('/entry/$id')({
  component: EntryPage
});

function EntryPage() {
  const { id } = Route.useParams();
  const entryId = Number(id);
  const valid = Number.isInteger(entryId) && entryId > 0;
  const query = useQuery({ ...entryQuery(entryId), enabled: valid });

  if (!valid) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyTitle>Invalid entry</EmptyTitle>
          <EmptyDescription>“{id}” is not a valid entry id.</EmptyDescription>
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
      <div className="flex flex-col gap-3" aria-label="Loading entry">
        <Skeleton className="h-8 w-64" />
        <Skeleton className="h-64" />
      </div>
    );
  }

  if (query.isError) {
    return (
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <ArrowLeftIcon aria-hidden />
          </EmptyMedia>
          <EmptyTitle>Entry not found</EmptyTitle>
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

  const { entry, dictionary } = query.data;
  return (
    <div className="flex flex-col gap-4">
      <nav aria-label="Breadcrumb" className="text-sm text-muted-foreground">
        <Link to="/" search={{}} className="hover:text-foreground hover:underline">
          Search
        </Link>
        <span aria-hidden> / </span>
        <Link
          to="/dict/$slug"
          params={{ slug: dictionary.slug }}
          search={{}}
          className="hover:text-foreground hover:underline"
        >
          {dictionary.short_name}
        </Link>
      </nav>
      <Card>
        <CardHeader className="flex flex-row items-center gap-2 border-b">
          <Badge variant="secondary" className="font-bold">
            {dictionary.short_name}
          </Badge>
          <span className="text-sm text-muted-foreground">{dictionary.name}</span>
        </CardHeader>
        <CardContent className="flex flex-col gap-3 p-4 sm:p-5">
          <h1 className="text-3xl font-bold sm:text-4xl">
            <EntryHeadword headword={entry.headword} />
          </h1>
          <EntrySenses senses={entry.senses} />
          {entry.data ? (
            <dl className="grid grid-cols-1 gap-x-6 gap-y-1 rounded-lg bg-muted/60 p-3 text-sm sm:grid-cols-2">
              {Object.entries(entry.data).map(([k, v]) => (
                <div key={k} className="flex gap-2">
                  <dt className="shrink-0 font-semibold text-muted-foreground">{k}</dt>
                  <dd className="min-w-0 break-words">{String(v)}</dd>
                </div>
              ))}
            </dl>
          ) : null}
          <div className="flex flex-wrap gap-2 text-sm">
            {entry.sutra_id ? (
              <Link
                to="/sutra/$id"
                params={{ id: entry.sutra_id }}
                className="text-primary underline underline-offset-2"
              >
                Sūtra {entry.sutra_id}
              </Link>
            ) : null}
            {entry.dhatu_id ? (
              <Link
                to="/dhatu/$id"
                params={{ id: String(entry.dhatu_id) }}
                className="text-primary underline underline-offset-2"
              >
                Dhātu #{entry.dhatu_id}
              </Link>
            ) : null}
          </div>
        </CardContent>
      </Card>
      <InflectionSection query={entry.headword} />
      <BrowseNearby slug={dictionary.slug} seq={entry.seq} currentId={entry.id} />
    </div>
  );
}

function BrowseNearby({ slug, seq, currentId }: { slug: string; seq: number; currentId: number }) {
  const dicts = useQuery(dictionariesQuery());
  const dictionary = dicts.data?.find((d) => d.slug === slug);
  const browse = useQuery({
    ...browseQuery({ slug, seq, before: 3, after: 3 }),
    enabled: true
  });
  const items = browse.data?.items ?? [];
  const idx = items.findIndex((i) => i.id === currentId);
  const prev = idx > 0 ? items[idx - 1] : null;
  const next = idx >= 0 && idx < items.length - 1 ? items[idx + 1] : null;

  const maxSeq = Math.max(0, (dictionary?.entry_count ?? 1) - 1);
  const prevSeq = seq > 0 ? seq - 1 : null;
  const nextSeq = seq < maxSeq ? seq + 1 : null;

  return (
    <Card>
      <CardContent className="flex flex-col gap-3 p-4">
        <h2 className="text-sm font-semibold tracking-wide text-muted-foreground uppercase">
          Browse nearby in {dictionary?.short_name ?? slug}
        </h2>
        {browse.isPending ? (
          <Skeleton className="h-10" />
        ) : (
          <ul className="flex flex-col gap-1">
            {items.map((item) => (
              <li key={item.id}>
                <Link
                  to="/entry/$id"
                  params={{ id: String(item.id) }}
                  className={
                    item.id === currentId
                      ? 'font-devanagari block rounded-md bg-muted px-2 py-1 font-semibold'
                      : 'font-devanagari block rounded-md px-2 py-1 hover:bg-muted hover:underline'
                  }
                  aria-current={item.id === currentId ? 'page' : undefined}
                >
                  {item.headword}
                </Link>
              </li>
            ))}
          </ul>
        )}
        <div className="flex items-center justify-between gap-2">
          {prev ? (
            <Link to="/entry/$id" params={{ id: String(prev.id) }}>
              <Button variant="outline" size="sm">
                <ArrowLeftIcon className="size-4" aria-hidden />
                <span className="font-devanagari max-w-32 truncate">{prev.headword}</span>
              </Button>
            </Link>
          ) : prevSeq !== null ? (
            <BrowseSeqButton slug={slug} seq={prevSeq} label="Previous" icon="prev" />
          ) : (
            <span />
          )}
          {next ? (
            <Link to="/entry/$id" params={{ id: String(next.id) }}>
              <Button variant="outline" size="sm">
                <span className="font-devanagari max-w-32 truncate">{next.headword}</span>
                <ArrowRightIcon className="size-4" aria-hidden />
              </Button>
            </Link>
          ) : nextSeq !== null ? (
            <BrowseSeqButton slug={slug} seq={nextSeq} label="Next" icon="next" />
          ) : (
            <span />
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function BrowseSeqButton({
  slug,
  seq,
  label,
  icon
}: {
  slug: string;
  seq: number;
  label: string;
  icon: 'prev' | 'next';
}) {
  const browse = useQuery(browseQuery({ slug, seq, before: 0, after: 1 }));
  const item = browse.data?.items[0];
  if (!item) return <span />;
  const Icon = icon === 'prev' ? ArrowLeftIcon : ArrowRightIcon;
  return (
    <Link to="/entry/$id" params={{ id: String(item.id) }}>
      <Button variant="outline" size="sm">
        {icon === 'prev' ? <Icon className="size-4" aria-hidden /> : null}
        <span className="font-devanagari max-w-32 truncate">{item.headword || label}</span>
        {icon === 'next' ? <Icon className="size-4" aria-hidden /> : null}
      </Button>
    </Link>
  );
}
