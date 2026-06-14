"use client";

import { useAuthStore } from "@/store/auth-store";
import { useHydratedStore, useIsHydrated } from "@/hooks/use-hydrated-store";
import { ChatContainer } from "@/components/core/chat";
import { useRouter } from "next/navigation";
import { useEffect } from "react";
import { Loader2 } from "lucide-react";



export default function ChatPage() {
  const isHydrated = useIsHydrated();
  const isAuthenticated = useHydratedStore(
    useAuthStore,
    (state) => state.isAuthenticated
  );
  const router = useRouter();

  const fetchMe = useAuthStore((state) => state.fetchMe);

  useEffect(() => {
    if (isHydrated) {
      if (!isAuthenticated) {
        router.push("/auth");
      } else {
        fetchMe();
      }
    }
  }, [isHydrated, isAuthenticated, router, fetchMe]);

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
    // Return empty while redirecting
    return null;
  }

  return <ChatContainer />;
}
