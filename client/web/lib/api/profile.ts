import { API_BASE_URL, handleResponse, User } from "./api";

// API Request/Response Types
export interface UpdateProfileRequest {
  full_name?: string;
  avatar_url?: string;
  gender?: string;
  dob?: string; // Format: "YYYY-MM-DD"
  timezone: string;
}

export interface UpdatePreferenceRequest {
  theme: string;
  language: string;
  measurement_system: string;
  preferences: Record<string, any>;
}

export interface UserPreferenceResponse {
  user_id: string;
  theme: string;
  language: string;
  measurement_system: string;
  preferences: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface UpdateMedicalSafetyRequest {
  allergies: string[];
  medical_conditions: string[];
  is_pregnant: boolean;
  is_lactating: boolean;
}

export interface MedicalSafetyResponse {
  user_id: string;
  allergies: string[];
  medical_conditions: string[];
  is_pregnant: boolean;
  is_lactating: boolean;
  updated_at: string;
}

export interface CreateBiometricRequest {
  metric_type: string; // e.g. "weight", "body_fat_percentage"
  value: number;
  notes?: string;
  logged_at?: string;
}

export interface BiometricResponse {
  id: string;
  user_id: string;
  logged_at: string;
  metric_type: string;
  value: number;
  notes?: string;
  created_at: string;
}

export interface CreateWorkoutRequest {
  activity_name: string;
  duration_minutes: number;
  calories_burned: number;
  notes?: string;
  logged_at?: string;
}

export interface WorkoutResponse {
  id: string;
  user_id: string;
  logged_at: string;
  activity_name: string;
  duration_minutes: number;
  calories_burned: number;
  notes?: string;
  created_at: string;
}

export interface CreateGoalRequest {
  category: string;
  target_type: string;
  target_value: number;
  unit: string;
  config: Record<string, any>;
  start_date?: string; // Format: "YYYY-MM-DD"
  target_date?: string; // Format: "YYYY-MM-DD"
}

export interface GoalResponse {
  id: string;
  user_id: string;
  category: string;
  target_type: string;
  target_value: number;
  unit: string;
  config: Record<string, any>;
  start_date: string;
  target_date?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

// Fetch headers helper
function getHeaders(token: string) {
  return {
    "Content-Type": "application/json",
    Authorization: `Bearer ${token}`,
  };
}

// Profile API methods
export async function getProfileApi(token: string): Promise<User> {
  const response = await fetch(`${API_BASE_URL}/profile`, {
    method: "GET",
    headers: getHeaders(token),
  });
  const data = await handleResponse<any>(response);
  return {
    id: data.user_id,
    email: data.email,
    email_verified: data.email_verified,
    status: data.status,
    full_name: data.full_name || undefined,
    avatar_url: data.avatar_url || undefined,
    gender: data.gender || undefined,
    dob: data.dob || undefined,
    timezone: data.timezone,
    created_at: data.created_at,
    updated_at: data.updated_at,
  };
}

export async function updateProfileApi(
  token: string,
  payload: UpdateProfileRequest
): Promise<User> {
  const response = await fetch(`${API_BASE_URL}/profile`, {
    method: "PUT",
    headers: getHeaders(token),
    body: JSON.stringify(payload),
  });
  const data = await handleResponse<any>(response);
  return {
    id: data.user_id,
    email: data.email,
    email_verified: data.email_verified,
    status: data.status,
    full_name: data.full_name || undefined,
    avatar_url: data.avatar_url || undefined,
    gender: data.gender || undefined,
    dob: data.dob || undefined,
    timezone: data.timezone,
    created_at: data.created_at,
    updated_at: data.updated_at,
  };
}

export async function getPreferencesApi(token: string): Promise<UserPreferenceResponse> {
  const response = await fetch(`${API_BASE_URL}/profile/preferences`, {
    method: "GET",
    headers: getHeaders(token),
  });
  return handleResponse<UserPreferenceResponse>(response);
}

export async function updatePreferencesApi(
  token: string,
  payload: UpdatePreferenceRequest
): Promise<UserPreferenceResponse> {
  const response = await fetch(`${API_BASE_URL}/profile/preferences`, {
    method: "PUT",
    headers: getHeaders(token),
    body: JSON.stringify(payload),
  });
  return handleResponse<UserPreferenceResponse>(response);
}

export async function getMedicalSafetyApi(token: string): Promise<MedicalSafetyResponse> {
  const response = await fetch(`${API_BASE_URL}/profile/safety`, {
    method: "GET",
    headers: getHeaders(token),
  });
  return handleResponse<MedicalSafetyResponse>(response);
}

export async function updateMedicalSafetyApi(
  token: string,
  payload: UpdateMedicalSafetyRequest
): Promise<MedicalSafetyResponse> {
  const response = await fetch(`${API_BASE_URL}/profile/safety`, {
    method: "PUT",
    headers: getHeaders(token),
    body: JSON.stringify(payload),
  });
  return handleResponse<MedicalSafetyResponse>(response);
}

export interface PaginatedResponse<T> {
  data: T[];
  meta: {
    page: number;
    per_page: number;
    total: number;
    total_pages: number;
  };
}

export async function getBiometricsApi(token: string): Promise<PaginatedResponse<BiometricResponse>> {
  const response = await fetch(`${API_BASE_URL}/profile/biometrics`, {
    method: "GET",
    headers: getHeaders(token),
  });
  return handleResponse<PaginatedResponse<BiometricResponse>>(response);
}

export async function logBiometricApi(
  token: string,
  payload: CreateBiometricRequest
): Promise<BiometricResponse> {
  const response = await fetch(`${API_BASE_URL}/profile/biometrics`, {
    method: "POST",
    headers: getHeaders(token),
    body: JSON.stringify(payload),
  });
  return handleResponse<BiometricResponse>(response);
}

export async function getWorkoutsApi(token: string): Promise<PaginatedResponse<WorkoutResponse>> {
  const response = await fetch(`${API_BASE_URL}/profile/workouts`, {
    method: "GET",
    headers: getHeaders(token),
  });
  return handleResponse<PaginatedResponse<WorkoutResponse>>(response);
}

export async function logWorkoutApi(
  token: string,
  payload: CreateWorkoutRequest
): Promise<WorkoutResponse> {
  const response = await fetch(`${API_BASE_URL}/profile/workouts`, {
    method: "POST",
    headers: getHeaders(token),
    body: JSON.stringify(payload),
  });
  return handleResponse<WorkoutResponse>(response);
}

export async function getGoalsApi(token: string): Promise<GoalResponse[]> {
  const response = await fetch(`${API_BASE_URL}/profile/goals`, {
    method: "GET",
    headers: getHeaders(token),
  });
  return handleResponse<GoalResponse[]>(response);
}

export async function createGoalApi(
  token: string,
  payload: CreateGoalRequest
): Promise<GoalResponse> {
  const response = await fetch(`${API_BASE_URL}/profile/goals`, {
    method: "POST",
    headers: getHeaders(token),
    body: JSON.stringify(payload),
  });
  return handleResponse<GoalResponse>(response);
}

export async function deactivateGoalApi(
  token: string,
  category: string
): Promise<{ success: boolean }> {
  const response = await fetch(`${API_BASE_URL}/profile/goals/${category}`, {
    method: "DELETE",
    headers: getHeaders(token),
  });
  return handleResponse<{ success: boolean }>(response);
}
