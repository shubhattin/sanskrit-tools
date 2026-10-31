import { defineConfig } from 'vite';
import { tanstackStart } from '@tanstack/react-start/plugin/vite';
import { nitro } from 'nitro/vite';
import viteReact from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

// Static client app: TanStack Start SPA mode emits dist/index.html + assets,
// served as static files (no server needed). Axum serves /api/* separately;
// in dev, /api is proxied to the axum origin (env AXUM_DEV_ORIGIN or 127.0.0.1:5780).
// If the installed @tanstack/react-start version requires nitro, keep it below
// (see krida/vite.config.ts for the same workaround).

const axumOrigin = process.env.AXUM_DEV_ORIGIN ?? 'http://127.0.0.1:5780';

const config = defineConfig({
  resolve: {
    tsconfigPaths: true
  },
  plugins: [
    tailwindcss(),
    tanstackStart({
      spa: { enabled: true }
    }),
    // Required by this @tanstack/react-start version (see krida/vite.config.ts).
    // SPA build output: `.output/public/` (index shell + assets, served as a
    // static client app) plus a tiny node entry in `.output/server/`.
    // NOTE: vite `server.proxy` does NOT apply in dev (Start's dev handler
    // answers before it), so /api is proxied via nitro routeRules below
    // (applies in dev and in the node-server build; same-origin deploys
    // never hit it because axum serves /api itself).
    nitro({
      // Pinned: on Vercel (VERCEL=1) nitro would auto-switch to its `vercel`
      // preset and emit `.vercel/output` instead. We deploy only the static
      // `.output/public` (see root vercel.json); `/api` is the Rust function.
      preset: 'node-server',
      inlineDynamicImports: true,
      traceDeps: ['lipilekhika*'],
      routeRules: {
        '/api/**': { proxy: `${axumOrigin}/api/**` }
      }
    }),
    viteReact({
      babel: {
        plugins: ['babel-plugin-react-compiler']
      }
    })
  ]
});

export default config;
