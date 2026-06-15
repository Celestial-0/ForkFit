"use client"

import React from "react"
import Link from "next/link"
import { ForkFit } from "@/components/icons/main"
import { GithubIcon } from "@/components/icons/github-icon"
import { Heart } from "lucide-react"

export const Footer = () => {
  return (
    <footer className="w-full border-t border-border/60 bg-background/50 backdrop-blur-sm py-12 md:py-16 mt-12">
      <div className="container mx-auto px-4 max-w-5xl">
        <div className="grid grid-cols-1 md:grid-cols-12 gap-8 md:gap-12">
          
          {/* Logo & Vibe */}
          <div className="md:col-span-5 space-y-4">
            <Link href="/" className="flex items-center gap-2 font-heading font-extrabold text-foreground">
              <ForkFit className="h-6 w-6" />
              <span>Fork<span className="text-muted-foreground font-normal">Fit</span></span>
            </Link>
            <p className="text-xs text-muted-foreground max-w-[35ch] font-sans leading-relaxed">
              Enterprise Cognitive Intelligence Operating System for personalized nutrition modeling, safety auditing, and RAG memory graphs.
            </p>
          </div>

          {/* Links Column 1: Product */}
          <div className="md:col-span-2 space-y-3">
            <h4 className="text-xs font-mono font-bold uppercase tracking-wider text-foreground">Product</h4>
            <ul className="space-y-2 text-xs font-sans">
              <li>
                <Link href="/chat" className="text-muted-foreground hover:text-foreground transition-colors">
                  AI Chat Console
                </Link>
              </li>
              <li>
                <Link href="/dashboard" className="text-muted-foreground hover:text-foreground transition-colors">
                  User Dashboard
                </Link>
              </li>
              <li>
                <Link href="/auth" className="text-muted-foreground hover:text-foreground transition-colors">
                  Client Auth
                </Link>
              </li>
            </ul>
          </div>

          {/* Links Column 2: Resources */}
          <div className="md:col-span-2 space-y-3">
            <h4 className="text-xs font-mono font-bold uppercase tracking-wider text-foreground">Resources</h4>
            <ul className="space-y-2 text-xs font-sans">
              <li>
                <a href="https://celestial-0.github.io/ForkFit" target="_blank" rel="noreferrer" className="text-muted-foreground hover:text-foreground transition-colors">
                  Documentation
                </a>
              </li>
              <li>
                <Link href="#" className="text-muted-foreground hover:text-foreground transition-colors">
                  API Spec
                </Link>
              </li>
              <li>
                <Link href="#" className="text-muted-foreground hover:text-foreground transition-colors">
                  RBAC Seed Setup
                </Link>
              </li>
            </ul>
          </div>

          {/* Links Column 3: Social/Repo */}
          <div className="md:col-span-3 space-y-3">
            <h4 className="text-xs font-mono font-bold uppercase tracking-wider text-foreground">Developer</h4>
            <ul className="space-y-2 text-xs font-sans">
              <li>
                <a 
                  href="https://github.com" 
                  target="_blank" 
                  rel="noreferrer" 
                  className="inline-flex items-center gap-1.5 text-muted-foreground hover:text-foreground transition-colors"
                >
                  <GithubIcon className="size-3.5" />
                  <span>GitHub Repository</span>
                </a>
              </li>
            </ul>
          </div>

        </div>

        {/* Divider */}
        <div className="border-t border-border/40 my-8 pt-8 flex flex-col sm:flex-row items-center justify-between gap-4">
          <p className="text-[10px] md:text-xs text-muted-foreground font-mono">
            &copy; {new Date().getFullYear()} ForkFit. All rights reserved.
          </p>
          <div className="flex items-center gap-1 text-[10px] md:text-xs text-muted-foreground font-sans">
            <span>Built with</span>
            <Heart className="size-3 text-red-500 fill-red-500" />
            <span>using Rust & Next.js</span>
          </div>
        </div>
      </div>
    </footer>
  )
}
