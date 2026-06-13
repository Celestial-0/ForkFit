"use client"

import Link from "next/link"

import { ForkFit } from "@/components/icons/main"
import { WordRotate } from "@/components/ui/word-rotate"

export function AuthBanner() {
  return (
    <div className="relative hidden w-1/2 flex-col justify-between overflow-hidden border-border/30 bg-radial-[at_50%_40%] from-muted/30 via-background to-card p-12 text-foreground md:flex dark:border-r">
      {/* Subtle grid accent background */}
      <div className="absolute inset-0 bg-[linear-gradient(to_right,var(--color-border)_1px,transparent_1px),linear-gradient(to_bottom,var(--color-border)_1px,transparent_1px)] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_50%,#000_70%,transparent_100%)] bg-[size:4rem_4rem] opacity-30" />

      {/* Logo / Header */}
      <Link
        href="/"
        className="z-10 flex items-center gap-2"
      >
        <ForkFit className="h-6" />
        <span className="font-heading text-xl font-bold tracking-tight text-foreground">
          Fork<span className="text-muted-foreground">Fit</span>
        </span>
      </Link>

      {/* Animated Hero Copy */}
      <div className="z-10 my-auto max-w-lg space-y-6">
        <div className="flex flex-col">
          <span className="text-sm font-bold tracking-wider text-muted-foreground/80 uppercase">
            Next-Gen Fitness
          </span>
          <h1 className="font-heading text-4xl leading-none font-black tracking-tight text-foreground sm:text-5xl">
            REDEFINE
          </h1>
          <WordRotate
            words={[
              "YOUR DIET",
              "YOUR PROGRESS",
              "YOUR WORKOUTS",
              "YOUR HEALTH",
            ]}
            duration={3000}
            className="bg-linear-to-r from-foreground to-muted-foreground bg-clip-text font-heading text-4xl leading-none font-black tracking-tight text-transparent sm:text-5xl"
          />
        </div>
        <p className="max-w-sm text-lg leading-relaxed text-muted-foreground">
          Your ultimate nutrition, custom workout and meal scheduling
          companion. Fuel smarter, train harder.
        </p>
      </div>

      {/* Bottom Footer Details */}
      <div className="z-10 flex flex-row items-center justify-between text-xs text-muted-foreground/80">
        <span>© {new Date().getFullYear()} ForkFit Inc.</span>
        <div className="flex gap-4">
          <a href="#" className="transition-colors hover:text-foreground">
            Security
          </a>
          <a href="#" className="transition-colors hover:text-foreground">
            Privacy
          </a>
        </div>
      </div>
    </div>
  )
}
