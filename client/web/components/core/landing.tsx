import { Header } from "@/components/core/landing/header"
import { Hero } from "./landing/hero"
import { Features } from "./landing/features"
import { AgentVisualizer } from "./landing/agent-visualizer"
import { LiveDemo } from "./landing/live-demo"
import { CTA } from "./landing/cta"
import { Footer } from "./landing/footer"

export const Landing = () => {
  return (
    <div className="w-full flex flex-col min-h-screen">
      <Header />
      <Hero />
      <Features />
      <AgentVisualizer />
      <LiveDemo />
      <CTA />
      <Footer />
    </div>
  )
}
