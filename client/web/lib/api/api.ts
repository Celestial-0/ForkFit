export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:4000/api/v1";

// Unified User type containing account credentials and profile attributes
export interface User {
  id: string;
  email: string;
  email_verified: boolean;
  status: string;
  full_name?: string;
  avatar_url?: string;
  gender?: string;
  dob?: string;
  timezone?: string;
  created_at: string;
  updated_at: string;
}

export interface AuthResponse {
  access_token: string;
  token_type: string;
  expires_at: string;
  user: User;
}

export interface SignupRequest {
  email: string;
  password: string;
  full_name?: string;
}

export interface SigninRequest {
  email: string;
  password: string;
  device_name?: string;
}

// Error interfaces mapping to the Rust ErrorBody
interface ErrorMessage {
  code: string;
  message: string;
}

interface ErrorBody {
  error: ErrorMessage;
}

// Helper to handle response and parse error body
export async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let errorMessage = "An unexpected error occurred.";
    try {
      const errData = (await response.json()) as ErrorBody;
      if (errData?.error?.message) {
        errorMessage = errData.error.message;
      }
    } catch {
      errorMessage = response.statusText || errorMessage;
    }
    throw new Error(errorMessage);
  }
  return response.json() as Promise<T>;
}

// API functions
export async function signUpApi(payload: SignupRequest): Promise<AuthResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/signup`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<AuthResponse>(response);
}

export async function signInApi(payload: SigninRequest): Promise<AuthResponse> {
  const response = await fetch(`${API_BASE_URL}/auth/signin`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<AuthResponse>(response);
}

export async function signOutApi(token: string): Promise<{ signed_out: boolean }> {
  const response = await fetch(`${API_BASE_URL}/auth/signout`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
  });
  return handleResponse<{ signed_out: boolean }>(response);
}

export async function fetchMeApi(token: string): Promise<User> {
  const response = await fetch(`${API_BASE_URL}/profile`, {
    method: "GET",
    headers: {
      Authorization: `Bearer ${token}`,
    },
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

export async function sendVerificationOtpApi(payload: { email: string }): Promise<{ sent: boolean }> {
  const response = await fetch(`${API_BASE_URL}/auth/send-verification-otp`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<{ sent: boolean }>(response);
}

export async function verifyEmailApi(payload: { email: string; otp: string }): Promise<{ verified: boolean }> {
  const response = await fetch(`${API_BASE_URL}/auth/verify-email`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<{ verified: boolean }>(response);
}

export async function forgotPasswordApi(payload: { email: string }): Promise<{ sent: boolean }> {
  const response = await fetch(`${API_BASE_URL}/auth/forgot-password`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<{ sent: boolean }>(response);
}

export async function resetPasswordApi(payload: {
  email: string;
  otp: string;
  new_password: string;
}): Promise<{ reset: boolean }> {
  const response = await fetch(`${API_BASE_URL}/auth/reset-password`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return handleResponse<{ reset: boolean }>(response);
}
