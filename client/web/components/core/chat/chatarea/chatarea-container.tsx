"use client";

import { ChatHeader } from "./chat-header";
import { MessageList } from "./message-list";
import { ChatInput } from "./chat-input";

interface ChatareaContainerProps {
  onToggleSidebar?: () => void;
  onToggleRightPanel?: () => void;
}

export function ChatareaContainer({ onToggleSidebar, onToggleRightPanel }: ChatareaContainerProps) {
  return (
    <div className="flex flex-col h-full bg-background relative overflow-hidden">
      {/* Header */}
      <ChatHeader onToggleSidebar={onToggleSidebar} onToggleRightPanel={onToggleRightPanel} />

      {/* Message List area */}
      <div className="flex-1 overflow-y-hidden flex flex-col">
        <MessageList />
      </div>

      {/* Input Form area */}
      <ChatInput />
    </div>
  );
}
