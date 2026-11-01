import { useState } from 'react';
import { Link, useNavigate, useParams } from '@tanstack/react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  MessageSquareTextIcon,
  MoreHorizontalIcon,
  PanelLeftIcon,
  PencilIcon,
  PlusIcon,
  SearchIcon,
  Trash2Icon
} from 'lucide-react';
import { toast } from 'sonner';
import {
  agentChatQuery,
  agentChatsQuery,
  deleteAgentChat,
  renameAgentChat
} from '@/lib/agent/client';
import type { AgentChatSummary } from '@/lib/agent/types';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import { Input } from '@/components/ui/input';
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '@/components/ui/sheet';
import { Skeleton } from '@/components/ui/skeleton';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';
import { groupChatsByDate } from './chat-groups';

type ChatRef = { id: string; title: string };

function ChatHistory({ onNavigate }: { onNavigate?: () => void }) {
  const params = useParams({ strict: false });
  const chatsQuery = useQuery({
    ...agentChatsQuery(),
    // While any chat streams in the background, keep the list (and its badges) fresh.
    refetchInterval: (query) => (query.state.data?.some((chat) => chat.streaming) ? 3000 : false)
  });
  const [search, setSearch] = useState('');
  const [renaming, setRenaming] = useState<ChatRef | null>(null);
  const [deleting, setDeleting] = useState<ChatRef | null>(null);

  const needle = search.trim().toLowerCase();
  const chats = (chatsQuery.data ?? []).filter(
    (chat) => !needle || (chat.title || 'New chat').toLowerCase().includes(needle)
  );

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-col gap-2 px-3 pb-2">
        <Link to="/agent" onClick={onNavigate}>
          <Button variant="outline" className="w-full justify-start">
            <PlusIcon />
            New chat
          </Button>
        </Link>
        <div className="flex items-center gap-2 rounded-md border border-input bg-transparent px-2">
          <SearchIcon className="size-3.5 shrink-0 text-muted-foreground" aria-hidden />
          <input
            value={search}
            onChange={(event) => setSearch(event.currentTarget.value)}
            placeholder="Search chats"
            aria-label="Search chats"
            className="h-8 w-full bg-transparent text-sm outline-none placeholder:text-muted-foreground"
          />
        </div>
      </div>

      <nav aria-label="Chat history" className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
        {chatsQuery.isPending ? (
          <div className="flex flex-col gap-1.5 px-1">
            {Array.from({ length: 6 }, (_, index) => (
              <Skeleton key={index} className="h-8 w-full" />
            ))}
          </div>
        ) : chats.length === 0 ? (
          <p className="px-2 py-6 text-center text-xs text-muted-foreground">
            {chatsQuery.data?.length ? 'No chats match.' : 'No chats yet.'}
          </p>
        ) : (
          groupChatsByDate(chats).map((group) => (
            <section key={group.label} className="mb-3">
              <h2 className="px-2 pb-1 text-xs font-medium text-muted-foreground">{group.label}</h2>
              <ul className="flex flex-col gap-0.5">
                {group.chats.map((chat) => {
                  const active = params.chatId === chat.id;
                  return (
                    <li key={chat.id} className="group/chat relative">
                      <Link
                        to="/agent/$chatId"
                        params={{ chatId: chat.id }}
                        onClick={onNavigate}
                        className={cn(
                          'flex items-center gap-2 rounded-lg py-1.5 pr-8 pl-2 text-sm no-underline transition-colors',
                          active
                            ? 'bg-muted font-medium text-foreground'
                            : 'text-foreground/80 hover:bg-muted/60 hover:text-foreground'
                        )}
                      >
                        <MessageSquareTextIcon
                          className="size-4 shrink-0 text-muted-foreground"
                          aria-hidden
                        />
                        <span className="min-w-0 flex-1 truncate">{chat.title || 'New chat'}</span>
                        {chat.streaming ? (
                          <span
                            title="Replying in the background"
                            className="size-1.5 shrink-0 animate-pulse rounded-full bg-primary"
                          />
                        ) : null}
                      </Link>
                      <DropdownMenu>
                        <DropdownMenuTrigger
                          render={
                            <Button
                              variant="ghost"
                              size="icon-xs"
                              type="button"
                              aria-label={`Actions for ${chat.title || 'chat'}`}
                              className="absolute top-1/2 right-1 -translate-y-1/2 opacity-0 group-hover/chat:opacity-100 focus-visible:opacity-100 data-popup-open:opacity-100 max-md:opacity-100"
                            />
                          }
                        >
                          <MoreHorizontalIcon />
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end" className="w-36">
                          <DropdownMenuItem onClick={() => setRenaming(chat)}>
                            <PencilIcon />
                            Rename
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem variant="destructive" onClick={() => setDeleting(chat)}>
                            <Trash2Icon />
                            Delete
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </li>
                  );
                })}
              </ul>
            </section>
          ))
        )}
      </nav>

      {renaming ? (
        <RenameChatDialog key={renaming.id} chat={renaming} onClose={() => setRenaming(null)} />
      ) : null}
      <DeleteChatDialog chat={deleting} onClose={() => setDeleting(null)} />
    </div>
  );
}

/** Chat history beside the conversation (md and up). */
export function AgentSidebar() {
  return (
    <aside className="hidden w-72 shrink-0 flex-col border-r border-border/70 bg-muted/20 pt-3 md:flex">
      <ChatHistory />
    </aside>
  );
}

/** Opens the chat history in a sheet on small screens. */
export function MobileHistoryButton() {
  const [open, setOpen] = useState(false);
  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger
        render={
          <Button
            variant="ghost"
            size="icon-sm"
            type="button"
            aria-label="Chat history"
            className="shrink-0 md:hidden"
          />
        }
      >
        <PanelLeftIcon className="size-4" />
      </SheetTrigger>
      <SheetContent side="left" className="w-[82%] max-w-xs gap-0 p-0">
        <SheetHeader className="border-b p-4 pb-3">
          <SheetTitle className="text-sm font-semibold">Chats</SheetTitle>
        </SheetHeader>
        <div className="flex min-h-0 flex-1 flex-col pt-3">
          <ChatHistory onNavigate={() => setOpen(false)} />
        </div>
      </SheetContent>
    </Sheet>
  );
}

function RenameChatDialog({ chat, onClose }: { chat: ChatRef | null; onClose: () => void }) {
  const queryClient = useQueryClient();
  const [title, setTitle] = useState(chat?.title ?? '');

  const renameMut = useMutation({
    mutationFn: (input: { id: string; title: string }) => renameAgentChat(input.id, input.title),
    onSuccess: (_, input) => {
      void queryClient.invalidateQueries({ queryKey: agentChatsQuery().queryKey });
      void queryClient.invalidateQueries({ queryKey: agentChatQuery(input.id).queryKey });
      onClose();
    },
    onError: (error: Error) => toast.error(error.message)
  });

  return (
    <Dialog open={chat !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <form
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            if (chat && title.trim()) renameMut.mutate({ id: chat.id, title: title.trim() });
          }}
        >
          <DialogHeader>
            <DialogTitle>Rename chat</DialogTitle>
          </DialogHeader>
          <Input
            value={title}
            onChange={(event) => setTitle(event.currentTarget.value)}
            maxLength={120}
            autoFocus
            aria-label="Chat title"
          />
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              Cancel
            </Button>
            <Button type="submit" disabled={!title.trim() || renameMut.isPending}>
              {renameMut.isPending ? <Spinner /> : null}
              Save
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function DeleteChatDialog({ chat, onClose }: { chat: ChatRef | null; onClose: () => void }) {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const params = useParams({ strict: false });

  const deleteMut = useMutation({
    mutationFn: (id: string) => deleteAgentChat(id),
    onSuccess: (_, id) => {
      void queryClient.invalidateQueries({ queryKey: agentChatsQuery().queryKey });
      if (params.chatId === id) void navigate({ to: '/agent' });
      onClose();
    },
    onError: (error: Error) => toast.error(error.message)
  });

  return (
    <Dialog open={chat !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete “{chat?.title || 'New chat'}”?</DialogTitle>
        </DialogHeader>
        <p className="text-sm text-muted-foreground">
          The conversation and its tool calls are deleted. This cannot be undone.
        </p>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={onClose} disabled={deleteMut.isPending}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            disabled={deleteMut.isPending || !chat}
            onClick={() => chat && deleteMut.mutate(chat.id)}
          >
            {deleteMut.isPending ? <Spinner /> : null}
            Delete
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export type { AgentChatSummary };
