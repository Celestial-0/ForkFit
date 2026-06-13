"use client"

import React, { useState, useEffect } from "react"
import { useRouter } from "next/navigation"
import { useForm } from "@tanstack/react-form-nextjs"
import { toast } from "sonner"
import { HugeiconsIcon } from "@hugeicons/react"
import { Loading03Icon } from "@hugeicons/core-free-icons"

import { Button } from "@/components/ui/button"
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
} from "@/components/ui/input-otp"
import { verifyEmailApi, sendVerificationOtpApi } from "@/lib/api/api"
import { useAuthStore } from "@/store/auth-store"

interface VerifyEmailFormProps {
  email: string
  onBack: () => void
}

export function VerifyEmailForm({ email, onBack }: VerifyEmailFormProps) {
  const router = useRouter()
  const [countdown, setCountdown] = useState(60)
  const [isResending, setIsResending] = useState(false)

  // Countdown timer for resending OTP
  useEffect(() => {
    if (countdown === 0) return

    const timer = setInterval(() => {
      setCountdown((prev) => prev - 1)
    }, 1000)

    return () => clearInterval(timer)
  }, [countdown])

  const handleResendOtp = async () => {
    if (countdown > 0 || isResending) return

    setIsResending(true)
    try {
      await sendVerificationOtpApi({ email })
      toast.success("Verification OTP resent successfully!")
      setCountdown(60)
    } catch (err: any) {
      toast.error(err.message || "Failed to resend OTP. Please try again.")
    } finally {
      setIsResending(false)
    }
  }

  const form = useForm({
    defaultValues: {
      otp: "",
    },
    onSubmit: async ({ value }) => {
      if (!value.otp) {
        toast.error("Please enter the OTP.")
        return
      }

      try {
        await verifyEmailApi({ email, otp: value.otp })
        toast.success("Email verified successfully!")

        // Update the Zustand store user to reflect verified status
        useAuthStore.setState((state) => {
          if (state.user) {
            return {
              user: {
                ...state.user,
                email_verified: true,
              },
            }
          }
          return {}
        })

        // Route to landing dashboard
        router.push("/")
      } catch (err: any) {
        toast.error(
          err.message ||
            "Verification failed. Please check the OTP and try again."
        )
      }
    },
  })

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault()
        e.stopPropagation()
        form.handleSubmit()
      }}
      className="space-y-4"
    >
      <form.Field
        name="otp"
        validators={{
          onChange: ({ value }) => {
            if (!value) return "OTP is required"
            return undefined
          },
        }}
      >
        {(field) => (
          <div className="space-y-1">
            <label
              htmlFor={field.name}
              className="text-xs font-bold tracking-wider text-muted-foreground uppercase"
            >
              Verification OTP
            </label>
            <div className="flex justify-center py-2">
              <InputOTP
                id={field.name}
                maxLength={6}
                value={field.state.value}
                onBlur={field.handleBlur}
                onChange={(value) => field.handleChange(value)}
              >
                <InputOTPGroup>
                  <InputOTPSlot index={0} className="size-12 text-base md:size-10" />
                  <InputOTPSlot index={1} className="size-12 text-base md:size-10" />
                  <InputOTPSlot index={2} className="size-12 text-base md:size-10" />
                  <InputOTPSlot index={3} className="size-12 text-base md:size-10" />
                  <InputOTPSlot index={4} className="size-12 text-base md:size-10" />
                  <InputOTPSlot index={5} className="size-12 text-base md:size-10" />
                </InputOTPGroup>
              </InputOTP>
            </div>
            {field.state.meta.isTouched && field.state.meta.errors.length > 0 && (
              <p className="text-xs text-destructive mt-0.5">
                {field.state.meta.errors[0]}
              </p>
            )}
          </div>
        )}
      </form.Field>

      <form.Subscribe selector={(state) => [state.canSubmit, state.isSubmitting]}>
        {([canSubmit, isSubmitting]) => (
          <div className="space-y-2 mt-2">
            <Button
              type="submit"
              className="h-11 w-full text-sm font-bold tracking-tight md:h-10"
              disabled={isSubmitting || !canSubmit}
            >
              {isSubmitting ? (
                <>
                  <HugeiconsIcon
                    icon={Loading03Icon}
                    className="mr-2 size-4 animate-spin"
                  />
                  Verifying...
                </>
              ) : (
                "Verify Email"
              )}
            </Button>

            <div className="text-center pt-2">
              {countdown > 0 ? (
                <span className="text-xs text-muted-foreground">
                  Resend OTP in {countdown}s
                </span>
              ) : (
                <button
                  type="button"
                  onClick={handleResendOtp}
                  disabled={isResending}
                  className="text-xs font-bold text-primary hover:underline bg-transparent border-0 cursor-pointer"
                >
                  {isResending ? "Resending..." : "Resend OTP"}
                </button>
              )}
            </div>

            <Button
              type="button"
              variant="ghost"
              className="h-11 w-full text-xs font-bold text-muted-foreground hover:text-foreground md:h-10"
              onClick={onBack}
            >
              Back
            </Button>
          </div>
        )}
      </form.Subscribe>
    </form>
  )
}
