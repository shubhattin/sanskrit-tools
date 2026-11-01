import { ChevronDownIcon, CpuIcon } from 'lucide-react';
import { AGENT_MODEL_IDS, AGENT_MODELS, type AgentModelId } from '@/lib/agent/models';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';

export function ModelPicker({
  value,
  onChange,
  disabled
}: {
  value: AgentModelId;
  onChange: (model: AgentModelId) => void;
  disabled?: boolean;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button
            variant="ghost"
            size="sm"
            type="button"
            disabled={disabled}
            aria-label={`Model: ${AGENT_MODELS[value].label}`}
            className="shrink-0 text-muted-foreground"
          />
        }
      >
        <CpuIcon />
        {AGENT_MODELS[value].label}
        <ChevronDownIcon className="opacity-60" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="w-64">
        <DropdownMenuGroup>
          <DropdownMenuLabel>Model</DropdownMenuLabel>
          <DropdownMenuRadioGroup
            value={value}
            onValueChange={(next) => {
              if (next === 'gpt-6-luna' || next === 'claude-sonnet-5.5') onChange(next);
            }}
          >
            {AGENT_MODEL_IDS.map((id) => (
              <DropdownMenuRadioItem key={id} value={id} className="flex-col items-start gap-0">
                <span className="font-medium">{AGENT_MODELS[id].label}</span>
                <span className="text-xs text-muted-foreground">{AGENT_MODELS[id].hint}</span>
              </DropdownMenuRadioItem>
            ))}
          </DropdownMenuRadioGroup>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
