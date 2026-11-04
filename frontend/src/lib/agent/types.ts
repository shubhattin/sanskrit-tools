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
  vidyut_conjugate: {
    input: {
      root: string;
      gana?: number;
      prayoga?: VidyutPrayoga;
      lakara?: VidyutLakara;
      sanadi?: VidyutSanadi[];
      upasargas?: string[];
    };
    output: {
      root: VidyutRoot;
      homonyms: VidyutRoot[];
      stems: string[];
      prayoga: VidyutPrayoga;
      columns: string;
      /** `rows`: "प्रथमपुरुषः: a | b | c". */
      tables: { pada: string; lakara: VidyutLakara; rows: string[] }[];
      page: string;
    };
  };
  vidyut_decline: {
    input: { stem: string; linga: VidyutLinga };
    output: { stem: string; linga: VidyutLinga; columns: string; rows: string[]; page: string };
  };
  vidyut_krdantas: {
    input: { root: string; gana?: number; sanadi?: VidyutSanadi[]; upasargas?: string[] };
    output: {
      root: VidyutRoot;
      homonyms: VidyutRoot[];
      groups: { group: string; items: string[] }[];
      page: string;
    };
  };
  vidyut_derivation: {
    input: {
      root?: string;
      gana?: number;
      sanadi?: VidyutSanadi[];
      upasargas?: string[];
      prayoga?: VidyutPrayoga;
      lakara?: VidyutLakara;
      purusha?: 'prathama' | 'madhyama' | 'uttama';
      stem?: string;
      linga?: VidyutLinga;
      vibhakti?: string;
      vacana?: 'eka' | 'dvi' | 'bahu';
      form?: string;
      analysis?: string;
    };
    output: {
      results: string[];
      form: string;
      optional_rules: { rule: string; applied: boolean }[];
      steps: { rule: string; sutra_id: string | null; text: string | null; result: string }[];
      truncated: boolean;
    };
  };
  vidyut_sandhi: {
    input: { join?: string[]; split?: string };
    output:
      | { words: string[]; joined: string[]; page: string }
      | {
          note: string;
          chunks: {
            chunk: string;
            readings: { reading: string; cheda: boolean; words: string[] }[];
            splits: string[];
          }[];
          page: string;
        };
  };
  vidyut_analyze: {
    input: { text: string };
    output: {
      note: string;
      chunks: {
        chunk: string;
        reading: string | null;
        by: 'cheda' | 'optimizer' | null;
        words: { word: string; in_dictionaries: boolean; analyses: string[] }[];
        other_readings: string[];
      }[];
      page: string;
    };
  };
  vidyut_meter: {
    input: { verse: string };
    output: {
      meter: { name: string; kind: string; fit: string; note: string | null } | null;
      lines: { syllables: string; pattern: string; ganas: string; matras: number }[];
      also_compatible: string[];
      page: string;
    };
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

export type VidyutLakara =
  | 'lat'
  | 'lit'
  | 'lut'
  | 'lrt'
  | 'lot'
  | 'lan'
  | 'vidhi_lin'
  | 'ashir_lin'
  | 'lun'
  | 'lrn';
export type VidyutPrayoga = 'kartari' | 'karmani' | 'bhave';
export type VidyutLinga = 'pum' | 'stri' | 'napumsaka';
export type VidyutSanadi = 'nic' | 'san' | 'yan' | 'yan_luk';
export interface VidyutRoot {
  code: string;
  upadesha: string;
  root: string;
  artha: string;
  gana: number;
}

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
