import { AuthPage } from "@/components/core/auth";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Join ForkFit - Track, Fuel, Train, Transform",
  description: "Sign in or register for your ForkFit account to manage nutrition plans, workouts, and optimize your fitness goals.",
};

export default function Page() {
  return <AuthPage />;
}
