"use client"

import React from "react"
import { BentoGrid, BentoCard } from "@/components/ui/bento-grid"
import { ShieldAlert, Key, Calendar, Coins, Sparkles } from "lucide-react"

export function Features() {
  return (
    <section className="relative w-full py-24 md:py-32 bg-background">
      <div className="container mx-auto px-4 max-w-5xl relative z-10">
        
        {/* Section Title */}
        <div className="text-center max-w-3xl mx-auto mb-16">
          <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono bg-emerald-500/10 text-emerald-600 border border-emerald-500/20 mb-4 dark:text-emerald-400">
            <Sparkles className="size-3" />
            <span>Platform Core Capabilities</span>
          </div>
          <h2 className="text-3xl md:text-5xl font-extrabold tracking-tight font-heading mb-4 text-foreground">
            Smarter Core Features
          </h2>
          <p className="text-muted-foreground text-sm md:text-base max-w-[60ch] mx-auto font-sans">
            Engineered for high performance, maximum security, and sub-second execution speeds. 
            Discover the blocks powering ForkFit.
          </p>
        </div>

        {/* Bento Grid */}
        <BentoGrid className="grid grid-cols-1 md:grid-cols-3 gap-6 auto-rows-[20rem] md:auto-rows-[22rem]">
          
          {/* Card 1: Safety Agent */}
          <BentoCard
            name="Safety Guard Agent"
            className="md:col-span-2"
            Icon={ShieldAlert}
            description="Continuous safety audits protecting against food allergies, medical counter-indications, and diet-specific restriction violations in real-time."
            href="/docs"
            cta="Explore Safety Model"
            background={
              <div className="absolute inset-0 opacity-40 dark:opacity-20 pointer-events-none">
                <div className="absolute inset-0 bg-[radial-gradient(#ef4444_1px,transparent_1px)] [background-size:20px_20px] [mask-image:radial-gradient(ellipse_at_center,transparent_30%,black)]" />
                <div className="absolute right-12 bottom-12 size-40 rounded-full bg-red-500/10 blur-3xl" />
              </div>
            }
          />

          {/* Card 2: Granular RBAC & OAuth */}
          <BentoCard
            name="Granular Auth & RBAC"
            className="md:col-span-1"
            Icon={Key}
            description="Built-in OAuth 2.0 (Google/GitHub), robust token authentication, rate limits, and customizable Role-Based Access Control policies."
            href="/docs"
            cta="Inspect Security API"
            background={
              <div className="absolute inset-0 opacity-30 dark:opacity-25 pointer-events-none">
                <div className="absolute inset-0 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px] [mask-image:radial-gradient(ellipse_at_center,transparent_20%,black)]" />
                <div className="absolute right-4 bottom-4 size-32 rounded-full bg-blue-500/10 blur-2xl" />
              </div>
            }
          />

          {/* Card 3: Calendar & Workouts */}
          <BentoCard
            name="Calendar Synchronizer"
            className="md:col-span-1"
            Icon={Calendar}
            description="Align meal frequencies, nutrition timings, and hydration plans directly alongside workout regimes and sleep schedules."
            href="/docs"
            cta="Sync Workouts"
            background={
              <div className="absolute inset-0 opacity-30 dark:opacity-25 pointer-events-none">
                <div className="absolute inset-0 bg-[radial-gradient(#3b82f6_1.2px,transparent_1.2px)] [background-size:18px_18px] [mask-image:radial-gradient(ellipse_at_center,transparent_20%,black)]" />
                <div className="absolute right-4 bottom-4 size-32 rounded-full bg-indigo-500/10 blur-2xl" />
              </div>
            }
          />

          {/* Card 4: Smart Budget Optimizer */}
          <BentoCard
            name="Grocery Budget Costing"
            className="md:col-span-2"
            Icon={Coins}
            description="Calculate optimal weekly shopping lists that match your target nutrition. Swaps ingredients based on real-time pantry inventory and spending bounds."
            href="/docs"
            cta="Analyze Pricing Logic"
            background={
              <div className="absolute inset-0 opacity-40 dark:opacity-20 pointer-events-none">
                <div className="absolute inset-0 bg-[radial-gradient(#10b981_1px,transparent_1px)] [background-size:16px_16px] [mask-image:radial-gradient(ellipse_at_center,transparent_30%,black)]" />
                <div className="absolute right-12 bottom-12 size-40 rounded-full bg-emerald-500/10 blur-3xl" />
              </div>
            }
          />

        </BentoGrid>
      </div>
    </section>
  )
}
