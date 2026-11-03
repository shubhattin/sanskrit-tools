import type { AgentUIMessage, AgentUITools } from '@/lib/agent/types';

export type AgentPart = AgentUIMessage['parts'][number];

export type AgentToolPart<N extends keyof AgentUITools> = Extract<AgentPart, { type: `tool-${N}` }>;

/** Parts shown as rows of the step trace (thinking and tool calls). */
export type StepPart =
  | Extract<AgentPart, { type: 'reasoning' }>
  | AgentToolPart<'web_search'>
  | AgentToolPart<'search_dictionaries'>
  | AgentToolPart<'get_entry'>
  | AgentToolPart<'suggest_headwords'>
  | AgentToolPart<'browse_dictionary'>
  | AgentToolPart<'find_paradigms'>
  | AgentToolPart<'get_paradigm'>
  | AgentToolPart<'search_dhatus'>
  | AgentToolPart<'get_dhatu'>
  | AgentToolPart<'search_sutras'>
  | AgentToolPart<'get_sutra'>
  | VidyutToolPart;

export type VidyutToolName =
  | 'vidyut_conjugate'
  | 'vidyut_decline'
  | 'vidyut_krdantas'
  | 'vidyut_derivation'
  | 'vidyut_sandhi'
  | 'vidyut_meter';

export type VidyutToolPart = { [N in VidyutToolName]: AgentToolPart<N> }[VidyutToolName];

export type PartGroup =
  | { kind: 'steps'; key: string; parts: AgentPart[] }
  | { kind: 'text'; key: string; text: string };

/** A finished reasoning part without text only carries encrypted reasoning for replay. */
const isStep = (part: AgentPart): boolean =>
  (part.type === 'reasoning' && (part.text.trim() !== '' || part.state === 'streaming')) ||
  part.type === 'dynamic-tool' ||
  (typeof part.type === 'string' && part.type.startsWith('tool-'));

/**
 * Consecutive steps share one trace; text stands on its own. `step-start` parts carry no visual.
 */
export function groupParts(parts: AgentPart[]): PartGroup[] {
  const groups: PartGroup[] = [];
  parts.forEach((part, index) => {
    if (isStep(part)) {
      const last = groups.at(-1);
      if (last?.kind === 'steps') last.parts.push(part);
      else groups.push({ kind: 'steps', key: `steps-${index}`, parts: [part] });
    } else if (part.type === 'text' && part.text.trim()) {
      groups.push({ kind: 'text', key: `text-${index}`, text: part.text });
    }
  });
  return groups;
}

export function messageText(message: AgentUIMessage): string {
  return message.parts.map((part) => (part.type === 'text' ? part.text : '')).join('\n');
}
