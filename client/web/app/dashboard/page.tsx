"use client";

import { useAuthStore } from "@/store/auth-store";
import { useHydratedStore, useIsHydrated } from "@/hooks/use-hydrated-store";
import { DashboardContainer } from "@/components/core/dashboard";
import { useRouter } from "next/navigation";
import { useEffect, Suspense } from "react";
import { Loader2 } from "lucide-react";



export default function DashboardPage() {
  const isHydrated = useIsHydrated();
  const isAuthenticated = useHydratedStore(
    useAuthStore,
    (state) => state.isAuthenticated
  );
  const router = useRouter();

  useEffect(() => {
    if (isHydrated && !isAuthenticated) {
      router.push("/auth");
    }
  }, [isHydrated, isAuthenticated, router]);

  if (!isHydrated) {
    return (
      <div className="fixed inset-0 flex items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="size-8 text-primary animate-spin" />
          <p className="text-xs text-muted-foreground font-semibold">
            Connecting securely...
          </p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return null;
  }

  return (
    <Suspense
      fallback={
        <div className="fixed inset-0 flex items-center justify-center bg-background">
          <Loader2 className="size-8 text-primary animate-spin" />
        </div>
      }
    >
      <DashboardContainer />
    </Suspense>
  );
}
