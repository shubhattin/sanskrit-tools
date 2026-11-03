import { Link, Outlet, createFileRoute, useLocation } from '@tanstack/react-router';
import { DerivationProvider } from '@/components/vidyut/derivation';
import { AmbudaCredit, TOOLS, VidyutMark } from '@/components/vidyut/tools';
import { cn } from '@/lib/utils';

export const Route = createFileRoute('/vidyut')({
  head: () => ({ meta: [{ title: 'विद्युत् · Sanskrit grammar tools' }] }),
  component: VidyutLayout
});

function VidyutLayout() {
  const { pathname } = useLocation();
  const onIndex = pathname === '/vidyut' || pathname === '/vidyut/';
  return (
    <DerivationProvider>
      <div className="flex flex-col gap-5">
        <div className="flex flex-wrap items-center gap-x-5 gap-y-3">
          <Link to="/vidyut" className="group flex items-center gap-2.5">
            <VidyutMark className="transition-transform group-hover:-rotate-6" />
            <span className="flex flex-col leading-tight">
              <span className="font-devanagari text-xl font-bold">विद्युत्</span>
              <span className="text-xs text-muted-foreground">Sanskrit grammar tools</span>
            </span>
          </Link>
          {onIndex ? null : (
            <nav
              aria-label="Vidyut tools"
              className="flex max-w-full gap-0.5 overflow-x-auto rounded-xl bg-muted p-0.5"
            >
              {TOOLS.map(({ to, label, icon: Icon }) => {
                const active = pathname.startsWith(to);
                return (
                  <Link
                    key={to}
                    to={to}
                    aria-current={active ? 'page' : undefined}
                    className={cn(
                      'flex shrink-0 items-center gap-1.5 rounded-[10px] px-3 py-1.5 text-sm font-medium transition-colors',
                      active
                        ? 'bg-background text-foreground shadow-sm'
                        : 'text-muted-foreground hover:text-foreground'
                    )}
                  >
                    <Icon className="size-4" aria-hidden />
                    {label}
                  </Link>
                );
              })}
            </nav>
          )}
        </div>
        <Outlet />
        {onIndex ? null : <AmbudaCredit className="mt-4 border-t pt-4" />}
      </div>
    </DerivationProvider>
  );
}
