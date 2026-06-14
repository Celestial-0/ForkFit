"use client";

import { useRef, useEffect } from "react";
import { useChatStore } from "@/store/chat-store";
import {
  Message,
  MessageContent,
  MessageResponse,
} from "@/components/ai-elements/message";
import {
  ChainOfThought,
  ChainOfThoughtHeader,
  ChainOfThoughtContent,
  ChainOfThoughtStep,
} from "@/components/ai-elements/chain-of-thought";
import {
  Conversation,
  ConversationContent,
  ConversationEmptyState,
  ConversationScrollButton,
} from "@/components/ai-elements/conversation";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { 
  Calendar, 
  ChevronRight, 
  Clock, 
  Eye, 
  Loader2, 
  MessageSquare, 
  ShoppingCart,
  Check,
  AlertCircle,
  Utensils
} from "lucide-react";
import { cn } from "@/lib/utils";
import { RecipeExpandableCard } from "./recipe-expandable-card";
import { ForkFit } from "@/components/icons/main";

export function MessageList() {
  const {
    messages,
    activeThreadId,
    isStreaming,
    streamedContent,
    steps,
    uiElements,
    isLoadingMessages,
    setSelectedUiElement,
    fetchRecipeDetail,
  } = useChatStore();

  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new messages or stream updates
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages, streamedContent, steps, isStreaming]);

  if (!activeThreadId) {
    return (
      <ConversationEmptyState
        title="Welcome to ForkFit AI"
        description="Ask your personalized nutritionist and culinary assistant to formulate meal plans, search pantry items, or craft custom recipes."
        icon={<ForkFit className="size-10 text-primary animate-pulse" />}
      />
    );
  }

  if (isLoadingMessages && messages.length === 0) {
    return (
      <div className="flex-1 flex flex-col gap-6 p-4 overflow-y-auto">
        <div className="flex items-start gap-3 max-w-[80%]">
          <Skeleton className="size-8 rounded-full" />
          <div className="space-y-2 flex-1">
            <Skeleton className="h-4 w-32" />
            <Skeleton className="h-10 w-full rounded-lg" />
          </div>
        </div>
        <div className="flex items-start gap-3 max-w-[80%] ml-auto flex-row-reverse">
          <Skeleton className="size-8 rounded-full" />
          <div className="space-y-2 flex-1">
            <Skeleton className="h-4 w-24" />
            <Skeleton className="h-14 w-full rounded-lg" />
          </div>
        </div>
      </div>
    );
  }

interface UiData {
  items?: Array<unknown>;
  recipe?: {
    id?: string;
    servings?: number;
  };
  id?: string;
  servings?: number;
}

interface UiElement {
  type: string;
  title: string;
  config: unknown;
  data: unknown;
}

  const renderUiCard = (element: UiElement, index: number) => {
    const isMealPlan = element.type === "meal_plan";
    const isRecipe = element.type === "recipe";
    const Icon = isMealPlan ? Calendar : isRecipe ? Utensils : ShoppingCart;
    const data = (element.data as UiData) || {};

    return (
      <Card key={index} className="border bg-card/65 shadow-xs w-full border-primary/20 hover:border-primary/45 transition-all duration-300">
        <CardHeader className="p-3 pb-2">
          <div className="flex items-center gap-2">
            <div className="size-7 rounded-lg bg-primary/5 flex items-center justify-center">
              <Icon className="size-4 text-primary" />
            </div>
            <div className="min-w-0">
              <CardTitle className="text-xs font-semibold text-foreground truncate">
                {element.title}
              </CardTitle>
              <CardDescription className="text-[10px] text-muted-foreground mt-0.5">
                {isMealPlan ? "Generated Diet Schedule" : isRecipe ? "Detailed Recipe Guide" : "Generated Grocery List"}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="p-3 pt-0 pb-3 flex justify-between items-center gap-1">
          <div className="text-[10px] text-muted-foreground">
            {isMealPlan ? (
              <span>Days: {data.items?.length || 0} meals planned</span>
            ) : isRecipe ? (
              <span>Servings: {data.recipe?.servings || data.servings || 0} servings</span>
            ) : (
              <span>Items: {data.items?.length || 0} groceries listed</span>
            )}
          </div>
          <Button
            size="xs"
            onClick={async () => {
              if (isRecipe) {
                const recipeId = data.recipe?.id || data.id;
                if (recipeId) {
                  await fetchRecipeDetail(recipeId);
                  return;
                }
              }
              setSelectedUiElement(element);
            }}
            className="text-[10px] font-semibold gap-1 px-2.5 h-7 cursor-pointer"
          >
            <Eye className="size-3" />
            <span>Open in Panel</span>
            <ChevronRight className="size-3" />
          </Button>
        </CardContent>
      </Card>
    );
  };

  return (
    <Conversation className="flex-1 overflow-y-auto [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] scrollbar-none">
      <ConversationContent className="w-full px-4 py-6">
        <div className="max-w-3xl mx-auto w-full flex flex-col gap-6">
          {messages.map((message) => {
            const isUser = message.sender_role === "user";
            const parsedUiElements = message.metadata?.ui_elements || [];

            return (
              <Message key={message.id} from={message.sender_role}>
                <div className={cn("flex items-start gap-3 w-full", isUser && "flex-row-reverse")}>
                  <div
                    className={cn(
                      "size-7.5 rounded-full flex items-center justify-center shrink-0 border text-xs select-none",
                      isUser
                        ? "bg-secondary text-secondary-foreground"
                        : "bg-primary text-primary-foreground font-semibold"
                    )}
                  >
                    {isUser ? <MessageSquare className="size-3.5" /> : "AI"}
                  </div>

                  <div className="space-y-1 min-w-0 flex-1">
                    <span className="text-[10px] font-semibold text-muted-foreground/85 px-1">
                      {isUser ? "You" : "ForkFit Assistant"}
                    </span>
                    <MessageContent className={cn("rounded-xl p-3 m-2 border shadow-2xs max-w-full w-full", isUser ? "bg-muted/15 w-fit" : "bg-card")}>
                      {isUser ? (
                        <p className="whitespace-pre-wrap leading-relaxed">{message.content}</p>
                      ) : (
                        <MessageResponse>{message.content}</MessageResponse>
                      )}
                    </MessageContent>

                    {/* Inline UI Cards saved in database metadata */}
                    {parsedUiElements.length > 0 && (
                      <div className="grid grid-cols-1 sm:grid-cols-2 gap-3.5 mt-3 ml-1.5 w-full">
                        {parsedUiElements
                          .filter((el) => el.type === "meal_plan" || el.type === "shopping_list" || el.type === "recipe")
                          .map((el, idx: number) => {
                            if (el.type === "recipe") {
                              const data = (el.data as UiData) || {};
                              const recipeId = data.recipe?.id || data.id || "";
                              return (
                                <RecipeExpandableCard
                                  key={idx}
                                  recipeId={recipeId}
                                  initialTitle={el.title}
                                  initialDescription="Detailed Recipe Guide"
                                />
                              );
                            }
                            return renderUiCard(el, idx);
                          })}
                      </div>
                    )}
                  </div>
                </div>
              </Message>
            );
          })}

          {/* Live streaming/execution assistant response */}
          {isStreaming && (
            <Message from="assistant">
              <div className="flex items-start gap-3 w-full">
                <div className="size-7.5 rounded-full bg-primary text-primary-foreground flex items-center justify-center shrink-0 border text-xs font-semibold animate-pulse">
                  AI
                </div>

                <div className="space-y-2 min-w-0 flex-1">
                  <span className="text-[10px] font-semibold text-muted-foreground/85 px-1">
                    ForkFit Assistant
                  </span>

                  {/* Chain of thought steps from active state */}
                  {steps.length > 0 && (
                    <ChainOfThought defaultOpen={true} className="border rounded-lg p-2.5 bg-muted/5">
                      <ChainOfThoughtHeader className="text-xs font-medium">
                        Thinking Steps ({steps.filter((s) => s.status === "completed").length}/{steps.length})
                      </ChainOfThoughtHeader>
                      <ChainOfThoughtContent>
                        {steps.map((step, idx) => {
                          const isDone = step.status === "completed";
                          const isErr = step.status === "failed";
                          const isActive = step.status === "running";
                          let Icon = Clock;
                          if (isDone) Icon = Check;
                          if (isErr) Icon = AlertCircle;
                          if (isActive) Icon = Loader2;

                          return (
                            <ChainOfThoughtStep
                              key={idx}
                              label={step.agent.replace(/_/g, " ")}
                              status={isDone ? "complete" : isActive ? "active" : "pending"}
                              icon={Icon}
                              description={
                                step.latency_ms
                                  ? `Latency: ${step.latency_ms}ms`
                                  : isActive
                                  ? "Orchestrating..."
                                  : "Pending"
                              }
                            />
                          );
                        })}
                      </ChainOfThoughtContent>
                    </ChainOfThought>
                  )}

                  {/* Text stream response */}
                  {(streamedContent || isStreaming) && (
                    <MessageContent className="rounded-xl p-3 border shadow-2xs bg-card max-w-full w-full">
                      {streamedContent ? (
                        <MessageResponse>{streamedContent}</MessageResponse>
                      ) : (
                        <div className="flex items-center gap-2 text-muted-foreground/80 py-1.5 px-0.5 text-xs">
                          <Loader2 className="size-3.5 animate-spin" />
                          <span>Formulating recommendations...</span>
                        </div>
                      )}
                    </MessageContent>
                  )}

                  {/* Live structured UI Elements cards generated in this active stream */}
                  {uiElements.length > 0 && (
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3.5 mt-3 w-full">
                      {uiElements
                        .filter((el) => el.type === "meal_plan" || el.type === "shopping_list" || el.type === "recipe")
                        .map((el, idx) => {
                          if (el.type === "recipe") {
                            const data = (el.data as UiData) || {};
                            const recipeId = data.recipe?.id || data.id || "";
                            return (
                              <RecipeExpandableCard
                                key={idx}
                                recipeId={recipeId}
                                initialTitle={el.title}
                                initialDescription="Detailed Recipe Guide"
                              />
                            );
                          }
                          return renderUiCard(el, idx);
                        })}
                    </div>
                  )}
                </div>
              </div>
            </Message>
          )}

          {/* scroll anchor */}
          <div ref={scrollRef} />
        </div>
      </ConversationContent>

      <ConversationScrollButton />
    </Conversation>
  );
}
