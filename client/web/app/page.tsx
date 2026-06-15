import { Landing } from "@/components/core/landing"
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "ForkFit - AI-Powered Nutrition Intelligence & Auth API",
  description: "Enterprise Cognitive Intelligence Architecture with session revocation, rate-limiting, and granular RBAC built on Rust and Next.js.",
};

export default function Page() {
  return (
    <main className="w-full min-h-svh flex flex-col bg-background text-foreground selection:bg-primary/30">
      <Landing />
    </main>
  )
}
