"use client"

import React from "react"
import Link from "next/link"
import { AnimatedGridPattern } from "@/components/ui/animated-grid-pattern"
import { WordRotate } from "@/components/ui/word-rotate"
import { RainbowButton } from "@/components/ui/rainbow-button"
import { Button } from "@/components/ui/button"
import { Sparkles, ArrowRight } from "lucide-react"

export const Hero = () => {
  return (
    <section className="relative w-full min-h-[85vh] flex items-center justify-center overflow-hidden bg-background">
      {/* Background Grid Pattern */}
      <AnimatedGridPattern
        numSquares={30}
        maxOpacity={0.15}
        duration={3}
        repeatDelay={1}
        className="absolute inset-x-0 inset-y-[-30%] h-[150%] w-full skew-y-12 opacity-60 dark:opacity-30 pointer-events-none"
      />

      {/* Radial overlay to wash background lines */}
      <div className="absolute inset-0 bg-radial-gradient from-transparent via-background/60 to-background pointer-events-none" />

      <div className="container mx-auto px-4 max-w-5xl relative z-10 flex flex-col items-center text-center space-y-8 pt-12 pb-16">
        
        {/* Eyebrow / Tag */}
        <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono bg-zinc-100 dark:bg-zinc-900 border border-border/80">
          <Sparkles className="size-3 text-amber-500" />
          <span className="text-muted-foreground">Nutrition OS & RBAC Authorization Engine</span>
        </div>

        {/* H1 Headline (Strictly 2 lines on desktop) */}
        <h1 className="text-4xl md:text-6xl lg:text-7xl font-extrabold tracking-tight font-heading leading-[1.15] text-foreground max-w-4xl mx-auto">
          Nutrition Intelligence,<br />
          Guided by Multi-Agent Reasoning.
        </h1>

        {/* Subtitle / Word Rotate */}
        <div className="flex flex-col sm:flex-row items-center justify-center gap-2 text-muted-foreground text-base md:text-lg max-w-[65ch] font-sans">
          <span>Continuous cognitive loops designed to optimize your</span>
          <div className="font-semibold text-primary font-mono inline-block">
            <WordRotate 
              words={["macros & micros", "allergy safety", "grocery budget", "meal schedule", "active sessions", "granular roles"]} 
              duration={2500} 
              className="text-primary"
            />
          </div>
        </div>

        {/* Action CTAs */}
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 pt-4">
          <RainbowButton asChild size="lg">
            <Link href="/auth">Launch Core Sandbox</Link>
          </RainbowButton>
          <Button 
            size="lg" 
            variant="outline" 
            className="group gap-2 border-border/80 hover:bg-muted font-sans text-foreground"
            render={<a href="#reasoning-loop" />}
            nativeButton={false}
          >
            <span>Explore Architecture</span>
            <ArrowRight className="size-4 text-muted-foreground transition-transform duration-200 group-hover:translate-x-0.5" />
          </Button>
        </div>
      </div>
    </section>
  )
}