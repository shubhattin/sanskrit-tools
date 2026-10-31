import { useNavigate } from '@tanstack/react-router';
import { useCallback } from 'react';
import { cn } from '@/lib/utils';
import type { MatchKind } from '@/lib/api/types';

const MATCH_STYLES: Record<MatchKind, string> = {
  exact: 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300',
  form: 'bg-sky-500/15 text-sky-700 dark:text-sky-300',
  prefix: 'bg-violet-500/15 text-violet-700 dark:text-violet-300',
  fuzzy: 'bg-amber-500/15 text-amber-700 dark:text-amber-300',
  fulltext: 'bg-rose-500/15 text-rose-700 dark:text-rose-300'
};

export function MatchBadge({ match }: { match: MatchKind }) {
  return (
    <span
      className={cn(
        'inline-flex shrink-0 items-center rounded-full px-2 py-0.5 text-[11px] font-semibold',
        MATCH_STYLES[match]
      )}
      title={`Match type: ${match}`}
    >
      {match}
    </span>
  );
}

/**
 * Renders a server-sanitized sense HTML fragment. Cross-references
 * (`a[data-q]`) are intercepted and routed to `/?q=<data-q>`.
 */
export function SenseHtml({ html, className }: { html: string; className?: string }) {
  const navigate = useNavigate();

  const onClick = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      const anchor = (event.target as HTMLElement).closest('a[data-q]');
      if (!anchor) return;
      const q = anchor.getAttribute('data-q');
      if (!q) return;
      event.preventDefault();
      void navigate({ to: '/', search: { q, mode: 'sa' } });
    },
    [navigate]
  );

  return (
    <div
      className={cn('sense-html text-[15px]', className)}
      dangerouslySetInnerHTML={{ __html: html }}
      onClick={onClick}
    />
  );
}
