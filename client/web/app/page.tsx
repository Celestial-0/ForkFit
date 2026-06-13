import { Landing } from "@/components/core/landing"
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "ForkFit",
  description: "",
};

export default function Page() {
  return (
    <div className="flex min-h-svh p-6">
      <Landing />
    </div>
  )
}
