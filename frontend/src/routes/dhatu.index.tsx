import { useState } from 'react';
import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { searchInt, searchString } from '@/lib/search-params';
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
    q: searchString(search.q),
    gana: (() => {
      const g = searchInt(search.gana);
      return g && g >= 1 && g <= 10 ? g : undefined;
    })()
  }),
  component: DhatuListPage
});

const GANA_NAMES = [
  'भ्वादिः',
  'अदादिः',
  'जुहोत्यादिः',
  'दिवादिः',
  'स्वादिः',
  'तुदादिः',
  'रुधादिः',
  'तनादिः',
  'क्र्यादिः',
  'चुरादिः'
] as const;

/** Base UI Select renders the label of the selected value from `items`. */
const GANA_ITEMS: Record<string, string> = {
  all: 'All gaṇas',
  ...Object.fromEntries(GANA_NAMES.map((name, i) => [String(i + 1), `${i + 1} · ${name}`]))
};

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
          items={GANA_ITEMS}
          value={search.gana ? String(search.gana) : 'all'}
          onValueChange={(v) => set({ gana: !v || v === 'all' ? 0 : Number(v) })}
        >
          <SelectTrigger className="h-10 w-48 rounded-xl" aria-label="Filter by gaṇa">
            <SelectValue placeholder="Gaṇa" />
          </SelectTrigger>
          <SelectContent>
            {Object.entries(GANA_ITEMS).map(([value, label]) => (
              <SelectItem key={value} value={value}>
                {label}
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
                  <Card className="py-0 transition-colors hover:border-primary/50">
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
