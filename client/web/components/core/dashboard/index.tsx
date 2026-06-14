"use client";

import { useEffect, useState } from "react";
import { useProfileStore } from "@/store/profile-store";
import { useAuthStore } from "@/store/auth-store";
import { useRouter, useSearchParams } from "next/navigation";
import { cn } from "@/lib/utils";
import {
  ArrowLeft,
  User as UserIcon,
  Settings,
  ShieldAlert,
  Target,
  Activity,
} from "lucide-react";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";

// Modular forms
import { PersonalForm } from "./personal-form";
import { PreferencesForm } from "./preferences-form";
import { SafetyForm } from "./safety-form";
import { GoalsForm } from "./goals-form";
import { LogsForm } from "./logs-form";

export function DashboardContainer() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const tabParam = searchParams.get("tab");
  const user = useAuthStore((state) => state.user);

  const {
    profile,
    fetchProfile,
    fetchPreferences,
    fetchMedicalSafety,
    fetchGoals,
    fetchBiometrics,
    fetchWorkouts,
  } = useProfileStore();

  const [activeTab, setActiveTab] = useState("profile");

  useEffect(() => {
    fetchProfile().catch((err) => console.error(err));
    fetchPreferences().catch((err) => console.error(err));
    fetchMedicalSafety().catch((err) => console.error(err));
    fetchGoals().catch((err) => console.error(err));
    fetchBiometrics().catch((err) => console.error(err));
    fetchWorkouts().catch((err) => console.error(err));
  }, []);

  useEffect(() => {
    if (tabParam && ["profile", "preferences", "safety", "goals", "logs"].includes(tabParam)) {
      setActiveTab(tabParam);
    }
  }, [tabParam]);

  const handleTabChange = (tabName: string) => {
    setActiveTab(tabName);
    router.push(`/dashboard?tab=${tabName}`);
  };

  const getInitials = (name?: string, email?: string) => {
    if (name) {
      return name
        .split(" ")
        .map((n) => n[0])
        .join("")
        .toUpperCase()
        .substring(0, 2);
    }
    if (email) {
      return email.substring(0, 2).toUpperCase();
    }
    return "US";
  };

  return (
    <div className="min-h-screen bg-background text-foreground flex flex-col font-sans">
      {/* Top Header Navigation */}
      <header className="h-16 flex items-center justify-between border-b border-border/80 px-6 bg-card shrink-0 select-none">
        <Button
          variant="ghost"
          onClick={() => router.push("/chat")}
          className="gap-2 text-muted-foreground hover:text-foreground cursor-pointer -ml-2 text-xs font-semibold"
        >
          <ArrowLeft className="size-4" />
          <span>Back to Chat</span>
        </Button>
        <span className="font-semibold text-xs tracking-wider uppercase text-muted-foreground/80">Dashboard</span>
        <div className="w-24" />
      </header>

      {/* Main Settings Panel */}
      <main className="flex-1 max-w-5xl w-full mx-auto py-10 px-6 overflow-y-auto">
        <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 border-b pb-6 mb-8 border-border/60">
          <div className="flex items-center gap-4">
            <Avatar className="size-16 border bg-secondary shrink-0">
              <AvatarImage
                src={user?.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${encodeURIComponent(user?.full_name || user?.email || "user")}`}
                alt={user?.full_name || "User Avatar"}
              />
              <AvatarFallback className="text-lg font-bold">
                {getInitials(user?.full_name, user?.email)}
              </AvatarFallback>
            </Avatar>
            <div className="min-w-0">
              <h1 className="text-2xl font-bold tracking-tight">
                {profile?.full_name || user?.email?.split("@")[0] || "User Profile"}
              </h1>
              <p className="text-xs text-muted-foreground truncate">{user?.email}</p>
            </div>
          </div>
          <div className="text-xs text-muted-foreground/75 font-mono select-none">
            Member since: {profile?.created_at ? new Date(profile.created_at).toLocaleDateString() : "Unknown"}
          </div>
        </div>

        {/* Tab Coordinator */}
        <div className="flex flex-col md:flex-row gap-8 items-start">
          {/* Navigation Sidebar/Top Bar */}
          <div className="flex flex-row md:flex-col items-stretch justify-start bg-transparent border-b md:border-b-0 md:border-r border-border/60 rounded-none h-auto md:w-56 p-0 gap-1 overflow-x-auto md:overflow-x-visible shrink-0 pb-2 md:pb-0 w-full md:pr-4 select-none">
            <button
              onClick={() => handleTabChange("profile")}
              className={cn(
                "flex items-center justify-start gap-3 px-3 py-2.5 text-xs font-semibold rounded-lg hover:bg-muted/50 text-left cursor-pointer transition-all shrink-0 md:w-full",
                activeTab === "profile"
                  ? "bg-muted text-foreground"
                  : "text-muted-foreground"
              )}
            >
              <UserIcon className="size-4 shrink-0" />
              <span>Personal Details</span>
            </button>
            <button
              onClick={() => handleTabChange("preferences")}
              className={cn(
                "flex items-center justify-start gap-3 px-3 py-2.5 text-xs font-semibold rounded-lg hover:bg-muted/50 text-left cursor-pointer transition-all shrink-0 md:w-full",
                activeTab === "preferences"
                  ? "bg-muted text-foreground"
                  : "text-muted-foreground"
              )}
            >
              <Settings className="size-4 shrink-0" />
              <span>App Preferences</span>
            </button>
            <button
              onClick={() => handleTabChange("safety")}
              className={cn(
                "flex items-center justify-start gap-3 px-3 py-2.5 text-xs font-semibold rounded-lg hover:bg-muted/50 text-left cursor-pointer transition-all shrink-0 md:w-full",
                activeTab === "safety"
                  ? "bg-muted text-foreground"
                  : "text-muted-foreground"
              )}
            >
              <ShieldAlert className="size-4 shrink-0" />
              <span>Health & Safety</span>
            </button>
            <button
              onClick={() => handleTabChange("goals")}
              className={cn(
                "flex items-center justify-start gap-3 px-3 py-2.5 text-xs font-semibold rounded-lg hover:bg-muted/50 text-left cursor-pointer transition-all shrink-0 md:w-full",
                activeTab === "goals"
                  ? "bg-muted text-foreground"
                  : "text-muted-foreground"
              )}
            >
              <Target className="size-4 shrink-0" />
              <span>Fitness Goals</span>
            </button>
            <button
              onClick={() => handleTabChange("logs")}
              className={cn(
                "flex items-center justify-start gap-3 px-3 py-2.5 text-xs font-semibold rounded-lg hover:bg-muted/50 text-left cursor-pointer transition-all shrink-0 md:w-full",
                activeTab === "logs"
                  ? "bg-muted text-foreground"
                  : "text-muted-foreground"
              )}
            >
              <Activity className="size-4 shrink-0" />
              <span>Activity & Logs</span>
            </button>
          </div>

          {/* Active Tab Panel */}
          <div className="flex-1 min-w-0 w-full">
            {activeTab === "profile" && <PersonalForm />}
            {activeTab === "preferences" && <PreferencesForm />}
            {activeTab === "safety" && <SafetyForm />}
            {activeTab === "goals" && <GoalsForm />}
            {activeTab === "logs" && <LogsForm />}
          </div>
        </div>
      </main>
    </div>
  );
}
