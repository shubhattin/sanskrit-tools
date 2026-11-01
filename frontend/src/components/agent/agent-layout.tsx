import { Navigate, Outlet } from '@tanstack/react-router';
import { AgentSidebar } from '@/components/agent/agent-sidebar';
import { NoAccess } from '@/components/auth/no-access';
import { authClient, isAdmin } from '@/lib/auth-client';
import { Skeleton } from '@/components/ui/skeleton';

/**
 * Admin-gated agent shell: the root layout's `main` is a centered max-w-7xl column, so this
 * breaks out with negative margins and fills the viewport height under the header; the
 * sidebar sits beside the open chat and collapses to a sheet on small screens.
 *
 * UX gate only — the Rust API re-checks the session cookie on every request.
 */
export function AgentLayout() {
  const { data, isPending } = authClient.useSession();

  if (isPending && !data) {
    return (
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-4 px-4 py-8" aria-busy="true">
        <Skeleton className="h-6 w-56" />
        <Skeleton className="h-10 w-2/3 rounded-2xl" />
        <Skeleton className="h-64 w-full rounded-2xl" />
      </div>
    );
  }

  const user = data?.user ?? null;
  if (!user) {
    return <Navigate to="/" search={{}} />;
  }
  if (!isAdmin(user)) {
    return <NoAccess email={user.email} />;
  }

  return (
    <div className="-mx-3 -my-5 flex min-h-[calc(100dvh-3.5rem)] flex-1 flex-col sm:-mx-6 sm:-my-8 md:flex-row">
      <AgentSidebar />
      <div className="flex min-h-[calc(100dvh-3.5rem)] min-w-0 flex-1 flex-col">
        <Outlet />
      </div>
    </div>
  );
}
