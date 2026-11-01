import { Outlet } from '@tanstack/react-router';
import { AgentSidebar } from '@/components/agent/agent-sidebar';

/**
 * Full-width agent shell: the root layout's `main` is a centered max-w-7xl column, so this
 * breaks out with negative margins and fills the viewport height under the header; the
 * sidebar sits beside the open chat and collapses to a sheet on small screens.
 */
export function AgentLayout() {
  return (
    <div className="-mx-3 -my-5 flex min-h-[calc(100dvh-3.5rem)] flex-1 flex-col sm:-mx-6 sm:-my-8 md:flex-row">
      <AgentSidebar />
      <div className="flex min-h-[calc(100dvh-3.5rem)] min-w-0 flex-1 flex-col">
        <Outlet />
      </div>
    </div>
  );
}
