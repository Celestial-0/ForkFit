"use client";

import { useChatStore } from "@/store/chat-store";
import { cn } from "@/lib/utils";
import {  Menu, Sidebar } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  TooltipProvider,
} from "@/components/ui/tooltip";

interface ChatHeaderProps {
  onToggleSidebar?: () => void;
  onToggleRightPanel?: () => void;
}

export function ChatHeader({ onToggleSidebar, onToggleRightPanel }: ChatHeaderProps) {
  const {  isStreaming } = useChatStore();


  return (
    <header className="h-14 border-b flex items-center justify-between px-4 bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60 sticky top-0 z-40 shrink-0">
      <div className="flex items-center gap-3 min-w-0">
        {onToggleSidebar && (
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={onToggleSidebar}
            className="md:hidden text-muted-foreground hover:text-foreground"
          >
            <Menu className="size-4.5" />
          </Button>
        )}
        <div className="flex items-center gap-2 min-w-0">
          
          <div className="min-w-0">
            
            <div className="flex items-center gap-1.5 mt-0.5">
              <span
                className={cn(
                  "size-1.5 rounded-full shrink-0",
                  isStreaming ? "bg-blue-500 animate-ping" : "bg-emerald-500"
                )}
              />
              <span className="text-xs font-medium text-muted-foreground">
                {isStreaming ? "AI is formulating plan..." : "Ready to consult"}
              </span>
            </div>
          </div>
        </div>
      </div>

      {onToggleRightPanel && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-sm"
                  onClick={onToggleRightPanel}
                  className="text-muted-foreground hover:text-foreground hidden md:inline-flex cursor-pointer"
                />
              }
            >
              <Sidebar className="size-4.5 rotate-180" />
            </TooltipTrigger>
            <TooltipContent>
              <p>Toggle Agent Runtime</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}
    </header>
  );
}
