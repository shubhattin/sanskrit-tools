import { Loader2Icon } from 'lucide-react';
import { cn } from '@/lib/utils';

function Spinner({ className, label = 'Loading', ...props }: React.ComponentProps<'svg'> & { label?: string }) {
  return (
    <Loader2Icon
      role="status"
      aria-label={label}
      className={cn('size-4 animate-spin text-muted-foreground', className)}
      {...props}
    />
  );
}

export { Spinner };
