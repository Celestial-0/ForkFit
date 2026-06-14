import React from "react"
import { Button } from "@/components/ui/button"
import { GoogleIcon } from "@/components/icons/google-icon"
import { GithubIcon } from "@/components/icons/github-icon"
import { toast } from "sonner"
import { API_BASE_URL } from "@/lib/api/api"

export function AuthOAuth() {
  const handleProviderSignIn = (provider: "google" | "github") => {
    toast.info(`Redirecting to ${provider} OAuth...`)
    window.location.href = `${API_BASE_URL}/auth/oauth/${provider}/authorize`
  }

  return (
    <div className="grid grid-cols-2 gap-3">
      <Button
        type="button"
        variant="outline"
        className="h-11 rounded-2xl border border-border/80 text-xs font-bold hover:bg-muted md:h-10"
        onClick={() => handleProviderSignIn("google")}
      >
        <GoogleIcon className="mr-2 size-4" />
        Google
      </Button>
      <Button
        type="button"
        variant="outline"
        className="h-11 rounded-2xl border border-border/80 text-xs font-bold hover:bg-muted md:h-10"
        onClick={() => handleProviderSignIn("github")}
      >
        <GithubIcon className="mr-2 size-4" />
        GitHub
      </Button>
    </div>
  )
}
