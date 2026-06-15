"use client"

import React, { useRef } from "react"
import { AnimatedBeam } from "@/components/ui/animated-beam"
import { 
  User, 
  Cpu, 
  ShieldAlert, 
  Apple, 
  Coins, 
  Globe, 
  Utensils, 
  Calendar, 
  ShoppingCart, 
  GitMerge, 
  CheckCircle2, 
  Sparkles, 
  FileText
} from "lucide-react"

export function AgentVisualizer() {
  const containerRef = useRef<HTMLDivElement>(null)
  
  // Node Refs
  const userRef = useRef<HTMLDivElement>(null)
  const plannerRef = useRef<HTMLDivElement>(null)
  
  const safetyRef = useRef<HTMLDivElement>(null)
  const nutritionRef = useRef<HTMLDivElement>(null)
  const budgetRef = useRef<HTMLDivElement>(null)
  const cultureRef = useRef<HTMLDivElement>(null)
  const recipeRef = useRef<HTMLDivElement>(null)
  const calendarRef = useRef<HTMLDivElement>(null)
  const shoppingRef = useRef<HTMLDivElement>(null)
  
  const consensusRef = useRef<HTMLDivElement>(null)
  const judgeRef = useRef<HTMLDivElement>(null)
  const vizRef = useRef<HTMLDivElement>(null)
  const outputRef = useRef<HTMLDivElement>(null)

  return (
    <section id="reasoning-loop" className="relative w-full py-24 md:py-32 bg-zinc-50/50 dark:bg-zinc-950/20 border-y border-border/40 overflow-hidden">
      {/* Background patterns */}
      <div className="absolute inset-0 bg-[radial-gradient(#e4e4e7_1px,transparent_1px)] dark:bg-[radial-gradient(#18181b_1px,transparent_1px)] [background-size:16px_16px] [mask-image:radial-gradient(ellipse_at_center,transparent_20%,black)] opacity-60 pointer-events-none" />
      
      <div className="container mx-auto px-4 max-w-6xl relative z-10">
        <div className="text-center max-w-3xl mx-auto mb-16">
          <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono bg-primary/10 text-primary border border-primary/20 mb-4">
            <Sparkles className="size-3 animate-pulse" />
            <span>LangGraph Orchestration</span>
          </div>
          <h2 className="text-3xl md:text-5xl font-extrabold tracking-tight font-heading mb-4 text-foreground">
            LangGraph Cognitive State Machine
          </h2>
          <p className="text-muted-foreground text-sm md:text-base max-w-[65ch] mx-auto font-sans leading-relaxed">
            ForkFit runs a cyclic state graph that routes tasks across specialized agents in parallel. 
            The Judge audits outputs, triggering dynamic replanning loops on constraint violations.
          </p>
        </div>

        {/* Graph Container (Desktop Viewport) */}
        <div className="hidden lg:block">
          <div 
            ref={containerRef}
            className="relative flex w-full h-[650px] items-center justify-between rounded-xl border border-border bg-background/50 p-6 xl:p-12 shadow-sm backdrop-blur-sm"
          >
            {/* Column 1: Entry */}
            <div className="flex flex-col justify-center items-center h-full z-20">
              <div className="flex flex-col items-center gap-3">
                <div 
                  ref={userRef}
                  className="flex size-12 items-center justify-center rounded-full border border-slate-500/20 bg-background hover:border-slate-500 hover:shadow-[0_0_15px_rgba(100,116,139,0.3)] transition-all duration-300 cursor-pointer"
                >
                  <User className="size-5 text-muted-foreground" />
                </div>
                <div className="text-center">
                  <p className="text-xs font-mono font-bold text-foreground">User Entry</p>
                  <p className="text-[9px] text-muted-foreground font-sans">Prompt Input</p>
                </div>
              </div>
            </div>

            {/* Column 2: Planner */}
            <div className="flex flex-col justify-center items-center h-full z-20">
              <div className="flex flex-col items-center gap-3">
                <div 
                  ref={plannerRef}
                  className="flex size-14 items-center justify-center rounded-full border-2 border-violet-500/30 bg-violet-500/5 hover:border-violet-500 hover:shadow-[0_0_15px_rgba(139,92,246,0.3)] transition-all duration-300 cursor-pointer"
                >
                  <Cpu className="size-6 text-violet-500 animate-pulse" />
                </div>
                <div className="text-center">
                  <p className="text-xs font-mono font-bold text-violet-500">Cognitive Planner</p>
                  <p className="text-[9px] text-muted-foreground font-sans">State Router</p>
                </div>
              </div>
            </div>

            {/* Column 3: Parallel Specialist Agents (Stacked Vertically to avoid crossing lines) */}
            <div className="flex flex-col justify-center gap-y-3 h-full z-20 py-4 xl:w-44">
              
              {/* Safety Agent */}
              <div className="flex items-center gap-3 justify-start w-full">
                <div 
                  ref={safetyRef}
                  className="flex size-10 items-center justify-center rounded-full border border-red-500/20 bg-background hover:border-red-500 hover:shadow-[0_0_15px_rgba(239,68,68,0.3)] transition-all duration-300 cursor-pointer shrink-0"
                >
                  <ShieldAlert className="size-5 text-red-500" />
                </div>
                <div className="text-left hidden xl:block">
                  <p className="text-[11px] font-bold text-foreground leading-tight">Safety</p>
                  <p className="text-[9px] text-muted-foreground">Allergy Guard</p>
                </div>
              </div>

              {/* Nutrition Agent */}
              <div className="flex items-center gap-3 justify-start w-full">
                <div 
                  ref={nutritionRef}
                  className="flex size-10 items-center justify-center rounded-full border border-emerald-500/20 bg-background hover:border-emerald-500 hover:shadow-[0_0_15px_rgba(16,185,129,0.3)] transition-all duration-300 cursor-pointer shrink-0"
                >
                  <Apple className="size-5 text-emerald-500" />
                </div>
                <div className="text-left hidden xl:block">
                  <p className="text-[11px] font-bold text-foreground leading-tight">Nutrition</p>
                  <p className="text-[9px] text-muted-foreground">Macro Mapping</p>
                </div>
              </div>

              {/* Budget Agent */}
              <div className="flex items-center gap-3 justify-start w-full">
                <div 
                  ref={budgetRef}
                  className="flex size-10 items-center justify-center rounded-full border border-amber-500/20 bg-background hover:border-amber-500 hover:shadow-[0_0_15px_rgba(245,158,11,0.3)] transition-all duration-300 cursor-pointer shrink-0"
                >
                  <Coins className="size-5 text-amber-500" />
                </div>
                <div className="text-left hidden xl:block">
                  <p className="text-[11px] font-bold text-foreground leading-tight">Budget</p>
                  <p className="text-[9px] text-muted-foreground">Cost Optimizer</p>
                </div>
              </div>

              {/* Culture Agent */}
              <div className="flex items-center gap-3 justify-start w-full">
                <div 
                  ref={cultureRef}
                  className="flex size-10 items-center justify-center rounded-full border border-blue-500/20 bg-background hover:border-blue-500 hover:shadow-[0_0_15px_rgba(59,130,246,0.3)] transition-all duration-300 cursor-pointer shrink-0"
                >
                  <Globe className="size-5 text-blue-500" />
                </div>
                <div className="text-left hidden xl:block">
                  <p className="text-[11px] font-bold text-foreground leading-tight">Culture</p>
                  <p className="text-[9px] text-muted-foreground">Dietary Fit</p>
                </div>
              </div>

              {/* Recipe Agent */}
              <div className="flex items-center gap-3 justify-start w-full">
                <div 
                  ref={recipeRef}
                  className="flex size-10 items-center justify-center rounded-full border border-purple-500/20 bg-background hover:border-purple-500 hover:shadow-[0_0_15px_rgba(168,85,247,0.3)] transition-all duration-300 cursor-pointer shrink-0"
                >
                  <Utensils className="size-5 text-purple-500" />
                </div>
                <div className="text-left hidden xl:block">
                  <p className="text-[11px] font-bold text-foreground leading-tight">Recipe</p>
                  <p className="text-[9px] text-muted-foreground">Meal Prep</p>
                </div>
              </div>

              {/* Calendar Agent */}
              <div className="flex items-center gap-3 justify-start w-full">
                <div 
                  ref={calendarRef}
                  className="flex size-10 items-center justify-center rounded-full border border-indigo-500/20 bg-background hover:border-indigo-500 hover:shadow-[0_0_15px_rgba(99,102,241,0.3)] transition-all duration-300 cursor-pointer shrink-0"
                >
                  <Calendar className="size-5 text-indigo-500" />
                </div>
                <div className="text-left hidden xl:block">
                  <p className="text-[11px] font-bold text-foreground leading-tight">Calendar</p>
                  <p className="text-[9px] text-muted-foreground">Schedule Align</p>
                </div>
              </div>

              {/* Shopping Agent */}
              <div className="flex items-center gap-3 justify-start w-full">
                <div 
                  ref={shoppingRef}
                  className="flex size-10 items-center justify-center rounded-full border border-orange-500/20 bg-background hover:border-orange-500 hover:shadow-[0_0_15px_rgba(249,115,22,0.3)] transition-all duration-300 cursor-pointer shrink-0"
                >
                  <ShoppingCart className="size-5 text-orange-500" />
                </div>
                <div className="text-left hidden xl:block">
                  <p className="text-[11px] font-bold text-foreground leading-tight">Shopping</p>
                  <p className="text-[9px] text-muted-foreground">Pantry & Cart</p>
                </div>
              </div>

            </div>

            {/* Column 4: Consensus */}
            <div className="flex flex-col justify-center items-center h-full z-20">
              <div className="flex flex-col items-center gap-3">
                <div 
                  ref={consensusRef}
                  className="flex size-12 items-center justify-center rounded-full border border-slate-500/20 bg-background hover:border-slate-500 hover:shadow-[0_0_15px_rgba(100,116,139,0.3)] transition-all duration-300 cursor-pointer"
                >
                  <GitMerge className="size-5 text-slate-500" />
                </div>
                <div className="text-center">
                  <p className="text-xs font-mono font-bold text-foreground">Consensus</p>
                  <p className="text-[9px] text-muted-foreground font-sans">Plan Merger</p>
                </div>
              </div>
            </div>

            {/* Column 5: Judge */}
            <div className="flex flex-col justify-center items-center h-full z-20">
              <div className="flex flex-col items-center gap-3">
                <div 
                  ref={judgeRef}
                  className="flex size-14 items-center justify-center rounded-full border-2 border-indigo-500/30 bg-indigo-500/5 hover:border-indigo-500 hover:shadow-[0_0_15px_rgba(99,102,241,0.3)] transition-all duration-300 cursor-pointer"
                >
                  <CheckCircle2 className="size-6 text-indigo-500" />
                </div>
                <div className="text-center">
                  <p className="text-xs font-mono font-bold text-indigo-500">Judge Layer</p>
                  <p className="text-[9px] text-muted-foreground font-sans">Policy Audit</p>
                </div>
              </div>
            </div>

            {/* Column 6: Output / UI */}
            <div className="flex flex-col justify-center gap-y-16 h-full z-20">
              {/* UI Composer */}
              <div className="flex flex-col items-center gap-3">
                <div 
                  ref={vizRef}
                  className="flex size-12 items-center justify-center rounded-full border border-violet-500/20 bg-background hover:border-violet-500 hover:shadow-[0_0_15px_rgba(139,92,246,0.3)] transition-all duration-300 cursor-pointer"
                >
                  <Sparkles className="size-5 text-violet-500" />
                </div>
                <div className="text-center">
                  <p className="text-xs font-mono font-bold text-violet-500">UI Composer</p>
                  <p className="text-[9px] text-muted-foreground font-sans">Viz Generator</p>
                </div>
              </div>

              {/* Structured OS */}
              <div className="flex flex-col items-center gap-3">
                <div 
                  ref={outputRef}
                  className="flex size-12 items-center justify-center rounded-full border border-emerald-500/20 bg-background hover:border-emerald-500 hover:shadow-[0_0_15px_rgba(16,185,129,0.3)] transition-all duration-300 cursor-pointer"
                >
                  <FileText className="size-5 text-emerald-500" />
                </div>
                <div className="text-center">
                  <p className="text-xs font-mono font-bold text-emerald-500">Structured OS</p>
                  <p className="text-[9px] text-muted-foreground font-sans">Final Payload</p>
                </div>
              </div>
            </div>

            {/* --- Animated Beams (Connecting adjacent columns to avoid any line crossing) --- */}
            
            {/* User Entry -> Cognitive Planner */}
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={userRef} 
              toRef={plannerRef} 
              duration={3.5}
              gradientStartColor="var(--color-ring)"
              gradientStopColor="var(--color-primary)"
              pathColor="var(--color-border)"
            />

            {/* Planner -> 7 Specialists (Fan-out) */}
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={plannerRef} 
              toRef={safetyRef} 
              curvature={-45}
              duration={4}
              gradientStartColor="var(--color-primary)"
              gradientStopColor="#ef4444"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={plannerRef} 
              toRef={nutritionRef} 
              curvature={-20}
              duration={4.2}
              gradientStartColor="var(--color-primary)"
              gradientStopColor="#10b981"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={plannerRef} 
              toRef={budgetRef} 
              curvature={-5}
              duration={4.4}
              gradientStartColor="var(--color-primary)"
              gradientStopColor="#f59e0b"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={plannerRef} 
              toRef={cultureRef} 
              curvature={10}
              duration={4.6}
              gradientStartColor="var(--color-primary)"
              gradientStopColor="#3b82f6"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={plannerRef} 
              toRef={recipeRef} 
              curvature={25}
              duration={4.8}
              gradientStartColor="var(--color-primary)"
              gradientStopColor="#a855f7"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={plannerRef} 
              toRef={calendarRef} 
              curvature={40}
              duration={5}
              gradientStartColor="var(--color-primary)"
              gradientStopColor="#6366f1"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={plannerRef} 
              toRef={shoppingRef} 
              curvature={55}
              duration={5.2}
              gradientStartColor="var(--color-primary)"
              gradientStopColor="#f97316"
            />

            {/* 7 Specialists -> Consensus (Fan-in) */}
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={safetyRef} 
              toRef={consensusRef} 
              curvature={-45}
              duration={4}
              gradientStartColor="#ef4444"
              gradientStopColor="var(--color-foreground)"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={nutritionRef} 
              toRef={consensusRef} 
              curvature={-20}
              duration={4.2}
              gradientStartColor="#10b981"
              gradientStopColor="var(--color-foreground)"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={budgetRef} 
              toRef={consensusRef} 
              curvature={-5}
              duration={4.4}
              gradientStartColor="#f59e0b"
              gradientStopColor="var(--color-foreground)"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={cultureRef} 
              toRef={consensusRef} 
              curvature={10}
              duration={4.6}
              gradientStartColor="#3b82f6"
              gradientStopColor="var(--color-foreground)"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={recipeRef} 
              toRef={consensusRef} 
              curvature={25}
              duration={4.8}
              gradientStartColor="#a855f7"
              gradientStopColor="var(--color-foreground)"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={calendarRef} 
              toRef={consensusRef} 
              curvature={40}
              duration={5}
              gradientStartColor="#6366f1"
              gradientStopColor="var(--color-foreground)"
            />
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={shoppingRef} 
              toRef={consensusRef} 
              curvature={55}
              duration={5.2}
              gradientStartColor="#f97316"
              gradientStopColor="var(--color-foreground)"
            />

            {/* Consensus -> Judge */}
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={consensusRef} 
              toRef={judgeRef} 
              duration={3}
              gradientStartColor="var(--color-foreground)"
              gradientStopColor="#6366f1"
            />

            {/* REPLAN LOOPBACK (Judge -> Planner) - Arches beautifully over the specialists stack */}
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={judgeRef} 
              toRef={plannerRef} 
              curvature={-85}
              duration={6.5}
              reverse={true}
              gradientStartColor="#6366f1"
              gradientStopColor="#ef4444"
              pathColor="#ef4444"
              pathOpacity={0.15}
              pathWidth={1.5}
            />

            {/* Judge -> UI Composer */}
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={judgeRef} 
              toRef={vizRef} 
              duration={3.5}
              gradientStartColor="#6366f1"
              gradientStopColor="#a855f7"
            />

            {/* UI Composer -> Output */}
            <AnimatedBeam 
              containerRef={containerRef} 
              fromRef={vizRef} 
              toRef={outputRef} 
              duration={3.5}
              gradientStartColor="#a855f7"
              gradientStopColor="#10b981"
            />

          </div>

        </div>

        {/* Mobile Viewport: Stacked Timeline Flow */}
        <div className="block lg:hidden mt-8 space-y-6">
          <div className="rounded-xl border border-border bg-background/50 p-6 space-y-6">
            <div className="flex items-center gap-3">
              <User className="size-5 text-muted-foreground" />
              <div>
                <p className="text-xs font-mono font-bold text-foreground">1. User Entry</p>
                <p className="text-[10px] text-muted-foreground">Prompt parsed & contextualized</p>
              </div>
            </div>
            
            <div className="w-px h-6 bg-border ml-2.5" />

            <div className="flex items-center gap-3">
              <Cpu className="size-5 text-primary" />
              <div>
                <p className="text-xs font-mono font-bold text-primary">2. Cognitive Planner</p>
                <p className="text-[10px] text-muted-foreground">Decomposes and dispatches specialist tasks</p>
              </div>
            </div>

            <div className="w-px h-6 bg-border ml-2.5" />

            <div className="pl-4 border-l border-primary/20 space-y-4">
              <p className="text-[9px] font-mono uppercase tracking-wider text-muted-foreground">Parallel Specialists Running...</p>
              <div className="flex items-center gap-3">
                <ShieldAlert className="size-4 text-red-500" />
                <span className="text-xs text-foreground">Safety Audit Agent</span>
              </div>
              <div className="flex items-center gap-3">
                <Apple className="size-4 text-emerald-500" />
                <span className="text-xs text-foreground">Nutrition & Macros Agent</span>
              </div>
              <div className="flex items-center gap-3">
                <Coins className="size-4 text-amber-500" />
                <span className="text-xs text-foreground">Budget & Swaps Agent</span>
              </div>
              <div className="flex items-center gap-3">
                <Globe className="size-4 text-blue-500" />
                <span className="text-xs text-foreground">Culture & Dietary Agent</span>
              </div>
              <div className="flex items-center gap-3">
                <Utensils className="size-4 text-purple-500" />
                <span className="text-xs text-foreground">Recipe & Pantry Agent</span>
              </div>
              <div className="flex items-center gap-3">
                <Calendar className="size-4 text-indigo-500" />
                <span className="text-xs text-foreground">Workout & Schedule Agent</span>
              </div>
              <div className="flex items-center gap-3">
                <ShoppingCart className="size-4 text-orange-500" />
                <span className="text-xs text-foreground">Shopping List Agent</span>
              </div>
            </div>

            <div className="w-px h-6 bg-border ml-2.5" />

            <div className="flex items-center gap-3">
              <GitMerge className="size-5 text-slate-500" />
              <div>
                <p className="text-xs font-mono font-bold text-foreground">3. Consensus Layer</p>
                <p className="text-[10px] text-muted-foreground">Merges agent answers into cohesive meal plan</p>
              </div>
            </div>

            <div className="w-px h-6 bg-border ml-2.5" />

            <div className="flex items-center gap-3">
              <CheckCircle2 className="size-5 text-indigo-500" />
              <div>
                <p className="text-xs font-mono font-bold text-indigo-500">4. Judge Verification</p>
                <p className="text-[10px] text-muted-foreground">Audits final plan against constraints (triggers replan if violated)</p>
              </div>
            </div>

            <div className="w-px h-6 bg-border ml-2.5" />

            <div className="flex items-center gap-3">
              <Sparkles className="size-5 text-violet-500" />
              <div>
                <p className="text-xs font-mono font-bold text-violet-500">5. UI Composer</p>
                <p className="text-[10px] text-muted-foreground">Generates charts and timeline visuals</p>
              </div>
            </div>

            <div className="w-px h-6 bg-border ml-2.5" />

            <div className="flex items-center gap-3">
              <FileText className="size-5 text-emerald-500" />
              <div>
                <p className="text-xs font-mono font-bold text-emerald-500">6. Structured OS Payload</p>
                <p className="text-[10px] text-muted-foreground">Delivers complete response payload to user device</p>
              </div>
            </div>

          </div>
        </div>

      </div>
    </section>
  )
}
