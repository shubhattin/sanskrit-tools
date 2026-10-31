import { useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { LeafIcon, SearchIcon } from 'lucide-react';
import { dhatusQuery } from '@/lib/api/client';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import { DhatuHeadword } from '@/components/dict/entry';

export const Route = createFileRoute('/dhatu/')({
  validateSearch: (search: Record<string, unknown>): { q?: string; gana?: number } => ({
    q: typeof search.q === 'string' ? search.q : undefined,
    gana:
      typeof search.gana === 'string' && /^[0-9]+$/.test(search.gana)
        ? Number(search.gana)
        : undefined
  }),
  component: DhatuListPage
});

const GANAS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10] as const;

function DhatuListPage() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const [text, setText] = useState(search.q ?? '');
  const list = useQuery(dhatusQuery({ q: search.q || undefined, gana: search.gana }));

  const set = (patch: { q?: string; gana?: number }) =>
    void navigate({
      to: '/dhatu',
      search: {
        q: patch.q !== undefined ? patch.q || undefined : search.q,
        gana: patch.gana !== undefined ? patch.gana || undefined : search.gana
      }
    });

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-bold sm:text-3xl">Dhātupāṭha</h1>
      <div className="flex max-w-xl flex-col gap-2 sm:flex-row">
        <form
          role="search"
          className="min-w-0 flex-1"
          onSubmit={(e) => {
            e.preventDefault();
            set({ q: text.trim() });
          }}
        >
          <InputGroup className="h-10 rounded-xl">
            <InputGroupAddon>
              <SearchIcon aria-hidden />
            </InputGroupAddon>
            <InputGroupInput
              value={text}
              onChange={(e) => setText(e.currentTarget.value)}
              placeholder="Search dhātus…"
              aria-label="Search dhātus"
              autoComplete="off"
              spellCheck={false}
              className="font-devanagari"
            />
          </InputGroup>
        </form>
        <Select
          value={search.gana ? String(search.gana) : 'all'}
          onValueChange={(v) => set({ gana: v === 'all' ? 0 : Number(v) })}
        >
          <SelectTrigger className="w-36" aria-label="Filter by gaṇa">
            <SelectValue placeholder="Gaṇa" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All gaṇas</SelectItem>
            {GANAS.map((g) => (
              <SelectItem key={g} value={String(g)}>
                Gaṇa {g}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {list.isPending ? (
        <div className="flex flex-col gap-2" aria-label="Loading dhātus">
          {Array.from({ length: 10 }).map((_, i) => (
            <Skeleton key={i} className="h-12" />
          ))}
        </div>
      ) : list.isError ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>Could not load dhātus</EmptyTitle>
            <EmptyDescription>{list.error.message}</EmptyDescription>
          </EmptyHeader>
          <Button variant="outline" size="sm" onClick={() => void list.refetch()}>
            Retry
          </Button>
        </Empty>
      ) : list.data.length === 0 ? (
        <Empty>
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <LeafIcon aria-hidden />
            </EmptyMedia>
            <EmptyTitle>No dhātus found</EmptyTitle>
          </EmptyHeader>
        </Empty>
      ) : (
        <>
          <p className="text-sm text-muted-foreground tabular-nums" role="status">
            {list.data.length} dhātus
          </p>
          <ol className="flex flex-col gap-2">
            {list.data.map((d) => (
              <li key={d.id}>
                <Link to="/dhatu/$id" params={{ id: String(d.id) }}>
                  <Card className="transition-colors hover:border-primary/50">
                    <CardContent className="flex flex-wrap items-baseline gap-x-3 gap-y-1 p-3 sm:p-4">
                      <Badge variant="secondary" className="shrink-0 tabular-nums">
                        {d.number}
                      </Badge>
                      <DhatuHeadword text={d.upadesha} className="text-lg font-semibold" />
                      <span className="text-sm text-muted-foreground">
                        <DhatuHeadword text={d.root} /> · <DhatuHeadword text={d.artha} />
                      </span>
                      <span className="ml-auto text-xs text-muted-foreground">
                        gaṇa {d.gana}
                        {d.pada ? ` · ${d.pada}` : ''}
                      </span>
                    </CardContent>
                  </Card>
                </Link>
              </li>
            ))}
          </ol>
        </>
      )}
    </div>
  );
}
