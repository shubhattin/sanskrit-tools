// Typed mirror of `docs/agent.md` (types only — no runtime imports from the
// server). `ParadigmTable` and `Dhatu` are the shared API types from
// `frontend/src/lib/api/types.ts`.

import type { UIMessage } from 'ai';
import type { Dhatu, ParadigmTable } from '@/lib/api/types';

export type AgentModelId = 'gpt-6-luna' | 'claude-sonnet-5.5';

export interface AgentChatSummary {
  id: string;
  title: string;
  model: AgentModelId;
  streaming: boolean;
  /** ISO timestamps. */
  created_at: string;
  updated_at: string;
}

export interface AgentChat extends AgentChatSummary {
  dicts: string[];
}

export interface AgentMessageMetadata {
  model?: AgentModelId;
  /** USD actually charged by OpenRouter for the reply's model calls (web searches are billed to OpenAI separately). */
  cost?: number;
  usage?: {
    inputTokens?: number;
    outputTokens?: number;
    cachedInputTokens?: number;
    reasoningTokens?: number;
  };
  /** Finished web_search tool calls in this reply. */
  webSearches?: number;
}

export type AgentUITools = {
  web_search: {
    /** Runs through OpenAI's web_search, the same for both models. */
    input: { query: string };
    output: { summary: string; sources: { url: string; title?: string }[] };
  };
  search_dictionaries: {
    input: { query: string; dicts?: string[]; mode?: 'sa' | 'en'; limit?: number };
    output: {
      query: string;
      mode: 'sa' | 'en';
      slp1: string[];
      /** Dictionary slugs searched. */
      searched: string[] | 'all';
      results: {
        dict: string;
        dict_name: string;
        total: number;
        entries: { id: number; headword: string; match: string; text: string }[];
      }[];
      /** Inflected-form analysis. */
      forms: {
        form: string;
        lemma: string;
        root: string | null;
        analysis: string;
        source: string;
        paradigm_ids: number[];
      }[];
      did_you_mean: string[];
    };
  };
  get_entry: {
    input: { id: number };
    output: {
      id: number;
      headword: string;
      dict: string;
      dict_name: string;
      text: string;
      truncated: boolean;
      sutra_id: string | null;
      dhatu_id: number | null;
    };
  };
  suggest_headwords: {
    input: { prefix: string; limit?: number };
    output: { items: { headword: string; count: number }[] };
  };
  browse_dictionary: {
    input: { dict: string; headword: string; before?: number; after?: number };
    output: {
      dict: string;
      dict_name: string;
      center: number;
      items: { id: number; seq: number; headword: string }[];
    };
  };
  find_paradigms: {
    input: { query: string };
    output: {
      query: string;
      total: number;
      paradigms: {
        id: number;
        source: string;
        lemma: string;
        root: string | null;
        label: string | null;
      }[];
    };
  };
  get_paradigm: {
    input: { id: number; group?: string; title?: string };
    output: {
      id: number;
      source: string;
      lemma: string;
      root: string | null;
      label: string | null;
      /** Same shape as /api/paradigms/{id}. */
      tables: ParadigmTable[];
      /** All available groups/titles. */
      groups: { group: string | null; titles: (string | null)[] }[];
      filtered: boolean;
    };
  };
  search_dhatus: {
    input: { query?: string; gana?: number };
    output: { total: number; dhatus: Dhatu[] };
  };
  get_dhatu: {
    input: { id: number };
    output: {
      dhatu: Dhatu;
      works: {
        dict: string;
        dict_name: string;
        entries: { id: number; headword: string; text: string }[];
      }[];
    };
  };
  search_sutras: {
    input: { query: string };
    output: { total: number; sutras: { id: string; text: string }[] };
  };
  get_sutra: {
    input: { id: string; commentaries?: string[] };
    output: {
      sutra: { id: string; text: string };
      prev: string | null;
      next: string | null;
      commentaries: { dict: string; dict_name: string; text: string; truncated: boolean }[];
    };
  };
};

export type AgentUIMessage = UIMessage<AgentMessageMetadata, never, AgentUITools>;

/** Exactly what the model is given (for the Context dialog). Mirror of `docs/agent.md`. */
export interface AgentContext {
  /** Full text, Markdown. */
  system_prompt: string;
  /** JSON Schema per tool. */
  tools: { name: string; description: string; parameters: unknown }[];
  /** Provider-run tools (currently none — web_search is an ordinary tool). */
  server_tools: { type: string; parameters: unknown }[];
  /** Per turn. */
  max_tool_calls: number;
  models: { id: AgentModelId; provider_id: string }[];
}
