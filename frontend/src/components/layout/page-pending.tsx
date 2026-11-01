import { Skeleton } from '@/components/ui/skeleton';

/** Generic page skeleton while a route (code chunk / loader) is pending. */
export function PagePending() {
  return (
    <div className="flex flex-col gap-4" role="status" aria-label="Loading page">
      <Skeleton className="h-9 w-64 rounded-lg" />
      <Skeleton className="h-11 w-full max-w-xl rounded-xl" />
      <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
        {Array.from({ length: 9 }).map((_, i) => (
          <Skeleton key={i} className="h-16 rounded-xl" />
        ))}
      </div>
    </div>
  );
}
