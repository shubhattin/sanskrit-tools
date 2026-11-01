import type { QueryClient } from '@tanstack/react-query';
import { HeadContent, Scripts, createRootRouteWithContext } from '@tanstack/react-router';
import { Link } from '@tanstack/react-router';
import { Button } from '@/components/ui/button';
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from '@/components/ui/empty';
import appCss from '../styles.css?url';
import '../fonts';
import {
  DEFAULT_THEME_STORAGE_KEY,
  ThemeProvider,
  createThemeInitScript,
  type Theme
} from '@/components/theme-provider';
import { LipiTypingProvider } from '@/components/lipi/typing';
import { Header } from '@/components/layout/header';
import { Toaster } from '@/components/ui/sonner';
import { cn } from '@/lib/utils';

export const Route = createRootRouteWithContext<{ queryClient: QueryClient }>()({
  head: () => ({
    meta: [
      { charSet: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      { title: 'शब्दकोशः · Sanskrit Dictionary' },
      {
        name: 'description',
        content:
          'Search Sanskrit dictionaries: lexicons, thesauri, dhātu works and sūtra commentaries.'
      }
    ],
    links: [
      { rel: 'stylesheet', href: appCss },
      { rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' }
    ]
  }),
  shellComponent: RootDocument,
  notFoundComponent: RootNotFound
});

const DEFAULT_THEME: Theme = 'system';

function RootNotFound() {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>Page not found</EmptyTitle>
        <EmptyDescription>The page you are looking for does not exist.</EmptyDescription>
      </EmptyHeader>
      <Link to="/" search={{}}>
        <Button variant="outline" size="sm">
          Back to search
        </Button>
      </Link>
    </Empty>
  );
}

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="sa" suppressHydrationWarning className={cn('font-sans')}>
      <head>
        <script
          dangerouslySetInnerHTML={{
            __html: createThemeInitScript(DEFAULT_THEME_STORAGE_KEY, DEFAULT_THEME)
          }}
        />
        <HeadContent />
      </head>
      <body className={cn('font-sans antialiased')}>
        <RootProviders>{children}</RootProviders>
        <Scripts />
      </body>
    </html>
  );
}

function RootProviders({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider defaultTheme={DEFAULT_THEME} storageKey={DEFAULT_THEME_STORAGE_KEY}>
      <LipiTypingProvider>
        <div className="relative isolate flex min-h-dvh flex-col">
          <div
            aria-hidden
            className="app-backdrop pointer-events-none absolute inset-x-0 top-0 -z-10 h-[32rem]"
          />
          <Header />
          <main className="mx-auto flex w-full max-w-7xl flex-1 flex-col px-3 py-5 sm:px-6 sm:py-8">
            {children}
          </main>
          <footer className="border-t border-border/60 py-5 text-center text-xs text-muted-foreground">
            Data: Arsha Drishti StarDict bundle · Cologne CDSL · Sanskrit Heritage · JNU
          </footer>
        </div>
        <Toaster richColors={true} />
      </LipiTypingProvider>
    </ThemeProvider>
  );
}
