import { API_BASE_URL, handleResponse } from "./api";

export interface ChatThread {
  id: string;
  user_id: string;
  title: string | null;
  agent_type: string;
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  thread_id: string;
  sender_role: "user" | "assistant" | "system";
  content: string;
  metadata: {
    ui_elements?: Array<{
      type: "meal_plan" | "shopping_list" | string;
      title: string;
      config: unknown;
      data: unknown;
    }>;
    [key: string]: unknown;
  };
  created_at: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: {
    page: number;
    per_page: number;
    total: number;
  };
}

export interface AgentMemory {
  id: string;
  user_id: string;
  memory_type: string;
  content: string;
  confidence: number;
  importance: number;
  is_active: boolean;
  metadata: unknown;
  created_at: string;
  updated_at: string;
}

export interface OrchestrateResponse {
  trace_id: string;
  stream_url: string;
  status: string;
}

export interface FeedbackRequest {
  category: "chat_response" | "meal_plan" | "recipe" | string;
  reference_id?: string;
  rating: number;
  comment?: string;
  metadata?: unknown;
}

export interface FeedbackResponse {
  id: string;
  user_id: string;
  category: string;
  reference_id: string | null;
  rating: number;
  comment: string | null;
  metadata: unknown;
  created_at: string;
}

export async function fetchThreadsApi(
  token: string,
  page = 1,
  perPage = 20
): Promise<PaginatedResponse<ChatThread>> {
  const response = await fetch(
    `${API_BASE_URL}/threads?page=${page}&per_page=${perPage}`,
    {
      method: "GET",
      headers: {
        Authorization: `Bearer ${token}`,
      },
    }
  );
  return handleResponse<PaginatedResponse<ChatThread>>(response);
}

export async function createThreadApi(
  token: string,
  title?: string,
  agentType = "nutritionist"
): Promise<ChatThread> {
  const response = await fetch(`${API_BASE_URL}/threads`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ title, agent_type: agentType }),
  });
  return handleResponse<ChatThread>(response);
}

export async function deleteThreadApi(
  token: string,
  id: string
): Promise<{ success: boolean }> {
  const response = await fetch(`${API_BASE_URL}/threads/${id}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  return handleResponse<{ success: boolean }>(response);
}

export async function fetchMessagesApi(
  token: string,
  threadId: string,
  page = 1,
  perPage = 50
): Promise<PaginatedResponse<ChatMessage>> {
  const response = await fetch(
    `${API_BASE_URL}/threads/${threadId}/messages?page=${page}&per_page=${perPage}`,
    {
      method: "GET",
      headers: {
        Authorization: `Bearer ${token}`,
      },
    }
  );
  return handleResponse<PaginatedResponse<ChatMessage>>(response);
}

export async function postMessageApi(
  token: string,
  threadId: string,
  content: string
): Promise<ChatMessage> {
  const response = await fetch(`${API_BASE_URL}/threads/${threadId}/messages`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ content }),
  });
  return handleResponse<ChatMessage>(response);
}

export async function orchestrateApi(
  token: string,
  threadId: string,
  prompt: string
): Promise<OrchestrateResponse> {
  const response = await fetch(`${API_BASE_URL}/intelligence/orchestrate`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ thread_id: threadId, prompt }),
  });
  return handleResponse<OrchestrateResponse>(response);
}

export async function fetchActiveMemoriesApi(
  token: string
): Promise<AgentMemory[]> {
  const response = await fetch(`${API_BASE_URL}/intelligence/memories`, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  return handleResponse<AgentMemory[]>(response);
}

export async function deactivateMemoryApi(
  token: string,
  id: string
): Promise<{ success: boolean }> {
  const response = await fetch(`${API_BASE_URL}/intelligence/memories/${id}`, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  return handleResponse<{ success: boolean }>(response);
}

export async function submitFeedbackApi(
  token: string,
  payload: FeedbackRequest
): Promise<FeedbackResponse> {
  const response = await fetch(`${API_BASE_URL}/feedback`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<FeedbackResponse>(response);
}

export interface RecipeResponse {
  id: string;
  owner_id: string | null;
  parent_recipe_id: string | null;
  title: string;
  description: string | null;
  instructions: string[];
  prep_time_minutes: number | null;
  cook_time_minutes: number | null;
  servings: number;
  cuisine: string | null;
  course: string | null;
  dietary_tags: string[];
  source_url: string | null;
  is_public: boolean;
  created_at: string;
}

export interface RecipeIngredientDetailResponse {
  ingredient_id: string;
  name: string;
  quantity: number;
  unit: string;
  grams_equivalent: number;
  notes: string | null;
}

export interface RecipeNutrients {
  calories: number;
  protein: number;
  carbs: number;
  fat: number;
  fiber: number;
  sodium: number;
}

export interface RecipeDetailResponse {
  recipe: RecipeResponse;
  ingredients: RecipeIngredientDetailResponse[];
  total_nutrition: RecipeNutrients;
  serving_nutrition: RecipeNutrients;
  total_estimated_cost: number;
  serving_estimated_cost: number;
  detected_allergens: string[];
}

export async function fetchRecipeDetailApi(
  token: string,
  id: string
): Promise<RecipeDetailResponse> {
  const response = await fetch(`${API_BASE_URL}/recipes/${id}`, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  return handleResponse<RecipeDetailResponse>(response);
}

