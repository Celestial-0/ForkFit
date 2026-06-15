"use client"

import React from "react"
import { useRouter } from "next/navigation"
import { BorderBeam } from "@/components/ui/border-beam"
import { InteractiveHoverButton } from "@/components/ui/interactive-hover-button"
import { ShieldCheck, Database, Key, Terminal } from "lucide-react"

export const CTA = () => {
  const router = useRouter()
  return (
    <section className="relative w-full py-24 md:py-32 overflow-hidden bg-background">
      {/* Background radial highlight */}
      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-96 rounded-full bg-primary/10 blur-3xl pointer-events-none" />

      <div className="container mx-auto px-4 max-w-5xl relative z-10">
        
        {/* BorderBeam Card Container */}
        <div className="relative overflow-hidden rounded-2xl border border-border/80 bg-zinc-50/50 dark:bg-zinc-950/40 p-8 md:p-16 shadow-lg backdrop-blur-sm">
          
          {/* BorderBeam for premium neon lighting effect */}
          <BorderBeam 
            size={250} 
            duration={8} 
            colorFrom="var(--color-primary)" 
            colorTo="var(--color-ring)" 
          />

          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-center relative z-20">
            
            {/* Left Column: Context Copy */}
            <div className="lg:col-span-8 space-y-6">
              <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono bg-primary/10 text-primary border border-primary/20">
                <Terminal className="size-3" />
                <span>Quick Integration</span>
              </div>

              <h2 className="text-3xl md:text-5xl font-extrabold tracking-tight font-heading leading-tight text-foreground">
                Deploy Your Nutrition Intelligence Instance
              </h2>

              <p className="text-muted-foreground text-sm md:text-base font-sans max-w-[55ch]">
                Set up your database urls, run automated migrations, and connect Google or GitHub OAuth integrations in under five minutes.
              </p>

              {/* Specs Grid */}
              <div className="grid grid-cols-2 gap-4 pt-2">
                <div className="flex items-center gap-2">
                  <ShieldCheck className="size-4 text-emerald-500 shrink-0" />
                  <span className="text-xs font-mono text-muted-foreground">Allergy Safety Audited</span>
                </div>
                <div className="flex items-center gap-2">
                  <Database className="size-4 text-blue-500 shrink-0" />
                  <span className="text-xs font-mono text-muted-foreground">pgvector Search-Ready</span>
                </div>
                <div className="flex items-center gap-2">
                  <Key className="size-4 text-amber-500 shrink-0" />
                  <span className="text-xs font-mono text-muted-foreground">Granular RBAC Mapped</span>
                </div>
                <div className="flex items-center gap-2">
                  <Terminal className="size-4 text-purple-500 shrink-0" />
                  <span className="text-xs font-mono text-muted-foreground">Active Session Control</span>
                </div>
              </div>
            </div>

            {/* Right Column: CTA Trigger */}
            <div className="lg:col-span-4 flex justify-center lg:justify-end">
              <InteractiveHoverButton 
                className="font-sans text-sm font-semibold border-border/80"
                onClick={() => router.push("/auth")}
              >
                Get Started Now
              </InteractiveHoverButton>
            </div>

          </div>
        </div>

      </div>
    </section>
  )
}
