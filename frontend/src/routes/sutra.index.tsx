import { Link, createFileRoute, useNavigate } from '@tanstack/react-router';
import { searchString } from '@/lib/search-params';
import { useQuery } from '@tanstack/react-query';
import { BookOpenTextIcon, SearchIcon } from 'lucide-react';
import { sutrasQuery } from '@/lib/api/client';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group';
import { Skeleton } from '@/components/ui/skeleton';
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle
} from '@/components/ui/empty';
import { useState } from 'react';
import { SutraText } from '@/components/dict/entry';

export const Route = createFileRoute('/sutra/')({
  validateSearch: (search: Record<string, unknown>): { q?: string } => ({
    q: searchString(search.q)
  }),
  component: SutraListPage
});

function SutraListPage() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const [text, setText] = useState(search.q ?? '');
  const list = useQuery(sutrasQuery(search.q || undefined));

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-bold sm:text-3xl">Aṣṭādhyāyī Sūtras</h1>
      <form
        role="search"
        className="max-w-md"
        onSubmit={(e) => {
          e.preventDefault();
          void navigate({
            to: '/sutra',
            search: { q: text.trim() || undefined }
          });
        }}
      >
        <InputGroup className="h-10 rounded-xl">
          <InputGroupAddon>
            <SearchIcon aria-hidden />
          </InputGroupAddon>
          <InputGroupInput
            value={text}
            onChange={(e) => setText(e.currentTarget.value)}
            placeholder="Search by number (1.1) or text…"
            aria-label="Search sutras"
            autoComplete="off"
            spellCheck={false}
            className="font-devanagari"
          />
        </InputGroup>
      </form>
      {list.isPending ? (
        <div className="flex flex-col gap-2" aria-label="Loading sutras">
          {Array.from({ length: 10 }).map((_, i) => (
            <Skeleton key={i} className="h-12" />
          ))}
        </div>
      ) : list.isError ? (
        <Empty>
          <EmptyHeader>
            <EmptyTitle>Could not load sūtras</EmptyTitle>
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
              <BookOpenTextIcon aria-hidden />
            </EmptyMedia>
            <EmptyTitle>No sūtras found</EmptyTitle>
          </EmptyHeader>
        </Empty>
      ) : (
        <ol className="flex flex-col gap-2">
          {list.data.map((s) => (
            <li key={s.id}>
              <Link to="/sutra/$id" params={{ id: s.id }}>
                <Card className="transition-colors hover:border-primary/50">
                  <CardContent className="flex items-baseline gap-3 p-3 sm:p-4">
                    <Badge variant="secondary" className="shrink-0 tabular-nums">
                      {s.id}
                    </Badge>
                    <SutraText text={s.text} className="text-lg" />
                  </CardContent>
                </Card>
              </Link>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}
