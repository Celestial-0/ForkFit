import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Dashboard - ForkFit",
  description: "Your personalized fitness dashboard to track your progress, set goals, and stay motivated on your fitness journey.",
};

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}
