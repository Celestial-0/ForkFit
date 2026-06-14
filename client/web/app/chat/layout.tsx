import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Chat with ForkFit",
  description: "Engage in a conversation with ForkFit, your personal fitness assistant. Ask questions, get workout recommendations, and receive personalized fitness advice to help you achieve your goals.",
};

export default function ChatLayout({ children }: { children: React.ReactNode }) {
  return <>{children}</>;
}
