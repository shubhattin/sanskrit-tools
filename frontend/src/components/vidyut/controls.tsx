import type { ReactNode } from 'react';
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group';
import { Textarea } from '@/components/ui/textarea';
import { Skeleton } from '@/components/ui/skeleton';
import { useLipiInput } from '@/components/lipi/typing';
import { cn } from '@/lib/utils';

/**
 * Single-line Devanagari field. Lipi Lekhika typing is always on (romanized keys type
 * Devanagari; Devanagari keyboards work as usual), since these tools accept Devanagari only.
 */
export function DevaInput({
  value,
  onChange,
  placeholder,
  label,
  icon,
  className,
  autoFocus
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  label: string;
  icon?: ReactNode;
  className?: string;
  autoFocus?: boolean;
}) {
  const { props } = useLipiInput(value, onChange, true);
  return (
    <InputGroup className={cn('h-10 rounded-xl', className)}>
      {icon ? <InputGroupAddon>{icon}</InputGroupAddon> : null}
      <InputGroupInput
        {...props}
        placeholder={placeholder}
        aria-label={label}
        autoComplete="off"
        autoCapitalize="off"
        spellCheck={false}
        autoFocus={autoFocus}
        className="font-devanagari text-base"
      />
    </InputGroup>
  );
}

export function DevaTextarea({
  value,
  onChange,
  placeholder,
  label,
  rows = 4
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  label: string;
  rows?: number;
}) {
  const { props } = useLipiInput(value, onChange, true);
  return (
    <div className="relative">
      <Textarea
        {...props}
        rows={rows}
        placeholder={placeholder}
        aria-label={label}
        spellCheck={false}
        autoCapitalize="off"
        className="min-h-28 resize-y rounded-xl font-devanagari text-base leading-relaxed"
      />
    </div>
  );
}

/** Compact segmented control (radio group of pills). */
export function Segmented<T extends string>({
  value,
  onChange,
  options,
  label,
  className,
  size = 'md'
}: {
  value: T;
  onChange: (v: T) => void;
  options: { id: T; label: ReactNode; hint?: string; disabled?: boolean }[];
  label: string;
  className?: string;
  size?: 'sm' | 'md';
}) {
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={cn('inline-flex flex-wrap gap-0.5 rounded-xl bg-muted p-0.5', className)}
    >
      {options.map((o) => {
        const on = o.id === value;
        return (
          <button
            key={o.id}
            type="button"
            role="radio"
            aria-checked={on}
            disabled={o.disabled}
            title={o.hint}
            onClick={() => onChange(o.id)}
            className={cn(
              'rounded-[10px] font-medium transition-colors disabled:pointer-events-none disabled:opacity-40',
              size === 'sm' ? 'px-2.5 py-1 text-xs' : 'px-3 py-1.5 text-sm',
              on
                ? 'bg-background text-foreground shadow-sm'
                : 'text-muted-foreground hover:text-foreground'
            )}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

/** Toggleable chip (multi-select lists: sanādi, upasargas). */
export function Chip({
  on,
  onClick,
  children,
  title,
  order
}: {
  on: boolean;
  onClick: () => void;
  children: ReactNode;
  title?: string;
  /** Position in the chosen sequence (shown when on). */
  order?: number;
}) {
  return (
    <button
      type="button"
      aria-pressed={on}
      title={title}
      onClick={onClick}
      className={cn(
        'inline-flex items-center gap-1 rounded-full border px-2.5 py-0.5 font-devanagari text-sm transition-colors',
        on
          ? 'border-primary/50 bg-primary/10 text-foreground'
          : 'border-border text-muted-foreground hover:border-foreground/30 hover:text-foreground'
      )}
    >
      {on && order !== undefined ? (
        <span className="grid size-4 place-items-center rounded-full bg-primary text-[10px] leading-none font-semibold text-primary-foreground">
          {order}
        </span>
      ) : null}
      {children}
    </button>
  );
}

export function SectionLabel({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <p
      className={cn(
        'text-[11px] font-semibold tracking-wider text-muted-foreground uppercase',
        className
      )}
    >
      {children}
    </p>
  );
}

export function LoadingBlock({ rows = 3, label }: { rows?: number; label: string }) {
  return (
    <div className="flex flex-col gap-2" role="status" aria-label={label}>
      {Array.from({ length: rows }).map((_, i) => (
        <Skeleton key={i} className="h-12 rounded-xl" />
      ))}
    </div>
  );
}

export function ErrorNote({ error }: { error: Error }) {
  return (
    <p
      role="alert"
      className="rounded-xl border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive"
    >
      {error.message}
    </p>
  );
}
