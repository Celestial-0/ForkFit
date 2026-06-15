"use client"

import React from "react"
import { AnimatedList } from "@/components/ui/animated-list"
import { Sparkles, Terminal, Activity, ShieldCheck, Database, Key } from "lucide-react"

interface LogItem {
  id: string
  title: string
  description: string
  time: string
  icon: React.ReactNode
  color: string
}

const mockLogs: LogItem[] = [
  {
    id: "log-1",
    title: "User Authenticated",
    description: "Session verified (OAuth GitHub). Session TTL: 30 days.",
    time: "Just now",
    icon: <Key className="size-4" />,
    color: "bg-blue-500/10 text-blue-500 border-blue-500/25",
  },
  {
    id: "log-2",
    title: "Meal Log Registered",
    description: "User logged 'Spicy Tofu & Quinoa Bowl'. RAG embedding created.",
    time: "2m ago",
    icon: <Database className="size-4" />,
    color: "bg-emerald-500/10 text-emerald-500 border-emerald-500/25",
  },
  {
    id: "log-3",
    title: "Allergy Audit Complete",
    description: "Safety Agent verified 0 allergen conflicts. Severity: None.",
    time: "5m ago",
    icon: <ShieldCheck className="size-4" />,
    color: "bg-indigo-500/10 text-indigo-500 border-indigo-500/25",
  },
  {
    id: "log-4",
    title: "Memory Reflection Loop",
    description: "Extracted preference: Prefers vegan high-protein ingredients.",
    time: "12m ago",
    icon: <Sparkles className="size-4" />,
    color: "bg-purple-500/10 text-purple-500 border-purple-500/25",
  },
  {
    id: "log-5",
    title: "Budget Threshold Check",
    description: "Optimized pantry utilization (estimated grocery savings: $12.40).",
    time: "15m ago",
    icon: <Activity className="size-4" />,
    color: "bg-amber-500/10 text-amber-500 border-amber-500/25",
  },
]

export function LiveDemo() {
  return (
    <section className="relative w-full py-24 md:py-32 overflow-hidden bg-background">
      <div className="container mx-auto px-4 max-w-5xl relative z-10">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 items-center">
          
          {/* Left Column: Context & Explainer */}
          <div className="lg:col-span-5 space-y-6">
            <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-mono bg-indigo-500/10 text-indigo-500 border border-indigo-500/20">
              <Terminal className="size-3" />
              <span>Learning & Knowledge Loops</span>
            </div>
            
            <h2 className="text-3xl md:text-4xl font-extrabold tracking-tight font-heading text-foreground">
              Dynamic Feedback & Knowledge Graphs
            </h2>
            
            <p className="text-muted-foreground text-sm md:text-base font-sans leading-relaxed">
              ForkFit doesn't just calculate macros. Every meal logged feeds into our hybrid RAG database via pgvector, traversing your allergen history, kitchen inventory, and budget limits to continuously calibrate recommendations.
            </p>
            
            <div className="space-y-4 pt-2">
              <div className="flex gap-3 items-start">
                <div className="size-6 rounded-md bg-primary/10 border border-primary/20 flex items-center justify-center text-primary shrink-0 mt-0.5">
                  <Database className="size-3.5" />
                </div>
                <div>
                  <p className="text-sm font-semibold font-sans">pgvector Vector Search</p>
                  <p className="text-xs text-muted-foreground font-sans mt-0.5">Sub-second HNSW indexing for instant historical context lookup.</p>
                </div>
              </div>
              <div className="flex gap-3 items-start">
                <div className="size-6 rounded-md bg-primary/10 border border-primary/20 flex items-center justify-center text-primary shrink-0 mt-0.5">
                  <ShieldCheck className="size-3.5" />
                </div>
                <div>
                  <p className="text-sm font-semibold font-sans">Continuous Reflection Loops</p>
                  <p className="text-xs text-muted-foreground font-sans mt-0.5">Extracts subtle preference rules to fine-tune active memory vectors.</p>
                </div>
              </div>
            </div>
          </div>
          
          {/* Right Column: Animated List Logger */}
          <div className="lg:col-span-7 flex flex-col justify-center">
            <div className="w-full max-w-md mx-auto lg:max-w-none rounded-xl border border-border bg-zinc-50/50 dark:bg-zinc-950/40 p-4 md:p-6 shadow-sm backdrop-blur-sm relative">
              
              {/* Terminal Title Bar */}
              <div className="flex items-center justify-between border-b border-border/60 pb-3 mb-4">
                <div className="flex items-center gap-2">
                  <div className="flex gap-1.5">
                    <span className="size-2.5 rounded-full bg-red-500/60" />
                    <span className="size-2.5 rounded-full bg-amber-500/60" />
                    <span className="size-2.5 rounded-full bg-emerald-500/60" />
                  </div>
                  <span className="text-[10px] md:text-xs font-mono text-muted-foreground pl-2 border-l border-border/50">
                    forkfit-core-agent-logger
                  </span>
                </div>
                <div className="flex items-center gap-1.5 px-2 py-0.5 rounded bg-muted/60 text-[9px] font-mono text-muted-foreground">
                  <span className="size-1.5 rounded-full bg-emerald-500 animate-pulse" />
                  <span>LIVE TRACING</span>
                </div>
              </div>
              
              {/* Scrolling List */}
              <div className="relative min-h-[340px] flex flex-col justify-end overflow-hidden">
                <AnimatedList className="w-full">
                  {mockLogs.map((log) => (
                    <div 
                      key={log.id} 
                      className="flex items-start gap-4 p-3.5 rounded-lg border border-border bg-background/80 shadow-[0_2px_8px_rgba(0,0,0,0.02)] transition-all duration-300 hover:scale-[1.01]"
                    >
                      <div className={`size-8 rounded-full border ${log.color} flex items-center justify-center shrink-0`}>
                        {log.icon}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between gap-2">
                          <p className="text-xs font-bold font-sans text-foreground truncate">{log.title}</p>
                          <span className="text-[9px] font-mono text-muted-foreground shrink-0">{log.time}</span>
                        </div>
                        <p className="text-[11px] text-muted-foreground font-sans mt-0.5 leading-relaxed">{log.description}</p>
                      </div>
                    </div>
                  ))}
                </AnimatedList>
              </div>
            </div>
          </div>
          
        </div>
      </div>
    </section>
  )
}
