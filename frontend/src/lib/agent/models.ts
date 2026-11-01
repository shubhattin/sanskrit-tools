import type { AgentModelId } from './types';

export type { AgentModelId };

export const AGENT_MODELS: Record<AgentModelId, { label: string; hint: string }> = {
  'gpt-6-luna': { label: 'GPT-6 Luna', hint: 'Default · balanced reasoning and cost' },
  'claude-sonnet-5.5': { label: 'Claude Sonnet 5.5', hint: 'Strong Sanskrit and commentary prose' }
};

export const AGENT_MODEL_IDS = Object.keys(AGENT_MODELS) as AgentModelId[];

export const DEFAULT_AGENT_MODEL: AgentModelId = 'gpt-6-luna';

const STORAGE_KEY = 'sans-dict-agent-model';

export function isAgentModelId(value: unknown): value is AgentModelId {
  return value === 'gpt-6-luna' || value === 'claude-sonnet-5.5';
}

export function loadStoredModel(): AgentModelId {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (isAgentModelId(raw)) return raw;
  } catch {
    // Storage unavailable: fall through to the default.
  }
  return DEFAULT_AGENT_MODEL;
}

export function storeModel(model: AgentModelId): void {
  try {
    localStorage.setItem(STORAGE_KEY, model);
  } catch {
    // Private mode etc: the choice just doesn't persist.
  }
}
