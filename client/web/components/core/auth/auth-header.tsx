import React from "react"

interface AuthHeaderProps {
  screen: "signin" | "signup" | "forgot-password" | "reset-password" | "verify-email"
}

export function AuthHeader({ screen }: AuthHeaderProps) {
  const getHeaderContent = () => {
    switch (screen) {
      case "signin":
        return {
          title: "Welcome Back",
          description: "Simplify your fitness journey. Sign in below.",
        }
      case "signup":
        return {
          title: "Create Account",
          description: "Start tracking and optimizing your health today.",
        }
      case "forgot-password":
        return {
          title: "Forgot Password",
          description: "Enter your email to receive a password reset OTP.",
        }
      case "reset-password":
        return {
          title: "Reset Password",
          description: "Enter the OTP sent to your email and choose a new password.",
        }
      case "verify-email":
        return {
          title: "Verify Email",
          description: "Please enter the verification OTP sent to your email.",
        }
    }
  }

  const { title, description } = getHeaderContent()

  return (
    <div className="mb-8 flex flex-col space-y-2">
      <h2 className="font-heading text-3xl font-black tracking-tight text-foreground">
        {title}
      </h2>
      <p className="text-sm text-muted-foreground">{description}</p>
    </div>
  )
}
