"use client";

import { useChatStore } from "@/store/chat-store";
import { cn } from "@/lib/utils";
import { MessageSquare, Trash2, Calendar } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import { formatDistanceToNow } from "date-fns";

export function ThreadList() {
  const {
    threads,
    activeThreadId,
    setActiveThreadId,
    deleteThread,
    isLoadingThreads,
  } = useChatStore();

  const handleDelete = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    if (confirm("Are you sure you want to delete this chat?")) {
      deleteThread(id);
    }
  };

  if (isLoadingThreads) {
    return (
      <div className="space-y-2 px-3 py-2">
        {[1, 2, 3, 4].map((n) => (
          <div key={n} className="flex items-center gap-3 p-2 rounded-md">
            <Skeleton className="h-4 w-4 rounded-full" />
            <div className="space-y-1.5 flex-1">
              <Skeleton className="h-3.5 w-3/4" />
              <Skeleton className="h-2.5 w-1/2" />
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (threads.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-48 px-4 text-center">
        <MessageSquare className="size-8 text-muted-foreground/45 mb-2" />
        <p className="text-sm font-medium text-muted-foreground">No chats yet</p>
        <p className="text-xs text-muted-foreground/70 mt-0.5">
          Start a new session to get fitness recommendations.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-1 px-2 py-1 overflow-y-auto max-h-[calc(100vh-280px)]">
      {threads.map((thread) => {
        const isActive = thread.id === activeThreadId;
        const relativeTime = thread.updated_at
          ? formatDistanceToNow(new Date(thread.updated_at), { addSuffix: true })
          : "";

        return (
          <div
            key={thread.id}
            onClick={() => setActiveThreadId(thread.id)}
            className={cn(
              "group relative flex items-center justify-between gap-3 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-200 select-none",
              isActive
                ? "bg-secondary text-foreground font-medium"
                : "text-muted-foreground hover:bg-muted/65 hover:text-foreground"
            )}
          >
            <div className="flex items-center gap-3 min-w-0 flex-1">
              <MessageSquare
                className={cn(
                  "size-4 shrink-0",
                  isActive ? "text-primary" : "text-muted-foreground/60"
                )}
              />
              <div className="min-w-0 flex-1">
                <p className="text-sm truncate pr-4">
                  {thread.title || "Untitled Session"}
                </p>
                <span className="text-[10px] text-muted-foreground/75 flex items-center gap-1 mt-0.5">
                  <Calendar className="size-2.5" />
                  {relativeTime}
                </span>
              </div>
            </div>

            <button
              onClick={(e) => handleDelete(e, thread.id)}
              className={cn(
                "opacity-0 group-hover:opacity-100 p-1 hover:bg-destructive/10 rounded text-muted-foreground hover:text-destructive transition-all duration-200 absolute right-2",
                isActive && "opacity-100"
              )}
              aria-label="Delete thread"
            >
              <Trash2 className="size-3.5" />
            </button>
          </div>
        );
      })}
    </div>
  );
}
