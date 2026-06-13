"use client"

import React, { useState, useEffect } from "react"
import { useRouter } from "next/navigation"
import { motion, AnimatePresence } from "motion/react"

import { useAuthStore } from "@/store/auth-store"
import { DotPattern } from "@/components/ui/dot-pattern"
import { DecorIcon } from "@/components/core/auth/decor-icon"
import { AuthDivider } from "@/components/core/auth/auth-divider"
import { AuthBanner } from "@/components/core/auth/auth-banner"
import { AuthHeader } from "@/components/core/auth/auth-header"
import { AuthOAuth } from "@/components/core/auth/auth-oauth"
import { SignInForm } from "@/components/core/auth/signin-form"
import { SignupForm } from "@/components/core/auth/signup-form"
import { ForgotPasswordForm } from "@/components/core/auth/forgot-password-form"
import { ResetPasswordForm } from "@/components/core/auth/reset-password-form"
import { VerifyEmailForm } from "@/components/core/auth/verify-email-form"
import { useHydratedStore } from "@/hooks/use-hydrated-store"

type Screen = "signin" | "signup" | "forgot-password" | "reset-password" | "verify-email"

export function AuthPage() {
  const router = useRouter()
  
  // Hydration-safe store state readings
  const isAuthenticated = useHydratedStore(useAuthStore, (state) => state.isAuthenticated)
  const user = useHydratedStore(useAuthStore, (state) => state.user)
  const error = useAuthStore((state) => state.error)
  const clearError = useAuthStore((state) => state.clearError)

  const [screen, setScreen] = useState<Screen>("signin")
  const [email, setEmail] = useState("")

  // Derive the active view screen and active email based on authentication & verification status
  const isUnverified = isAuthenticated && !user?.email_verified
  const activeScreen = isUnverified ? "verify-email" : screen
  const activeEmail = isUnverified ? (user?.email || "") : email

  // Redirect if already authenticated and verified
  useEffect(() => {
    if (isAuthenticated && user?.email_verified) {
      router.replace("/")
    }
  }, [isAuthenticated, user, router])

  // Clear errors when toggling screens
  useEffect(() => {
    clearError()
  }, [activeScreen, clearError])

  return (
    <div className="relative flex min-h-screen w-full overflow-hidden bg-background select-none">
      {/* Dynamic Glowing dot background */}
      <DotPattern
        width={24}
        height={24}
        glow={true}
        cr={1.2}
        className="text-border opacity-40"
      />

      <div className="flex w-full flex-row">
        {/* Left Panel: Premium Branding Section (Desktop only) */}
        <AuthBanner />

        {/* Right Panel: Authentication Card (Mobile & Desktop) */}
        <div className="z-10 flex w-full flex-col items-center justify-center px-6 py-12 md:w-1/2 md:px-12 lg:px-20">
          <div className="relative w-full max-w-md rounded-2xl border border-border bg-card/60 p-8 shadow-[0_12px_40px_rgba(0,0,0,0.03)] dark:shadow-none backdrop-blur-xl transition-all duration-300">
            {/* Corner Decorative Crosshairs */}
            <DecorIcon position="top-left" className="text-border/80" />
            <DecorIcon position="bottom-right" className="text-border/80" />

            {/* Dynamic Headers */}
            <AuthHeader screen={activeScreen} />

            {/* Error Message Alert */}
            <AnimatePresence>
              {error && (
                <motion.div
                  initial={{ opacity: 0, y: -10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  className="mb-6 rounded-2xl border border-destructive/20 bg-destructive/10 p-4 text-sm text-destructive"
                >
                  {error}
                </motion.div>
              )}
            </AnimatePresence>

            {/* Switcher for auth flow screens with premium transitions */}
            <AnimatePresence mode="wait" initial={false}>
              <motion.div
                key={activeScreen}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                transition={{ duration: 0.2, ease: "easeInOut" }}
                className="w-full outline-none"
              >
                {activeScreen === "signin" && (
                  <SignInForm
                    onForgotPassword={() => setScreen("forgot-password")}
                  />
                )}

                {activeScreen === "signup" && (
                  <SignupForm
                    onSuccess={(signupEmail) => {
                      setEmail(signupEmail)
                      setScreen("verify-email")
                    }}
                  />
                )}

                {activeScreen === "forgot-password" && (
                  <ForgotPasswordForm
                    onSuccess={(resetEmail) => {
                      setEmail(resetEmail)
                      setScreen("reset-password")
                    }}
                    onBack={() => setScreen("signin")}
                  />
                )}

                {activeScreen === "reset-password" && (
                  <ResetPasswordForm
                    email={activeEmail}
                    onBack={() => setScreen("signin")}
                  />
                )}

                {activeScreen === "verify-email" && (
                  <VerifyEmailForm
                    email={activeEmail}
                    onBack={() => {
                      setScreen("signin")
                    }}
                  />
                )}
              </motion.div>
            </AnimatePresence>

            {/* Social Logins and Switcher (only for signin and signup screens) */}
            {(activeScreen === "signin" || activeScreen === "signup") && (
              <>
                {/* Social Dividers */}
                <AuthDivider className="my-6">Or continue with</AuthDivider>

                {/* OAuth Sign In Buttons */}
                <AuthOAuth />

                {/* Bottom Form Switcher */}
                <div className="mt-8 text-center text-sm">
                  <span className="text-muted-foreground">
                    {activeScreen === "signin"
                      ? "Don't have an account? "
                      : "Already have an account? "}
                  </span>
                  <button
                    type="button"
                    className="font-bold text-primary hover:underline bg-transparent border-0 cursor-pointer"
                    onClick={() =>
                      setScreen(activeScreen === "signin" ? "signup" : "signin")
                    }
                  >
                    {activeScreen === "signin" ? "Sign Up" : "Sign In"}
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
