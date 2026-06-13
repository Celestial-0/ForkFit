"use client"

import React, { useState } from "react"
import { useForm } from "@tanstack/react-form-nextjs"
import { toast } from "sonner"
import { useRouter } from "next/navigation"
import { HugeiconsIcon } from "@hugeicons/react"
import {
  LockIcon,
  EyeIcon,
  EyeOffIcon,
  Loading03Icon,
} from "@hugeicons/core-free-icons"

import { Button } from "@/components/ui/button"
import {
  InputGroup,
  InputGroupInput,
  InputGroupAddon,
} from "@/components/ui/input-group"
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
} from "@/components/ui/input-otp"
import { resetPasswordApi } from "@/lib/api/api"
import { useAuthStore } from "@/store/auth-store"

interface ResetPasswordFormProps {
  email: string
  onBack: () => void
}

export function ResetPasswordForm({ email, onBack }: ResetPasswordFormProps) {
  const router = useRouter()
  const { signIn } = useAuthStore()
  const [showPassword, setShowPassword] = useState(false)
  const [isLoggingIn, setIsLoggingIn] = useState(false)

  const form = useForm({
    defaultValues: {
      otp: "",
      newPassword: "",
    },
    onSubmit: async ({ value }) => {
      if (!value.otp || !value.newPassword) {
        toast.error("Please enter both the OTP and your new password.")
        return
      }

      if (value.newPassword.length < 8) {
        toast.error("Password must be at least 8 characters long.")
        return
      }

      try {
        await resetPasswordApi({
          email,
          otp: value.otp,
          new_password: value.newPassword,
        })
        toast.success("Password reset successful!")

        // Auto sign-in after password reset
        setIsLoggingIn(true)
        try {
          await signIn({ email, password: value.newPassword })
          toast.success("Successfully signed in automatically!")
          router.push("/")
        } catch (loginErr: any) {
          toast.error("Auto sign-in failed. Please sign in manually.")
          onBack()
        }
      } catch (err: any) {
        toast.error(
          err.message ||
            "Failed to reset password. Please check your OTP and try again."
        )
      } finally {
        setIsLoggingIn(false)
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
      {/* OTP Input */}
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

      {/* New Password Input */}
      <form.Field
        name="newPassword"
        validators={{
          onChange: ({ value }) => {
            if (!value) return "Password is required"
            if (value.length < 8) {
              return "Password must be at least 8 characters long"
            }
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
              New Password
            </label>
            <InputGroup className="h-11 border-input bg-muted/10 md:h-10">
              <InputGroupAddon align="inline-start">
                <HugeiconsIcon
                  icon={LockIcon}
                  className="size-4 text-muted-foreground"
                />
              </InputGroupAddon>
              <InputGroupInput
                id={field.name}
                name={field.name}
                type={showPassword ? "text" : "password"}
                placeholder="••••••••"
                value={field.state.value}
                onBlur={field.handleBlur}
                onChange={(e) => field.handleChange(e.target.value)}
              />
              <InputGroupAddon
                align="inline-end"
                className="cursor-pointer transition-colors hover:text-foreground"
                onClick={() => setShowPassword(!showPassword)}
              >
                <HugeiconsIcon
                  icon={showPassword ? EyeOffIcon : EyeIcon}
                  className="size-4"
                />
              </InputGroupAddon>
            </InputGroup>
            {field.state.meta.isTouched && field.state.meta.errors.length > 0 && (
              <p className="text-xs text-destructive mt-0.5">
                {field.state.meta.errors[0]}
              </p>
            )}
          </div>
        )}
      </form.Field>

      {/* Submit button */}
      <form.Subscribe selector={(state) => [state.canSubmit, state.isSubmitting]}>
        {([canSubmit, isSubmitting]) => (
          <div className="space-y-2 mt-2">
            <Button
              type="submit"
              className="h-11 w-full text-sm font-bold tracking-tight md:h-10"
              disabled={isSubmitting || isLoggingIn || !canSubmit}
            >
              {isSubmitting || isLoggingIn ? (
                <>
                  <HugeiconsIcon
                    icon={Loading03Icon}
                    className="mr-2 size-4 animate-spin"
                  />
                  {isLoggingIn ? "Signing in..." : "Resetting password..."}
                </>
              ) : (
                "Reset Password"
              )}
            </Button>
            <Button
              type="button"
              variant="ghost"
              className="h-11 w-full text-xs font-bold text-muted-foreground hover:text-foreground md:h-10"
              onClick={onBack}
              disabled={isSubmitting || isLoggingIn}
            >
              Cancel
            </Button>
          </div>
        )}
      </form.Subscribe>
    </form>
  )
}
