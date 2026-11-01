import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { nanoid } from 'nanoid';
import { SparklesIcon } from 'lucide-react';
import { loadStoredModel, storeModel, type AgentModelId } from '@/lib/agent/models';
import { Button } from '@/components/ui/button';
import { ChatComposer, type ComposerDraft } from './chat-composer';
import { ModelPicker } from './model-picker';
import { MobileHistoryButton } from './agent-sidebar';
import { setPendingMessage } from './pending-message';

const SUGGESTIONS = [
  'Decline राम in all cases',
  'Conjugate भू in लट् and लङ्',
  'What does Kāśikā say on 1.1.1?',
  'Meaning and etymology of धर्म across dictionaries'
];

/** Empty state: hero, composer, and starter prompts. Sending navigates to the new chat. */
export function NewChatPanel() {
  const navigate = useNavigate();
  const [model, setModelState] = useState<AgentModelId>(() => loadStoredModel());
  const setModel = (next: AgentModelId) => {
    setModelState(next);
    storeModel(next);
  };

  const start = (draft: ComposerDraft) => {
    const text = draft.text.trim();
    if (!text) return;
    const id = nanoid();
    setPendingMessage(id, { text });
    void navigate({ to: '/agent/$chatId', params: { chatId: id } });
  };

  return (
    <div className="flex h-full min-h-0 flex-1 flex-col">
      <header className="flex items-center gap-2 px-3 py-2 md:hidden">
        <MobileHistoryButton />
        <span className="text-sm font-semibold">New chat</span>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto flex w-full max-w-2xl flex-col gap-6 px-4 py-8 md:py-14">
          <div className="flex flex-col items-center gap-1 text-center">
            <p className="flex items-center gap-1.5 text-xs font-semibold tracking-widest text-primary uppercase">
              <SparklesIcon className="size-3.5" aria-hidden /> Dictionary agent
            </p>
            <h1 className="font-devanagari text-3xl font-bold tracking-tight">किं पृच्छसि?</h1>
            <p className="max-w-md text-sm leading-relaxed text-muted-foreground">
              Ask about Sanskrit words, roots and sūtras — the agent reads this site&apos;s
              dictionaries, inflection tables, dhātupāṭha and commentaries, plus the web.
            </p>
          </div>

          <ChatComposer
            onSend={start}
            busy={false}
            placeholder="Ask about a word, root, sūtra…"
            toolbar={<ModelPicker value={model} onChange={setModel} />}
            autoFocus
          />

          <div className="flex flex-wrap justify-center gap-2">
            {SUGGESTIONS.map((suggestion) => (
              <Button
                key={suggestion}
                variant="outline"
                size="sm"
                className="h-auto rounded-full py-1.5 whitespace-normal"
                onClick={() => start({ text: suggestion })}
              >
                <span className="font-devanagari">{suggestion}</span>
              </Button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
