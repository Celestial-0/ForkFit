import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import {
  signUpApi,
  signInApi,
  signOutApi,
  fetchMeApi,
  User,
  SigninRequest,
  SignupRequest,
} from "@/lib/api/api";

interface AuthState {
  user: User | null;
  accessToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  signIn: (payload: SigninRequest) => Promise<void>;
  signup: (payload: SignupRequest) => Promise<void>;
  logout: () => Promise<void>;
  fetchMe: () => Promise<void>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      accessToken: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      signIn: async (payload) => {
        set({ isLoading: true, error: null });
        try {
          const data = await signInApi(payload);
          set({
            user: data.user,
            accessToken: data.access_token,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (err: any) {
          const msg = err instanceof Error ? err.message : "Failed to sign in";
          set({ error: msg, isLoading: false });
          throw err;
        }
      },

      signup: async (payload) => {
        set({ isLoading: true, error: null });
        try {
          const data = await signUpApi(payload);
          set({
            user: data.user,
            accessToken: data.access_token,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch (err: any) {
          const msg = err instanceof Error ? err.message : "Failed to sign up";
          set({ error: msg, isLoading: false });
          throw err;
        }
      },

      logout: async () => {
        const { accessToken } = get();
        set({ isLoading: true });
        if (accessToken) {
          try {
            await signOutApi(accessToken);
          } catch (err) {
            console.error("Failed to sign out on server:", err);
          }
        }
        // Dynamically import and reset profile store to avoid circular dependency
        try {
          const { useProfileStore } = await import("./profile-store");
          useProfileStore.getState().resetProfileStore();
        } catch (err) {
          console.error("Failed to reset profile store:", err);
        }
        set({
          user: null,
          accessToken: null,
          isAuthenticated: false,
          error: null,
          isLoading: false,
        });
      },

      fetchMe: async () => {
        const { accessToken } = get();
        if (!accessToken) return;
        set({ isLoading: true, error: null });
        try {
          const data = await fetchMeApi(accessToken);
          set({ user: data, isAuthenticated: true, isLoading: false });
        } catch (err: any) {
          // If token verification fails, clear local auth credentials
          set({
            user: null,
            accessToken: null,
            isAuthenticated: false,
            isLoading: false,
          });
        }
      },

      clearError: () => set({ error: null }),
    }),
    {
      name: "forkfit-auth",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        user: state.user,
        accessToken: state.accessToken,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);
