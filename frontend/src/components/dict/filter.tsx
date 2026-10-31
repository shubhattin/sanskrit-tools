import { useState } from 'react';
import { Settings2Icon } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Separator } from '@/components/ui/separator';
import { Switch } from '@/components/ui/switch';
import { cn } from '@/lib/utils';
import type { Dictionary } from '@/lib/api/types';
import { DICT_KIND_LABELS, DICT_KIND_ORDER } from '@/lib/api/types';
import { useLipiTyping } from '@/components/lipi/typing';
import { LipiScriptSelect } from '@/components/script/select';

function groupByKind(dicts: Dictionary[]): Map<string, Dictionary[]> {
  const map = new Map<string, Dictionary[]>();
  for (const d of [...dicts].sort((a, b) => a.sort_order - b.sort_order)) {
    const list = map.get(d.kind) ?? [];
    list.push(d);
    map.set(d.kind, list);
  }
  return map;
}

export function SearchSettingsPopover({
  dictionaries,
  selected,
  onChange
}: {
  dictionaries: Dictionary[];
  /** Selected slugs; empty means "all". */
  selected: string[];
  onChange: (next: string[]) => void;
}) {
  const [open, setOpen] = useState(false);
  const { enabled, setEnabled } = useLipiTyping();
  const byKind = groupByKind(dictionaries);

  const toggleSlug = (slug: string) => {
    if (selected.includes(slug)) onChange(selected.filter((s) => s !== slug));
    else onChange([...selected, slug]);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={
          <Button
            variant="ghost"
            size="icon-sm"
            type="button"
            title="Search settings"
            aria-label="Search settings"
          />
        }
      >
        <Settings2Icon className="size-4" aria-hidden />
      </PopoverTrigger>
      <PopoverContent align="end" className="w-80">
        <div className="flex items-center justify-between gap-3">
          <Label htmlFor="lipi-toggle-settings" className="text-sm">
            Lipi Lekhika typing <span className="text-muted-foreground">(Alt+X)</span>
          </Label>
          <Switch
            id="lipi-toggle-settings"
            size="sm"
            checked={enabled}
            onCheckedChange={setEnabled}
            aria-label="Enable Lipi Lekhika typing"
          />
        </div>
        <LipiScriptSelect />
        <Separator />
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium">Dictionaries</span>
          <Button
            variant="ghost"
            size="sm"
            className="h-6 px-2 text-xs"
            onClick={() => onChange([])}
          >
            All
          </Button>
        </div>
        <div className="flex max-h-72 flex-col gap-3 overflow-y-auto">
          {DICT_KIND_ORDER.filter((k) => byKind.has(k)).map((kind) => (
            <div key={kind} className="flex flex-col gap-1">
              <p className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
                {DICT_KIND_LABELS[kind]}
              </p>
              {byKind.get(kind)!.map((d) => {
                const checked = selected.length === 0 || selected.includes(d.slug);
                return (
                  <Label
                    key={d.slug}
                    className={cn(
                      'flex cursor-pointer items-center gap-2 rounded-md px-1.5 py-1 text-sm hover:bg-muted',
                      selected.length > 0 && !checked && 'opacity-60'
                    )}
                  >
                    <Checkbox
                      checked={checked}
                      onCheckedChange={() => toggleSlug(d.slug)}
                      aria-label={d.name}
                    />
                    <span className="min-w-0 flex-1 truncate">
                      <span className="mr-1.5 font-bold text-muted-foreground">{d.short_name}</span>
                      {d.name}
                    </span>
                    <span className="text-xs text-muted-foreground tabular-nums">
                      {d.entry_count}
                    </span>
                  </Label>
                );
              })}
            </div>
          ))}
        </div>
      </PopoverContent>
    </Popover>
  );
}
