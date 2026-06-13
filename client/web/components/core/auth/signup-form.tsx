"use client"

import React, { useState } from "react"
import { useForm } from "@tanstack/react-form-nextjs"
import { toast } from "sonner"
import { HugeiconsIcon } from "@hugeicons/react"
import {
  AtIcon,
  LockIcon,
  UserIcon,
  EyeIcon,
  EyeOffIcon,
  Loading03Icon,
} from "@hugeicons/core-free-icons"

import { useAuthStore } from "@/store/auth-store"
import { Button } from "@/components/ui/button"
import {
  InputGroup,
  InputGroupInput,
  InputGroupAddon,
} from "@/components/ui/input-group"

interface SignupFormProps {
  onSuccess: (email: string) => void
}

export function SignupForm({ onSuccess }: SignupFormProps) {
  const { signup, isLoading } = useAuthStore()
  const [showPassword, setShowPassword] = useState(false)

  const form = useForm({
    defaultValues: {
      fullName: "",
      email: "",
      password: "",
    },
    onSubmit: async ({ value }) => {
      if (!value.email || !value.password) {
        toast.error("Please fill in all required fields.")
        return
      }

      if (value.password.length < 8) {
        toast.error("Password must be at least 8 characters long.")
        return
      }

      try {
        await signup({
          email: value.email,
          password: value.password,
          full_name: value.fullName || undefined,
        })
        toast.success(
          "Successfully signed up! Verification OTP sent to your email."
        )
        onSuccess(value.email)
      } catch (err: any) {
        toast.error(err.message || "Authentication failed. Please try again.")
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
      {/* Full Name */}
      <form.Field name="fullName">
        {(field) => (
          <div className="space-y-1">
            <label
              htmlFor={field.name}
              className="text-xs font-bold tracking-wider text-muted-foreground uppercase"
            >
              Full Name
            </label>
            <InputGroup className="h-11 border-input bg-muted/10 md:h-10">
              <InputGroupAddon align="inline-start">
                <HugeiconsIcon
                  icon={UserIcon}
                  className="size-4 text-muted-foreground"
                />
              </InputGroupAddon>
              <InputGroupInput
                id={field.name}
                name={field.name}
                type="text"
                placeholder="John Doe"
                value={field.state.value}
                onBlur={field.handleBlur}
                onChange={(e) => field.handleChange(e.target.value)}
              />
            </InputGroup>
          </div>
        )}
      </form.Field>

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

      {/* Password */}
      <form.Field
        name="password"
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
              Password
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

      {/* Submit Button */}
      <form.Subscribe selector={(state) => [state.canSubmit, state.isSubmitting]}>
        {([canSubmit, isSubmitting]) => (
          <Button
            type="submit"
            className="mt-2 h-11 w-full text-sm font-bold tracking-tight md:h-10"
            disabled={isLoading || isSubmitting || !canSubmit}
          >
            {isLoading || isSubmitting ? (
              <>
                <HugeiconsIcon
                  icon={Loading03Icon}
                  className="mr-2 size-4 animate-spin"
                />
                Connecting...
              </>
            ) : (
              "Create Account"
            )}
          </Button>
        )}
      </form.Subscribe>
    </form>
  )
}
