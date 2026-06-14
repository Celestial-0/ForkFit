import { create } from "zustand";
import { useAuthStore } from "./auth-store";
import { toast } from "sonner";
import {
  ChatThread,
  ChatMessage,
  AgentMemory,
  fetchThreadsApi,
  createThreadApi,
  deleteThreadApi,
  fetchMessagesApi,
  postMessageApi,
  orchestrateApi,
  fetchActiveMemoriesApi,
  deactivateMemoryApi,
  submitFeedbackApi,
  fetchRecipeDetailApi,
} from "@/lib/api/chat";
import { API_BASE_URL } from "@/lib/api/api";

function getErrorMessage(err: unknown, fallback: string): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "object" && err !== null && "message" in err) {
    return String((err as { message: unknown }).message);
  }
  return fallback;
}

interface ChatState {
  threads: ChatThread[];
  activeThreadId: string | null;
  messages: ChatMessage[];
  isLoadingThreads: boolean;
  isLoadingMessages: boolean;
  isStreaming: boolean;
  streamedContent: string;
  steps: Array<{
    agent: string;
    status: string;
    step_type: string;
    latency_ms?: number;
  }>;
  uiElements: Array<{
    type: string;
    title: string;
    config: unknown;
    data: unknown;
  }>;
  activeTraceId: string | null;
  memories: AgentMemory[];
  rightSidebarView: "runtime" | "detail";
  selectedUiElement: {
    type: string;
    title: string;
    config: unknown;
    data: unknown;
  } | null;

  fetchThreads: () => Promise<void>;
  createThread: (title?: string) => Promise<string>;
  setActiveThreadId: (id: string | null) => Promise<void>;
  deleteThread: (id: string) => Promise<void>;
  sendMessage: (prompt: string) => Promise<void>;
  fetchActiveMemories: () => Promise<void>;
  deactivateMemory: (id: string) => Promise<void>;
  submitFeedback: (payload: {
    category: string;
    reference_id?: string;
    rating: number;
    comment?: string;
  }) => Promise<void>;
  setRightSidebarView: (view: "runtime" | "detail") => void;
  setSelectedUiElement: (
    element: {
      type: string;
      title: string;
      config: unknown;
      data: unknown;
    } | null
  ) => void;
  fetchRecipeDetail: (recipeId: string) => Promise<void>;
}

export const useChatStore = create<ChatState>((set, get) => ({
  threads: [],
  activeThreadId: null,
  messages: [],
  isLoadingThreads: false,
  isLoadingMessages: false,
  isStreaming: false,
  streamedContent: "",
  steps: [],
  uiElements: [],
  activeTraceId: null,
  memories: [],
  rightSidebarView: "runtime",
  selectedUiElement: null,

  setRightSidebarView: (view) => set({ rightSidebarView: view }),
  setSelectedUiElement: (element) =>
    set({
      selectedUiElement: element,
      rightSidebarView: element ? "detail" : "runtime",
    }),

  fetchThreads: async () => {
    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    set({ isLoadingThreads: true });
    try {
      const response = await fetchThreadsApi(token, 1, 50);
      set({ threads: response.data, isLoadingThreads: false });
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to load chat history"));
      set({ isLoadingThreads: false });
    }
  },

  createThread: async (title) => {
    const token = useAuthStore.getState().accessToken;
    if (!token) throw new Error("Not authenticated");

    try {
      const thread = await createThreadApi(token, title || "New Session", "nutritionist");
      set((state) => ({
        threads: [thread, ...state.threads],
        activeThreadId: thread.id,
        messages: [],
        steps: [],
        uiElements: [],
        streamedContent: "",
        activeTraceId: null,
        selectedUiElement: null,
        rightSidebarView: "runtime",
      }));
      return thread.id;
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to create conversation"));
      throw err;
    }
  },

  setActiveThreadId: async (id) => {
    set({
      activeThreadId: id,
      messages: [],
      steps: [],
      uiElements: [],
      streamedContent: "",
      activeTraceId: null,
      selectedUiElement: null,
      rightSidebarView: "runtime",
    });

    if (!id) return;

    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    set({ isLoadingMessages: true });
    try {
      const response = await fetchMessagesApi(token, id, 1, 50);
      // Messages return oldest first in pagination normally, or we reverse them if needed.
      // Let's check: Axum page list_messages usually returns paginated.
      // Let's sort messages by created_at just in case to ensure they display sequentially.
      const sortedMessages = [...response.data].sort(
        (a, b) =>
          new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
      );
      set({ messages: sortedMessages, isLoadingMessages: false });
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to load messages"));
      set({ isLoadingMessages: false });
    }
  },

  deleteThread: async (id) => {
    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    try {
      await deleteThreadApi(token, id);
      set((state) => {
        const nextThreads = state.threads.filter((t) => t.id !== id);
        let nextActiveId = state.activeThreadId;
        if (state.activeThreadId === id) {
          nextActiveId = nextThreads.length > 0 ? nextThreads[0].id : null;
        }
        return {
          threads: nextThreads,
          activeThreadId: nextActiveId,
        };
      });

      const { activeThreadId } = get();
      if (activeThreadId) {
        get().setActiveThreadId(activeThreadId);
      } else {
        set({
          messages: [],
          steps: [],
          uiElements: [],
          streamedContent: "",
          activeTraceId: null,
          selectedUiElement: null,
        });
      }
      toast.success("Conversation deleted");
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to delete conversation"));
    }
  },

  sendMessage: async (prompt) => {
    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    let { activeThreadId } = get();
    if (!activeThreadId) {
      try {
        activeThreadId = await get().createThread(prompt.substring(0, 30));
      } catch {
        return;
      }
    }

    set({ isStreaming: true, streamedContent: "", steps: [], uiElements: [] });

    // 1. Post user message to Axum to save it
    try {
      const userMessage = await postMessageApi(token, activeThreadId!, prompt);
      set((state) => ({
        messages: [...state.messages, userMessage],
      }));
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to send message"));
      set({ isStreaming: false });
      return;
    }

    // 2. Trigger intelligence orchestrator
    let orchestrateRes;
    try {
      orchestrateRes = await orchestrateApi(token, activeThreadId!, prompt);
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Intelligence orchestration failed"));
      set({ isStreaming: false });
      return;
    }

    const { trace_id: traceId } = orchestrateRes;
    set({ activeTraceId: traceId });

    // 3. Open SSE readable stream manually to support Bearer token authentication headers
    try {
      const response = await fetch(
        `${API_BASE_URL}/intelligence/stream/${traceId}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      if (!response.ok) {
        throw new Error("Failed to connect to agent execution stream");
      }

      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error("Execution stream is not readable");
      }

      const decoder = new TextDecoder();
      let buffer = "";

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const parts = buffer.split("\n\n");
        buffer = parts.pop() || "";

        for (const part of parts) {
          if (!part.trim()) continue;

          let eventName = "";
          let dataStr = "";
          const lines = part.split("\n");
          for (const line of lines) {
            if (line.startsWith("event:")) {
              eventName = line.substring(6).trim();
            } else if (line.startsWith("data:")) {
              dataStr = line.substring(5).trim();
            }
          }

          if (dataStr) {
            try {
              const data = JSON.parse(dataStr);
              const event = data.event || eventName;
              const payload = data.data || data;

              switch (event) {
                case "trace_start":
                  set({
                    activeTraceId: payload.trace_id,
                    steps: [],
                    uiElements: [],
                    streamedContent: "",
                  });
                  break;

                case "agent_step":
                  set((state) => {
                    const stepPayload = payload;
                    const existingIdx = state.steps.findIndex(
                      (s) => s.agent === stepPayload.agent
                    );
                    const updatedSteps = [...state.steps];
                    if (existingIdx > -1) {
                      updatedSteps[existingIdx] = {
                        agent: stepPayload.agent,
                        status: stepPayload.status,
                        step_type: stepPayload.step_type,
                        latency_ms: stepPayload.latency_ms ?? undefined,
                      };
                    } else {
                      updatedSteps.push({
                        agent: stepPayload.agent,
                        status: stepPayload.status,
                        step_type: stepPayload.step_type,
                        latency_ms: stepPayload.latency_ms ?? undefined,
                      });
                    }
                    return { steps: updatedSteps };
                  });
                  break;

                case "message_delta":
                  set((state) => ({
                    streamedContent: state.streamedContent + payload.content,
                  }));
                  break;

                case "ui_element":
                  set((state) => ({
                    uiElements: [
                      ...state.uiElements,
                      {
                        type: payload.element_type,
                        title: payload.title,
                        config: payload.config_json,
                        data: payload.data_json,
                      },
                    ],
                  }));
                  break;

                case "done":
                  set({ isStreaming: false });
                  // Refetch thread messages to fetch the full stored assistant message with metadata
                  if (activeThreadId) {
                    const messagesResponse = await fetchMessagesApi(
                      token,
                      activeThreadId,
                      1,
                      50
                    );
                    const sorted = [...messagesResponse.data].sort(
                      (a, b) =>
                        new Date(a.created_at).getTime() -
                        new Date(b.created_at).getTime()
                    );
                    set({ messages: sorted });
                    // Refresh active memories
                    get().fetchActiveMemories();
                  }
                  break;

                case "error":
                  toast.error(payload.message || "Agent execution failed");
                  set({ isStreaming: false });
                  break;

                default:
                  break;
              }
            } catch (jsonErr) {
              console.error("Failed to parse SSE JSON line:", jsonErr, dataStr);
            }
          }
        }
      }
    } catch (streamErr: unknown) {
      toast.error(getErrorMessage(streamErr, "Connection to execution stream failed"));
      set({ isStreaming: false });
    }
  },

  fetchActiveMemories: async () => {
    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    try {
      const memories = await fetchActiveMemoriesApi(token);
      set({ memories });
    } catch (err: unknown) {
      console.error("Failed to load user memories:", err);
    }
  },

  deactivateMemory: async (id) => {
    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    try {
      await deactivateMemoryApi(token, id);
      set((state) => ({
        memories: state.memories.filter((m) => m.id !== id),
      }));
      toast.success("Constraint deactivated");
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to deactivate constraint"));
    }
  },

  submitFeedback: async (payload) => {
    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    try {
      await submitFeedbackApi(token, payload);
      toast.success("Feedback submitted. Thank you!");
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to submit feedback"));
    }
  },

  fetchRecipeDetail: async (recipeId) => {
    const token = useAuthStore.getState().accessToken;
    if (!token) return;

    try {
      const recipeDetail = await fetchRecipeDetailApi(token, recipeId);
      set({
        selectedUiElement: {
          type: "recipe",
          title: recipeDetail.recipe.title,
          config: {},
          data: recipeDetail,
        },
        rightSidebarView: "detail",
      });
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Failed to load recipe details"));
    }
  },
}));
