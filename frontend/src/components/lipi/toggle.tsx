import { LanguagesIcon } from 'lucide-react';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useLipiTyping } from './typing';

/** Compact Lipi Lekhika toggle for the search input group. */
export function LipiToggle() {
  const { enabled, setEnabled } = useLipiTyping();
  return (
    <Label
      className="flex cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-1 hover:bg-muted"
      title="Lipi Lekhika typing (Alt+X)"
    >
      <LanguagesIcon className="size-5" aria-hidden />
      <Switch
        size="sm"
        checked={enabled}
        onCheckedChange={setEnabled}
        aria-label="Enable Lipi Lekhika typing in search"
      />
    </Label>
  );
}
