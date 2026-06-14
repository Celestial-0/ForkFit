"use client";

import { useState } from "react";
import { useChatStore } from "@/store/chat-store";
import {
  PromptInput,
  PromptInputBody,
  PromptInputTextarea,
  PromptInputSubmit,
  PromptInputFooter,
  PromptInputMessage,
} from "@/components/ai-elements/prompt-input";

export function ChatInput() {
  const { sendMessage, isStreaming } = useChatStore();
  const [value, setValue] = useState("");

  const handleSubmit = async (
    message: PromptInputMessage,
    e: React.FormEvent<HTMLFormElement>
  ) => {
    e.preventDefault();
    if (!message.text.trim() || isStreaming) return;

    const currentPrompt = message.text;
    setValue(""); // Clear quickly for better UX responsive feel
    await sendMessage(currentPrompt);
  };

  return (
    <div className="p-3 border-t bg-background shrink-0">
      <PromptInput onSubmit={handleSubmit} className="max-w-3xl mx-auto">
        <PromptInputBody className="border rounded-xl shadow-xs focus-within:ring-1 focus-within:ring-ring focus-within:border-ring transition-all bg-muted/10">
          <PromptInputTextarea
            value={value}
            onChange={(e) => setValue(e.target.value)}
            disabled={isStreaming}
            placeholder="Type a message or request fitness plans..."
            className="min-h-11 resize-none bg-transparent pr-12 text-sm max-h-48 border-none focus-visible:ring-0 focus-visible:outline-hidden"
          />
          <PromptInputSubmit
            status={isStreaming ? "submitted" : undefined}
            disabled={!value.trim() || isStreaming}
            className="absolute right-2 bottom-2 size-7.5 rounded-lg flex items-center justify-center shrink-0 cursor-pointer shadow-xs disabled:opacity-40 disabled:cursor-not-allowed"
          />
        </PromptInputBody>
        <PromptInputFooter className="text-[8px] text-muted-foreground/75 w-full justify-center text-center mt-1.5">
          ForkFit AI can make mistakes. Verify meal plans and ingredients with your doctor.
        </PromptInputFooter>
      </PromptInput>
    </div>
  );
}
