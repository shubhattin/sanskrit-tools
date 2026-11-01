import { ShieldAlertIcon } from 'lucide-react';
import { Link } from '@tanstack/react-router';
import { Button } from '@/components/ui/button';

/** Shown on admin-gated pages when the signed-in account lacks the admin role. */
export function NoAccess({ email }: { email?: string | null }) {
  return (
    <div className="mx-auto flex max-w-sm flex-col items-center pt-10 text-center sm:pt-16">
      <span className="grid size-14 place-items-center rounded-2xl bg-muted text-muted-foreground">
        <ShieldAlertIcon className="size-7" aria-hidden />
      </span>
      <h1 className="mt-6 text-xl font-semibold tracking-tight">Admins only</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {email ? (
          <>
            You&apos;re signed in as <span className="font-medium text-foreground">{email}</span>,
            which doesn&apos;t have admin access.
          </>
        ) : (
          <>Your account doesn&apos;t have admin access.</>
        )}{' '}
        Ask an admin to grant it, or sign in with another account.
      </p>
      <Link to="/" search={{}} className="mt-6">
        <Button variant="outline" size="sm">
          Back to search
        </Button>
      </Link>
    </div>
  );
}
