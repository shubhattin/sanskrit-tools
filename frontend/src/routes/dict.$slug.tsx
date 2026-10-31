import { useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { searchInt, searchString } from '@/lib/search-params';
import { useQuery } from '@tanstack/react-query';
import { ArrowLeftIcon, ArrowRightIcon, SearchIcon } from 'lucide-react';
import { browseQuery } from '@/lib/api/client';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group';
import { Skeleton } from '@/components/ui/skeleton';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty';
import { useLipiInput } from '@/components/lipi/typing';
import { LipiToggle } from '@/components/lipi/toggle';
import { useDisplayTexts } from '@/components/script/display-script';

export const Route = createFileRoute('/dict/$slug')({
  validateSearch: (search: Record<string, unknown>): { seq?: number; q?: string } => ({
    seq: searchInt(search.seq),
    q: searchString(search.q)
  }),
  component: DictBrowsePage
});

const PAGE = 30;

function JumpForm({
  initial,
  onJump
}: {
  initial: string | undefined;
  onJump: (q: string) => void;
}) {
  const [jump, setJump] = useState(initial ?? '');
  // Sync the jump input when the URL q changes. Render-time adjustment.
  const [synced, setSynced] = useState(initial);
  if (initial !== synced) {
    setSynced(initial);
    setJump(initial ?? '');
  }
  const lipiProps = useLipiInput(jump, setJump);
  return (
    <form
      role="search"
      aria-label="Jump to headword"
      className="max-w-md"
      onSubmit={(e) => {
        e.preventDefault();
        onJump(jump.trim());
      }}
    >
      <InputGroup className="h-10 rounded-xl">
        <InputGroupAddon>
          <SearchIcon aria-hidden />
        </InputGroupAddon>
        <InputGroupInput
          {...lipiProps}
          placeholder="Jump to headword…"
          aria-label="Jump to headword"
          autoComplete="off"
          spellCheck={false}
          className="font-devanagari"
        />
        <InputGroupAddon align="inline-end">
          <LipiToggle />
        </InputGroupAddon>
      </InputGroup>
    </form>
  );
}

function Pager({ center, onGo }: { center: number; onGo: (seq: number) => void }) {
  return (
    <div className="flex items-center justify-between gap-2">
      <Button
        variant="outline"
        size="sm"
        disabled={center <= 0}
        onClick={() => onGo(center - PAGE)}
      >
        <ArrowLeftIcon className="size-4" aria-hidden /> Earlier
      </Button>
      <span className="text-xs text-muted-foreground tabular-nums">seq {center}</span>
      <Button variant="outline" size="sm" onClick={() => onGo(center + PAGE)}>
        Later <ArrowRightIcon className="size-4" aria-hidden />
      </Button>
    </div>
  );
}

function DictBrowsePage() {
  const { slug } = Route.useParams();
  const search = Route.useSearch();
  const navigate = useNavigate();

  const center = Number.isFinite(search.seq) ? (search.seq as number) : 0;

  const browse = useQuery(
    browseQuery(
      search.q
        ? { slug, q: search.q, before: 5, after: PAGE }
        : { slug, seq: Math.max(0, center), before: 15, after: PAGE }
    )
  );

  const goSeq = (seq: number) =>
    void navigate({
      to: '/dict/$slug',
      params: { slug },
      search: { seq: Math.max(0, seq), q: undefined }
    });

  const items = browse.data?.items ?? [];
  const dictionary = browse.data?.dictionary;
  const headwords = useDisplayTexts(items.map((i) => i.headword));

  return (
    <div className="flex flex-col gap-4">
      <nav aria-label="Breadcrumb" className="text-sm text-muted-foreground">
        <Link to="/" search={{}} className="hover:text-foreground hover:underline">
          Search
        </Link>
        <span aria-hidden> / </span>
        <span>{dictionary?.name ?? slug}</span>
      </nav>

      <div className="flex flex-wrap items-center gap-2">
        {dictionary ? (
          <Badge variant="secondary" className="font-bold">
            {dictionary.short_name}
          </Badge>
        ) : null}
        <h1 className="text-xl font-bold sm:text-2xl">
          {dictionary?.name ?? <Skeleton className="inline-block h-6 w-48" />}
        </h1>
        {dictionary ? (
          <span className="text-sm text-muted-foreground tabular-nums">
            {dictionary.entry_count.toLocaleString('en-IN')} entries
          </span>
        ) : null}
      </div>

      <JumpForm
        initial={search.q}
        onJump={(q) =>
          void navigate({
            to: '/dict/$slug',
            params: { slug },
            search: q ? { q, seq: undefined } : { seq: 0, q: undefined }
          })
        }
      />

      {browse.isPending ? (
        <div className="flex flex-col gap-2" aria-label="Loading entries">
          {Array.from({ length: 12 }).map((_, i) => (
            <Skeleton key={i} className="h-9" />
          ))}
        </div>
      ) : browse.isError ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>Could not load dictionary</EmptyTitle>
            <EmptyDescription>{browse.error.message}</EmptyDescription>
          </EmptyHeader>
          <Button variant="outline" size="sm" onClick={() => void browse.refetch()}>
            Retry
          </Button>
        </Empty>
      ) : (
        <>
          {search.q ? (
            <p className="text-sm text-muted-foreground" role="status">
              Closest matches to{' '}
              <span className="font-devanagari font-semibold text-foreground">{search.q}</span> in
              book order
            </p>
          ) : null}
          <Card>
            <CardHeader className="border-b py-2.5 text-xs text-muted-foreground">
              Book order · seq {items[0]?.seq ?? 0} – {items[items.length - 1]?.seq ?? 0}
            </CardHeader>
            <CardContent className="p-2">
              <ol className="flex flex-col">
                {items.map((item, i) => (
                  <li key={item.id}>
                    <Link
                      to="/entry/$id"
                      params={{ id: String(item.id) }}
                      className="font-devanagari flex items-baseline justify-between gap-3 rounded-md px-2 py-1 text-lg hover:bg-muted hover:underline"
                    >
                      <span>{headwords[i] ?? item.headword}</span>
                      <span className="text-xs text-muted-foreground tabular-nums">{item.seq}</span>
                    </Link>
                  </li>
                ))}
              </ol>
            </CardContent>
          </Card>
          {!search.q ? <Pager center={center} onGo={goSeq} /> : null}
        </>
      )}
    </div>
  );
}
