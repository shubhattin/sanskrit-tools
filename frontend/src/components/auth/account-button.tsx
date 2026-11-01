import { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { LogOutIcon, UserIcon } from 'lucide-react';
import { toast } from 'sonner';
import { authClient, isAdmin, signInWith } from '@/lib/auth-client';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import { Skeleton } from '@/components/ui/skeleton';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle
} from '@/components/ui/alert-dialog';
import { BrandIcon } from './brand-icon';

function SignInMenuItems({
  pending,
  onSignIn
}: {
  pending: 'google' | 'github' | null;
  onSignIn: (provider: 'google' | 'github') => void;
}) {
  return (
    <>
      <DropdownMenuItem disabled={pending !== null} onClick={() => onSignIn('google')}>
        {pending === 'google' ? (
          <span
            className="size-4 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent"
            aria-hidden
          />
        ) : (
          <BrandIcon brand="google" className="size-4" />
        )}{' '}
        Continue with Google
      </DropdownMenuItem>
      <DropdownMenuItem disabled={pending !== null} onClick={() => onSignIn('github')}>
        {pending === 'github' ? (
          <span
            className="size-4 animate-spin rounded-full border-2 border-muted-foreground border-t-transparent"
            aria-hidden
          />
        ) : (
          <BrandIcon brand="github" className="size-4" />
        )}{' '}
        Continue with GitHub
      </DropdownMenuItem>
    </>
  );
}

/**
 * Top-right account button: skeleton until the session resolves, a log-in menu
 * when signed out, and the avatar (image with generic-icon fallback) with a
 * sign-out menu when signed in.
 */
export function AccountButton() {
  const queryClient = useQueryClient();
  const { data, isPending } = authClient.useSession();
  const [signInPending, setSignInPending] = useState<'google' | 'github' | null>(null);
  const [signingOut, setSigningOut] = useState(false);
  const [confirmSignOut, setConfirmSignOut] = useState(false);

  // Skeleton until the session resolves (avoids a log-in/avatar flash).
  if (isPending && !data) {
    return <Skeleton className="size-8 rounded-full" aria-label="Loading account" />;
  }

  const user = data?.user ?? null;

  async function signIn(provider: 'google' | 'github') {
    setSignInPending(provider);
    try {
      // Navigates away to the provider on success.
      await signInWith(provider);
    } catch (e) {
      setSignInPending(null);
      toast.error('Could not start sign-in', {
        description: e instanceof Error ? e.message : undefined
      });
    }
  }

  async function signOut() {
    setSigningOut(true);
    try {
      const { error } = await authClient.signOut();
      if (error) throw new Error(error.message);
      queryClient.clear();
    } catch (e) {
      toast.error('Sign-out failed', { description: e instanceof Error ? e.message : undefined });
    } finally {
      setSigningOut(false);
    }
  }

  if (!user) {
    return (
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <button
              type="button"
              className="ml-0.5 grid size-8 cursor-pointer place-items-center rounded-full bg-muted text-muted-foreground ring-1 ring-foreground/10 transition hover:text-foreground hover:ring-foreground/25"
              aria-label="Log in"
              title="Log in"
            >
              <UserIcon className="size-4" aria-hidden />
            </button>
          }
        />
        <DropdownMenuContent align="end" className="w-60">
          <DropdownMenuGroup>
            <DropdownMenuLabel>Sign in</DropdownMenuLabel>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <SignInMenuItems pending={signInPending} onSignIn={(p) => void signIn(p)} />
        </DropdownMenuContent>
      </DropdownMenu>
    );
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <button
              type="button"
              className="ml-0.5 grid size-8 cursor-pointer place-items-center overflow-hidden rounded-full ring-1 ring-foreground/10 transition hover:ring-foreground/25"
              aria-label="Account"
              title={user.email ?? user.name ?? 'Account'}
            >
              {user.image ? (
                <UserImage src={user.image} alt={user.name ?? user.email ?? 'Account'} />
              ) : (
                <span className="grid size-full place-items-center bg-muted text-muted-foreground">
                  <UserIcon className="size-4" aria-hidden />
                </span>
              )}
            </button>
          }
        />
        <DropdownMenuContent align="end" className="w-60">
          <DropdownMenuGroup>
            <DropdownMenuLabel className="flex flex-col gap-0.5 font-normal">
              {user.name ? (
                <span className="truncate text-sm font-medium text-foreground">{user.name}</span>
              ) : null}
              {user.email ? (
                <span className="truncate text-xs text-muted-foreground">{user.email}</span>
              ) : null}
              {isAdmin(user) ? (
                <span className="mt-1 text-[11px] font-medium tracking-wide text-primary uppercase">
                  Admin
                </span>
              ) : null}
            </DropdownMenuLabel>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem disabled={signingOut} onClick={() => setConfirmSignOut(true)}>
            <LogOutIcon className="size-4" aria-hidden /> Sign out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      <AlertDialog open={confirmSignOut} onOpenChange={setConfirmSignOut}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Sign out?</AlertDialogTitle>
            <AlertDialogDescription>
              You will need to sign in again to use the agent.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              disabled={signingOut}
              onClick={() => {
                setConfirmSignOut(false);
                void signOut();
              }}
            >
              Sign out
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

function UserImage({ src, alt }: { src: string; alt: string }) {
  const [broken, setBroken] = useState(false);
  if (broken) {
    return (
      <span className="grid size-full place-items-center bg-muted text-muted-foreground">
        <UserIcon className="size-4" aria-hidden />
      </span>
    );
  }
  return (
    <img
      src={src}
      alt={alt}
      referrerPolicy="no-referrer"
      className="size-full object-cover"
      onError={() => setBroken(true)}
    />
  );
}
