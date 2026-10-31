import { useMemo } from 'react';
import { SCRIPT_LIST, type ScriptListType } from 'lipilekhika';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import { useLipiTyping } from '@/components/lipi/typing';
import { useDisplayScript, type DisplayScript } from './display-script';

/** Display-script picker for the header: transliterates headwords client-side. */
export function DisplayScriptSelect() {
  const { script, setScript } = useDisplayScript();
  const items = useMemo(() => [...SCRIPT_LIST], []);
  return (
    <Select value={script} onValueChange={(v) => setScript(v as DisplayScript)}>
      <SelectTrigger
        size="sm"
        aria-label="Display script"
        title="Display script for headwords"
        className="w-auto max-w-36 gap-1.5 rounded-full"
      >
        <SelectValue />
      </SelectTrigger>
      <SelectContent align="end" className="max-h-80">
        {items.map((s) => (
          <SelectItem key={s} value={s}>
            {s}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

/** Lipi typing script picker used inside the search settings popover. */
export function LipiScriptSelect() {
  const { script, setScript } = useLipiTyping();
  const items = useMemo(() => [...SCRIPT_LIST], []);
  return (
    <div className="flex items-center justify-between gap-3">
      <Label htmlFor="lipi-script" className="text-sm">
        Typing script
      </Label>
      <Select value={script} onValueChange={(v) => setScript(v as ScriptListType)}>
        <SelectTrigger id="lipi-script" size="sm" className="w-40" aria-label="Typing script">
          <SelectValue />
        </SelectTrigger>
        <SelectContent align="end" className="max-h-80">
          {items.map((s) => (
            <SelectItem key={s} value={s}>
              {s}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
