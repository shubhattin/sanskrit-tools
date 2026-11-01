import { createAuthClient } from 'better-auth/react';
import { adminClient } from 'better-auth/client/plugins';

/**
 * Better Auth client talking directly to the tsc-users auth server. Session
 * cookies are shared across subdomains, so the Rust API sees the same session
 * (it forwards the request cookie to `${baseURL}/api/auth/get-session`).
 */
export const authClient = createAuthClient({
  baseURL: import.meta.env.VITE_BETTER_AUTH_URL,
  plugins: [adminClient()]
});

export type Session = typeof authClient.$Infer.Session;

/** Better Auth's admin plugin stores roles as a comma-separated string. */
export function isAdmin(user: { role?: string | null } | null | undefined): boolean {
  return (user?.role ?? '').split(',').some((r) => r.trim() === 'admin');
}

export async function signInWith(provider: 'google' | 'github') {
  const { error } = await authClient.signIn.social({
    provider,
    // Absolute: the auth server lives on another origin.
    callbackURL: window.location.href
  });
  if (error) throw new Error(error.message ?? 'Sign-in failed');
}
