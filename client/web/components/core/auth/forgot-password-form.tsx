"use client"

import React from "react"
import { useForm } from "@tanstack/react-form-nextjs"
import { toast } from "sonner"
import { HugeiconsIcon } from "@hugeicons/react"
import { AtIcon, Loading03Icon } from "@hugeicons/core-free-icons"

import { Button } from "@/components/ui/button"
import {
  InputGroup,
  InputGroupInput,
  InputGroupAddon,
} from "@/components/ui/input-group"
import { forgotPasswordApi } from "@/lib/api/api"

interface ForgotPasswordFormProps {
  onSuccess: (email: string) => void
  onBack: () => void
}

export function ForgotPasswordForm({
  onSuccess,
  onBack,
}: ForgotPasswordFormProps) {
  const form = useForm({
    defaultValues: {
      email: "",
    },
    onSubmit: async ({ value }) => {
      if (!value.email) {
        toast.error("Please enter your email address.")
        return
      }

      try {
        await forgotPasswordApi({ email: value.email })
        toast.success("Password reset OTP sent to your email.")
        onSuccess(value.email)
      } catch (err: any) {
        toast.error(err.message || "Failed to send OTP. Please try again.")
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
      {/* Email Address */}
      <form.Field
        name="email"
        validators={{
          onChange: ({ value }) => {
            if (!value) return "Email is required"
            if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
              return "Please enter a valid email address"
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
              Email Address
            </label>
            <InputGroup className="h-11 border-input bg-muted/10 md:h-10">
              <InputGroupAddon align="inline-start">
                <HugeiconsIcon
                  icon={AtIcon}
                  className="size-4 text-muted-foreground"
                />
              </InputGroupAddon>
              <InputGroupInput
                id={field.name}
                name={field.name}
                type="email"
                placeholder="name@example.com"
                value={field.state.value}
                onBlur={field.handleBlur}
                onChange={(e) => field.handleChange(e.target.value)}
              />
            </InputGroup>
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
                  Sending OTP...
                </>
              ) : (
                "Send Reset OTP"
              )}
            </Button>

            <Button
              type="button"
              variant="ghost"
              className="h-11 w-full text-xs font-bold text-muted-foreground hover:text-foreground md:h-10"
              onClick={onBack}
              disabled={isSubmitting}
            >
              Back to Sign In
            </Button>
          </div>
        )}
      </form.Subscribe>
    </form>
  )
}
