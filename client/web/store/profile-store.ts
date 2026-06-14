import { create } from "zustand";
import { useAuthStore } from "./auth-store";
import {
  getProfileApi,
  updateProfileApi,
  getPreferencesApi,
  updatePreferencesApi,
  getMedicalSafetyApi,
  updateMedicalSafetyApi,
  getBiometricsApi,
  logBiometricApi,
  getWorkoutsApi,
  logWorkoutApi,
  getGoalsApi,
  createGoalApi,
  deactivateGoalApi,
  UpdateProfileRequest,
  UpdatePreferenceRequest,
  UpdateMedicalSafetyRequest,
  CreateBiometricRequest,
  CreateWorkoutRequest,
  CreateGoalRequest,
  UserPreferenceResponse,
  MedicalSafetyResponse,
  BiometricResponse,
  WorkoutResponse,
  GoalResponse,
} from "@/lib/api/profile";
import { User } from "@/lib/api/api";

interface ProfileState {
  profile: User | null;
  preferences: UserPreferenceResponse | null;
  medicalSafety: MedicalSafetyResponse | null;
  biometricLogs: BiometricResponse[];
  workoutLogs: WorkoutResponse[];
  goals: GoalResponse[];
  isLoading: boolean;
  error: string | null;

  fetchProfile: () => Promise<void>;
  updateProfile: (payload: UpdateProfileRequest) => Promise<void>;
  fetchPreferences: () => Promise<void>;
  updatePreferences: (payload: UpdatePreferenceRequest) => Promise<void>;
  fetchMedicalSafety: () => Promise<void>;
  updateMedicalSafety: (payload: UpdateMedicalSafetyRequest) => Promise<void>;
  fetchBiometrics: () => Promise<void>;
  logBiometric: (payload: CreateBiometricRequest) => Promise<void>;
  fetchWorkouts: () => Promise<void>;
  logWorkout: (payload: CreateWorkoutRequest) => Promise<void>;
  fetchGoals: () => Promise<void>;
  createGoal: (payload: CreateGoalRequest) => Promise<void>;
  deactivateGoal: (category: string) => Promise<void>;
  clearProfileError: () => void;
  resetProfileStore: () => void;
}

export const useProfileStore = create<ProfileState>((set, get) => {
  const getTokenOrThrow = () => {
    const token = useAuthStore.getState().accessToken;
    if (!token) {
      throw new Error("No active session. Please sign in again.");
    }
    return token;
  };

  const handleAction = async (
    actionFn: (token: string) => Promise<any>,
    successCallback: (data: any) => void
  ) => {
    set({ isLoading: true, error: null });
    try {
      const token = getTokenOrThrow();
      const data = await actionFn(token);
      successCallback(data);
    } catch (err: any) {
      const msg = err instanceof Error ? err.message : "Request failed";
      set({ error: msg, isLoading: false });
      throw err;
    }
  };

  return {
    profile: null,
    preferences: null,
    medicalSafety: null,
    biometricLogs: [],
    workoutLogs: [],
    goals: [],
    isLoading: false,
    error: null,

    fetchProfile: async () => {
      await handleAction(getProfileApi, (data) => {
        set({ profile: data, isLoading: false });
        useAuthStore.setState({ user: data });
      });
    },

    updateProfile: async (payload) => {
      await handleAction(
        (token) => updateProfileApi(token, payload),
        (data) => {
          set({ profile: data, isLoading: false });
          useAuthStore.setState({ user: data });
        }
      );
    },

    fetchPreferences: async () => {
      await handleAction(getPreferencesApi, (data) => {
        set({ preferences: data, isLoading: false });
      });
    },

    updatePreferences: async (payload) => {
      await handleAction(
        (token) => updatePreferencesApi(token, payload),
        (data) => {
          set({ preferences: data, isLoading: false });
        }
      );
    },

    fetchMedicalSafety: async () => {
      await handleAction(getMedicalSafetyApi, (data) => {
        set({ medicalSafety: data, isLoading: false });
      });
    },

    updateMedicalSafety: async (payload) => {
      await handleAction(
        (token) => updateMedicalSafetyApi(token, payload),
        (data) => {
          set({ medicalSafety: data, isLoading: false });
        }
      );
    },

    fetchBiometrics: async () => {
      await handleAction(getBiometricsApi, (res) => {
        set({ biometricLogs: res.data, isLoading: false });
      });
    },

    logBiometric: async (payload) => {
      await handleAction(
        (token) => logBiometricApi(token, payload),
        (data) => {
          set((state) => ({
            biometricLogs: [data, ...state.biometricLogs],
            isLoading: false,
          }));
        }
      );
    },

    fetchWorkouts: async () => {
      await handleAction(getWorkoutsApi, (res) => {
        set({ workoutLogs: res.data, isLoading: false });
      });
    },

    logWorkout: async (payload) => {
      await handleAction(
        (token) => logWorkoutApi(token, payload),
        (data) => {
          set((state) => ({
            workoutLogs: [data, ...state.workoutLogs],
            isLoading: false,
          }));
        }
      );
    },

    fetchGoals: async () => {
      await handleAction(getGoalsApi, (data) => {
        set({ goals: data, isLoading: false });
      });
    },

    createGoal: async (payload) => {
      await handleAction(
        (token) => createGoalApi(token, payload),
        (data) => {
          set((state) => ({
            goals: [data, ...state.goals],
            isLoading: false,
          }));
        }
      );
    },

    deactivateGoal: async (category) => {
      await handleAction(
        (token) => deactivateGoalApi(token, category),
        () => {
          set((state) => ({
            goals: state.goals.map((g) =>
              g.category === category ? { ...g, is_active: false } : g
            ),
            isLoading: false,
          }));
        }
      );
    },

    clearProfileError: () => set({ error: null }),

    resetProfileStore: () =>
      set({
        profile: null,
        preferences: null,
        medicalSafety: null,
        biometricLogs: [],
        workoutLogs: [],
        goals: [],
        isLoading: false,
        error: null,
      }),
  };
});
