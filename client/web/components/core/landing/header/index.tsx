"use client"
import Link from "next/link"
import { cn } from "@/lib/utils"
import { useScroll } from "@/hooks/use-scroll"
import { Button } from "@/components/ui/button"
import { MobileNav } from "./mobile-nav"
import { useHydratedStore } from "@/hooks/use-hydrated-store"
import { useAuthStore } from "@/store/auth-store"
import { GithubIcon } from "@/components/icons/github-icon"
import { UserMenu } from "./user-menu"
import { ForkFit } from "@/components/icons/main"

export const navLinks = [
  {
    label: "Chat",
    href: "/chat",
  },
  {
    label: "DashBoard",
    href: "/dashboard",
  },
  {
    label: "Docs",
    href: "#",
  },
]

export function Header() {
  const scrolled = useScroll(10)
  const isAuthenticated = useHydratedStore(
    useAuthStore,
    (state) => state.isAuthenticated
  )

  const showAuth = isAuthenticated

  return (
    <header
      className={cn(
        "sticky top-0 z-50 mx-auto w-full max-w-4xl border-b border-transparent md:rounded-md md:border md:transition-all md:ease-out",
        {
          "border-border bg-background/95 backdrop-blur-sm supports-backdrop-filter:bg-background/50 md:top-2 md:max-w-3xl md:shadow":
            scrolled,
        }
      )}
    >
      <nav
        className={cn(
          "flex h-14 w-full items-center justify-between px-4 md:h-12 md:transition-all md:ease-out",
          {
            "md:px-2": scrolled,
          }
        )}
      >
        <Link
          href="/"
          className="flex items-center gap-2 rounded-md p-2 hover:bg-muted dark:hover:bg-muted/50"
        >
          <ForkFit className="h-6 w-6" />
          <span>Fork<span className="text-muted-foreground">Fit</span></span>
        </Link>
        <div className="hidden items-center gap-2.5 md:flex">
          <div className="flex items-center gap-0.5">
            {navLinks.map((link) => (
              <Button
                key={link.label}
                size="sm"
                variant="ghost"
                className="text-muted-foreground hover:text-foreground"
                render={<a href={link.href} />}
                nativeButton={false}
              >
                {link.label}
              </Button>
            ))}
          </div>
          {showAuth ? (
            <UserMenu />
          ) : (
            <>
              <Button
                size="sm"
                variant="ghost"
                className="gap-1.5 text-muted-foreground hover:text-foreground"
                render={
                  <a
                    href="https://github.com"
                    target="_blank"
                    rel="noreferrer"
                  />
                }
                nativeButton={false}
              >
                <GithubIcon className="size-4" />
                <span>GitHub</span>
              </Button>
              <Button
                size="sm"
                render={<Link href="/auth" />}
                nativeButton={false}
              >
                Sign In
              </Button>
            </>
          )}
        </div>
        <div className="flex items-center gap-2 md:hidden">
          {showAuth && <UserMenu />}
          <MobileNav />
        </div>
      </nav>
    </header>
  )
}
