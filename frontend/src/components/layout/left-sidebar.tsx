import { MessageSquare, MoreHorizontal, Pencil, Plus, Search, Trash2 } from "lucide-react";
import { useLayoutEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useThreadUi } from "@/hooks/use-thread-ui";
import { useUiState } from "@/hooks/use-ui-state";
import { cn, formatRelativeTime } from "@/lib/utils";
import type { LocalThread } from "@/types";
import type { RichMessage } from "@/types/chat-content";

/** Extract a plain-text preview (up to ~50 chars) from the last message in a thread. */
function getLastMessagePreview(messages: RichMessage[] | undefined): string {
  if (!messages || messages.length === 0) return "";
  const last = messages[messages.length - 1];
  for (const block of last.content) {
    if (block.type === "text" && block.text.trim()) {
      const text = block.text.trim();
      return text.length > 50 ? `${text.slice(0, 50)}...` : text;
    }
  }
  if (last.role === "assistant") {
    const tool = last.chunks?.find((chunk) => chunk.kind === "tool-call");
    if (tool?.kind === "tool-call") return `Used ${tool.toolName}`;
  }
  return "";
}

interface LeftSidebarProps { className?: string }

export function LeftSidebar({ className }: LeftSidebarProps) {
  const [search, setSearch] = useState("");
  const [renameThreadId, setRenameThreadId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [deleteThreadId, setDeleteThreadId] = useState<string | null>(null);
  const { setMobileSidebarOpen } = useUiState();
  const { threads, activeThreadId, hydrated, registerThread, setTitle, setActive, removeThread, messagesByThread, clearThread } = useThreadUi();

  const visibleThreads: LocalThread[] = Object.values(threads)
    .filter((t) => !t.isEphemeral)
    .filter((t) => t.title.toLowerCase().includes(search.toLowerCase()))
    .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime());

  const deleteThread = deleteThreadId ? threads[deleteThreadId] : null;

  useLayoutEffect(() => {
    if (!hydrated) return;
    const frame = requestAnimationFrame(() => {
      performance.mark("uar-thread-list:first-paint-frame");
    });
    return () => cancelAnimationFrame(frame);
  }, [hydrated]);

  const handleNewThread = () => {
    const id = crypto.randomUUID();
    registerThread(id);
    setActive(id);
    setMobileSidebarOpen(false);
  };

  const handleSelectThread = (thread: LocalThread) => {
    setActive(thread.id);
    setMobileSidebarOpen(false);
  };

  const handleConfirmDeleteThread = () => {
    if (!deleteThreadId) return;

    const id = deleteThreadId;
    removeThread(id);
    clearThread(id);
    if (activeThreadId === id) setActive(null);
    setDeleteThreadId(null);
  };

  const handleRenameSubmit = () => {
    if (!renameThreadId) return;
    const nextTitle = renameValue.trim();
    if (!nextTitle) return;

    setTitle(renameThreadId, nextTitle);
    setRenameThreadId(null);
    setRenameValue("");
  };

  return (
    <>
      <aside className={cn("flex h-full flex-col bg-chrome", className)}>
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-3">
        <span className="font-mono text-[11px] font-medium uppercase tracking-widest text-muted-foreground">Threads</span>
        <Button variant="ghost" size="sm" onClick={handleNewThread} className="h-7 gap-1.5 px-2.5 font-ui text-xs font-semibold text-muted-foreground hover:bg-ember-soft hover:text-ember" aria-label="New thread">
          <Plus size={14} />New thread
        </Button>
      </div>

      {/* Search */}
      <div className="px-3 py-2">
        <div className="flex items-center gap-2 rounded-md bg-surface px-2.5 py-1.5">
          <Search size={14} className="text-muted-foreground" />
          <input
            type="text"
            placeholder="Search threads…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-transparent font-ui text-sm text-foreground placeholder:text-muted-foreground focus:outline-none"
            aria-label="Search threads"
          />
        </div>
      </div>

      {/* Thread list */}
      <div
        className="flex-1 overflow-y-auto px-1.5 py-1"
        aria-busy={!hydrated}
        aria-label="Thread list"
      >
        {visibleThreads.length === 0 ? (
          <div className="px-3 py-8 text-center">
            <p className="font-mono text-[11px] text-primary">{"No threads yet"}</p>
            <p className="mt-1 font-body text-xs text-muted-foreground">Start a new thread above</p>
          </div>
        ) : (
          visibleThreads.map((thread) => {
            const preview = getLastMessagePreview(messagesByThread[thread.id]);
            return (
            <div key={thread.id} className="group relative">
              <Button
                variant="ghost"
                onClick={() => handleSelectThread(thread)}
                className={cn(
                  "h-auto w-full cursor-pointer justify-start gap-2 rounded-md px-2.5 py-2 pr-9 text-left",
                  activeThreadId === thread.id
                    ? "bg-ember-soft hover:bg-ember-soft"
                    : "hover:bg-card-hov",
                )}
                aria-current={activeThreadId === thread.id ? "page" : undefined}
              >
                <MessageSquare size={14} className="mt-0.5 shrink-0 text-muted-foreground" />
                <div className="min-w-0 flex-1">
                  <div className="flex items-baseline justify-between gap-2">
                    <p className="truncate font-display text-sm font-semibold text-foreground">{thread.title}</p>
                    <span className="shrink-0 text-xs text-muted-foreground">{formatRelativeTime(thread.updatedAt)}</span>
                  </div>
                  <p className="text-xs text-muted-foreground line-clamp-1">
                    <span className="font-mono">Default Assistant</span>
                    {preview && <>{" \u00B7 "}{preview}</>}
                  </p>
                </div>
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger
                  render={
                    <Button
                      variant="ghost"
                      size="icon"
                      className="absolute right-1 top-1/2 z-10 flex size-6 -translate-y-1/2 opacity-0 pointer-events-none text-muted-foreground transition-opacity hover:text-foreground group-hover:opacity-100 group-hover:pointer-events-auto data-[state=open]:opacity-100 data-[state=open]:pointer-events-auto"
                      aria-label={`Actions for ${thread.title}`}
                    />
                  }
                >
                  <MoreHorizontal size={14} />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" sideOffset={6}>
                  <DropdownMenuItem
                    onSelect={() => {
                      setRenameThreadId(thread.id);
                      setRenameValue(thread.title);
                    }}
                  >
                    <Pencil size={14} />Rename
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    className="text-destructive focus:text-destructive"
                    onSelect={() => setDeleteThreadId(thread.id)}
                  >
                    <Trash2 size={14} />Delete
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
            );
          })
        )}
      </div>
      </aside>

      <Dialog
        open={renameThreadId !== null}
        onOpenChange={(open) => {
          if (!open) {
            setRenameThreadId(null);
            setRenameValue("");
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Rename thread</DialogTitle>
            <DialogDescription>Enter a new name for this thread.</DialogDescription>
          </DialogHeader>
          <Input
            value={renameValue}
            onChange={(e) => setRenameValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                handleRenameSubmit();
              }
            }}
            placeholder="Thread name"
            autoFocus
            aria-label="Thread name"
          />
          <DialogFooter>
            <Button variant="ghost" onClick={() => {
              setRenameThreadId(null);
              setRenameValue("");
            }}>
              Cancel
            </Button>
            <Button onClick={handleRenameSubmit} disabled={!renameValue.trim()}>
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog
        open={deleteThreadId !== null}
        onOpenChange={(open) => {
          if (!open) setDeleteThreadId(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete thread?</AlertDialogTitle>
            <AlertDialogDescription>
              {deleteThread ? `This will permanently delete "${deleteThread.title}".` : "This action cannot be undone."}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmDeleteThread}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
