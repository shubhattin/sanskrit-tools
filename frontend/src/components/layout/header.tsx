import { useState } from 'react';
import { Link, useLocation, useRouterState } from '@tanstack/react-router';
import { useIsFetching } from '@tanstack/react-query';
import {
  LeafIcon,
  MenuIcon,
  MonitorIcon,
  MoonIcon,
  ScrollTextIcon,
  SearchIcon,
  SunIcon
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle
} from '@/components/ui/sheet';
import { useTheme, type Theme } from '@/components/theme-provider';
import { cn } from '@/lib/utils';

const NAV = [
  {
    to: '/',
    label: 'Search',
    hint: 'Lexicons, thesauri & commentaries',
    icon: SearchIcon,
    exact: true
  },
  {
    to: '/sutra',
    label: 'Sūtras',
    hint: 'Aṣṭādhyāyī with commentaries',
    icon: ScrollTextIcon,
    exact: false
  },
  { to: '/dhatu', label: 'Dhātus', hint: 'Verbal roots by gaṇa', icon: LeafIcon, exact: false }
] as const;

const THEMES: { v: Theme; label: string; icon: typeof SunIcon }[] = [
  { v: 'light', label: 'Light', icon: SunIcon },
  { v: 'dark', label: 'Dark', icon: MoonIcon },
  { v: 'system', label: 'System', icon: MonitorIcon }
];

function useIsActive() {
  const { pathname } = useLocation();
  return (to: string, exact: boolean) =>
    exact ? pathname === to : pathname === to || pathname.startsWith(`${to}/`);
}

export function LogoMark({ className }: { className?: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        'grid size-8 shrink-0 place-items-center rounded-xl bg-gradient-to-br from-primary via-teal-500 to-sky-500 font-devanagari text-lg leading-none font-bold text-white shadow-md shadow-primary/30',
        className
      )}
    >
      श
    </span>
  );
}

function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const Icon = theme === 'dark' ? MoonIcon : theme === 'light' ? SunIcon : MonitorIcon;
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button variant="ghost" size="icon-sm" aria-label="Theme" title="Theme" />}
      >
        <Icon className="size-4" aria-hidden />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {THEMES.map(({ v, label, icon: I }) => (
          <DropdownMenuItem key={v} onClick={() => setTheme(v)}>
            <I className="size-4" aria-hidden /> {label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function MobileNav() {
  const [open, setOpen] = useState(false);
  const isActive = useIsActive();
  const { theme, setTheme } = useTheme();
  return (
    <>
      <Button
        variant="ghost"
        size="icon-sm"
        className="md:hidden"
        aria-label="Open menu"
        onClick={() => setOpen(true)}
      >
        <MenuIcon className="size-5" aria-hidden />
      </Button>
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetContent side="left" className="w-[82%] max-w-xs gap-0 p-0">
          <SheetHeader className="flex-row items-center gap-3 border-b bg-gradient-to-br from-primary/10 via-transparent to-sky-500/10 p-4">
            <LogoMark className="size-10 text-xl" />
            <div>
              <SheetTitle className="font-devanagari text-lg font-bold">शब्दकोशः</SheetTitle>
              <SheetDescription className="text-xs">Sanskrit dictionary portal</SheetDescription>
            </div>
          </SheetHeader>
          <nav aria-label="Primary" className="flex flex-col gap-1 p-3">
            {NAV.map(({ to, label, hint, icon: Icon, exact }) => {
              const active = isActive(to, exact);
              return (
                <Link
                  key={to}
                  to={to}
                  onClick={() => setOpen(false)}
                  aria-current={active ? 'page' : undefined}
                  className={cn(
                    'flex items-center gap-3 rounded-xl px-3 py-2.5 transition-colors',
                    active
                      ? 'bg-primary/10 text-foreground'
                      : 'text-muted-foreground hover:bg-muted'
                  )}
                >
                  <span
                    className={cn(
                      'grid size-9 place-items-center rounded-lg',
                      active ? 'bg-primary text-primary-foreground' : 'bg-muted'
                    )}
                  >
                    <Icon className="size-4" aria-hidden />
                  </span>
                  <span className="flex flex-col">
                    <span className="text-sm font-semibold text-foreground">{label}</span>
                    <span className="text-xs">{hint}</span>
                  </span>
                </Link>
              );
            })}
          </nav>
          <div className="mt-auto border-t p-4">
            <p className="mb-2 text-[11px] font-semibold tracking-wider text-muted-foreground uppercase">
              Theme
            </p>
            <div className="grid grid-cols-3 gap-1 rounded-xl bg-muted p-1">
              {THEMES.map(({ v, label, icon: I }) => (
                <button
                  key={v}
                  type="button"
                  onClick={() => setTheme(v)}
                  aria-pressed={theme === v}
                  className={cn(
                    'flex items-center justify-center gap-1.5 rounded-lg py-1.5 text-xs font-medium transition-colors',
                    theme === v
                      ? 'bg-background text-foreground shadow-sm'
                      : 'text-muted-foreground hover:text-foreground'
                  )}
                >
                  <I className="size-3.5" aria-hidden /> {label}
                </button>
              ))}
            </div>
            <p className="mt-4 text-[11px] leading-relaxed text-muted-foreground">
              Data: Arsha Drishti StarDict bundle · Cologne CDSL · Sanskrit Heritage · JNU
            </p>
          </div>
        </SheetContent>
      </Sheet>
    </>
  );
}

/** Thin bar under the header while a route chunk or any query is loading. */
function LoadingBar() {
  const routeLoading = useRouterState({ select: (s) => s.status === 'pending' });
  const fetching = useIsFetching() > 0;
  const on = routeLoading || fetching;
  return (
    <div
      aria-hidden
      className={cn(
        'pointer-events-none absolute inset-x-0 -bottom-px h-0.5 overflow-hidden transition-opacity duration-300',
        on ? 'opacity-100 delay-150' : 'opacity-0'
      )}
    >
      {on ? (
        <div className="progress-indeterminate h-full w-1/2 rounded-full bg-gradient-to-r from-primary via-teal-400 to-sky-400" />
      ) : null}
    </div>
  );
}

export function Header() {
  const isActive = useIsActive();
  return (
    <header className="sticky top-0 z-40 border-b border-border/60 bg-background/75 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-14 max-w-7xl items-center gap-2 px-3 sm:px-6">
        <MobileNav />
        <Link
          to="/"
          search={{}}
          className="group flex min-w-0 items-center gap-2.5"
          aria-label="Home"
        >
          <LogoMark className="transition-transform duration-300 group-hover:scale-105 group-hover:-rotate-6" />
          <span className="flex min-w-0 flex-col leading-tight">
            <span className="truncate font-devanagari text-base font-bold">शब्दकोशः</span>
            <span className="truncate text-[11px] text-muted-foreground max-sm:hidden">
              Sanskrit Dictionary
            </span>
          </span>
        </Link>
        <nav aria-label="Primary" className="ml-4 hidden items-center gap-1 md:flex">
          {NAV.map(({ to, label, icon: Icon, exact }) => {
            const active = isActive(to, exact);
            return (
              <Link
                key={to}
                to={to}
                aria-current={active ? 'page' : undefined}
                className={cn(
                  'relative flex h-8 items-center gap-1.5 rounded-full px-3 text-sm font-medium transition-colors',
                  active
                    ? 'bg-primary/10 text-primary'
                    : 'text-muted-foreground hover:bg-muted hover:text-foreground'
                )}
              >
                <Icon className="size-4" aria-hidden />
                {label}
              </Link>
            );
          })}
        </nav>
        <div className="ml-auto flex items-center gap-1">
          <ThemeToggle />
        </div>
      </div>
      <LoadingBar />
    </header>
  );
}
