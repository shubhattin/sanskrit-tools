import { Link, useLocation, useNavigate } from '@tanstack/react-router';
import {
  BookOpenTextIcon,
  InfoIcon,
  MoonIcon,
  ScrollTextIcon,
  SearchIcon,
  SunIcon,
  MonitorIcon,
  LeafIcon
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import { useTheme } from '@/components/theme-provider';
import { DisplayScriptSelect } from '@/components/script/select';
import { cn } from '@/lib/utils';

const NAV = [
  { to: '/', label: 'Search', icon: SearchIcon, exact: true },
  { to: '/sutra', label: 'Sūtras', icon: ScrollTextIcon, exact: false },
  { to: '/dhatu', label: 'Dhātus', icon: LeafIcon, exact: false },
  { to: '/about', label: 'About', icon: InfoIcon, exact: false }
] as const;

function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const Icon = theme === 'dark' ? MoonIcon : theme === 'light' ? SunIcon : MonitorIcon;
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={<Button variant="ghost" size="icon-sm" aria-label="Toggle theme" title="Theme" />}
      >
        <Icon className="size-4" aria-hidden />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => setTheme('light')}>
          <SunIcon className="size-4" aria-hidden /> Light
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('dark')}>
          <MoonIcon className="size-4" aria-hidden /> Dark
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('system')}>
          <MonitorIcon className="size-4" aria-hidden /> System
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export function Header() {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <header className="sticky top-0 z-40 border-b bg-background/85 backdrop-blur">
      <div className="mx-auto flex h-14 max-w-6xl items-center gap-2 px-3 sm:gap-3 sm:px-4">
        <Link to="/" className="flex min-w-0 items-center gap-2" aria-label="Home">
          <BookOpenTextIcon className="size-5 shrink-0 text-primary" aria-hidden />
          <span className="truncate text-sm font-bold sm:text-base">
            <span className="font-devanagari">शब्दकोशः</span>
            <span className="text-muted-foreground"> · Sanskrit Dictionary</span>
          </span>
        </Link>
        <nav aria-label="Primary" className="ml-1 flex items-center gap-0.5 sm:ml-2 sm:gap-1">
          {NAV.map(({ to, label, icon: Icon, exact }) => {
            const active = exact
              ? location.pathname === to
              : location.pathname === to || location.pathname.startsWith(`${to}/`);
            return (
              <Button
                key={to}
                variant="ghost"
                size="sm"
                className={cn(
                  'px-2 sm:px-2.5',
                  active ? 'bg-muted text-foreground' : 'text-muted-foreground'
                )}
                onClick={() => void navigate({ to })}
                aria-current={active ? 'page' : undefined}
              >
                <Icon className="size-4 sm:hidden" aria-hidden />
                <span className="max-sm:hidden">{label}</span>
                <span className="sr-only sm:hidden">{label}</span>
              </Button>
            );
          })}
        </nav>
        <div className="ml-auto flex items-center gap-1 sm:gap-2">
          <DisplayScriptSelect />
          <ThemeToggle />
        </div>
      </div>
    </header>
  );
}
