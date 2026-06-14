"use client";

import { ThreadList } from "./thread-list";
import { ProfileFooter } from "./profile-footer";
import { Button } from "@/components/ui/button";
import { PlusCircle } from "lucide-react";
import { useChatStore } from "@/store/chat-store";
import { ForkFit } from "@/components/icons/main";
import Link from "next/link";
import { useEffect } from "react";

export function SidebarContainer() {
  const createThread = useChatStore((state) => state.createThread);
  const fetchThreads = useChatStore((state) => state.fetchThreads);

  useEffect(() => {
    fetchThreads();
  }, [fetchThreads]);

  const handleNewChat = () => {
    createThread();
  };

  return (
    <div className="flex flex-col h-full bg-sidebar text-sidebar-foreground">
      {/* Brand Header */}
      <div className="h-14 flex items-center justify-between px-4 border-b">
        <Link href="/" className="flex items-center gap-2 rounded-md p-1 hover:bg-muted/50 transition-colors">
          <ForkFit className="h-5.5 w-5.5 text-primary" />
          <span className="font-semibold text-sm tracking-tight">
            Fork<span className="text-muted-foreground font-normal">Fit</span>
          </span>
        </Link>
      </div>

      {/* Action Area */}
      <div className="p-3">
        <Button
          onClick={handleNewChat}
          className="w-full gap-2 text-xs font-semibold justify-center h-9 shadow-sm"
          variant="outline"
        >
          <PlusCircle className="size-4 text-muted-foreground" />
          <span>New Conversation</span>
        </Button>
      </div>

      {/* Navigation Header */}
      <div className="px-4 py-1 text-[10px] font-bold tracking-wider text-muted-foreground/80 uppercase">
        History
      </div>

      {/* Scrollable threads list */}
      <div className="flex-1 overflow-y-auto">
        <ThreadList />
      </div>

      {/* Footer Profile area */}
      <ProfileFooter />
    </div>
  );
}
