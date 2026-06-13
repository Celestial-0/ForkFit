"use client"

import React, { useState } from "react"
import Image from "next/image"
import { useRouter } from "next/navigation"
import { useIsMobile } from "@/hooks/use-mobile"
import { useAuthStore } from "@/store/auth-store"
import { useHydratedStore, useIsHydrated } from "@/hooks/use-hydrated-store"
import { Skeleton } from "@/components/ui/skeleton"
import { HugeiconsIcon } from "@hugeicons/react"
import { useTheme } from "next-themes"
import {
  UserIcon,
  DashboardCircleIcon,
  Logout01Icon,
  Settings01Icon,
  Loading03Icon,
  SunIcon,
  MoonIcon,
  ComputerIcon,
  Tick02Icon,
} from "@hugeicons/core-free-icons"

import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
} from "@/components/ui/dropdown-menu"

import {
  Drawer,
  DrawerTrigger,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
  DrawerDescription,
  DrawerFooter,
} from "@/components/ui/drawer"
import { Button } from "@/components/ui/button"


export function UserMenu() {
  const router = useRouter()
  const { theme, setTheme } = useTheme()
  const [isSigningOut, setIsSigningOut] = useState(false)
  const [isOpen, setIsOpen] = useState(false)

  const isHydrated = useIsHydrated()

  const user = useHydratedStore(useAuthStore, (state) => state.user)
  const isAuthenticated = useHydratedStore(useAuthStore, (state) => state.isAuthenticated)
  const isLoading = useAuthStore((state) => state.isLoading)
  const logout = useAuthStore((state) => state.logout)

  const isMobile = useIsMobile()

  const handleSignOut = async (e: React.MouseEvent) => {
    e.preventDefault()
    if (isSigningOut) return
    setIsSigningOut(true)
    try {
      await logout()
      router.push("/")
      setIsOpen(false)
    } catch (err) {
      console.error("Logout failed:", err)
    } finally {
      setIsSigningOut(false)
    }
  }

  const handleNavigate = (path: string) => {
    router.push(path)
    setIsOpen(false)
  }

  // Hydration or loading state placeholder
  if (!isHydrated || isLoading) {
    return <Skeleton className="size-8 rounded-full" />
  }

  if (!isAuthenticated || !user) {
    return null
  }

  const triggerClasses =
    "relative flex size-8 shrink-0 items-center justify-center rounded-full border border-border bg-muted/30 outline-none transition-transform active:scale-95 cursor-pointer hover:bg-muted/50 overflow-hidden"

  const triggerContent = user.avatar_url ? (
    <Image
      src={user.avatar_url}
      alt={user.full_name || user.email}
      className="size-full object-cover"
    />
  ) : (
    <HugeiconsIcon icon={UserIcon} className="size-4.5 text-muted-foreground" />
  )

  if (isMobile) {
    return (
      <Drawer open={isOpen} onOpenChange={setIsOpen}>
        <DrawerTrigger className={triggerClasses}>
          {triggerContent}
        </DrawerTrigger>
        <DrawerContent className="p-4">
          <DrawerHeader className="text-left border-b border-border/50 pb-4">
            <DrawerTitle className="text-sm font-semibold truncate">
              {user.full_name || "Profile"}
            </DrawerTitle>
            <DrawerDescription className="text-xs text-muted-foreground truncate">
              {user.email}
            </DrawerDescription>
          </DrawerHeader>

          <div className="py-4 space-y-1">
            <Button
              variant="ghost"
              className="w-full justify-start h-10 px-3 text-sm font-medium"
              onClick={() => handleNavigate("/dashboard")}
            >
              <HugeiconsIcon
                icon={DashboardCircleIcon}
                className="mr-2 size-4 text-muted-foreground"
              />
              Dashboard
            </Button>
            <Button
              variant="ghost"
              className="w-full justify-start h-10 px-3 text-sm font-medium"
              onClick={() => handleNavigate("/profile")}
            >
              <HugeiconsIcon
                icon={Settings01Icon}
                className="mr-2 size-4 text-muted-foreground"
              />
              Settings
            </Button>
          </div>

          <div className="py-4 border-t border-border/50">
            <span className="text-xs font-bold tracking-wider text-muted-foreground uppercase px-4 block mb-2 select-none">
              Theme
            </span>
            <div className="grid grid-cols-3 gap-1 px-3">
              <Button
                variant={theme === "light" ? "secondary" : "ghost"}
                size="sm"
                className="h-9 rounded-xl text-xs font-bold"
                onClick={() => setTheme("light")}
              >
                <HugeiconsIcon icon={SunIcon} className="mr-1.5 size-3.5 text-muted-foreground" />
                Light
              </Button>
              <Button
                variant={theme === "dark" ? "secondary" : "ghost"}
                size="sm"
                className="h-9 rounded-xl text-xs font-bold"
                onClick={() => setTheme("dark")}
              >
                <HugeiconsIcon icon={MoonIcon} className="mr-1.5 size-3.5 text-muted-foreground" />
                Dark
              </Button>
              <Button
                variant={theme === "system" ? "secondary" : "ghost"}
                size="sm"
                className="h-9 rounded-xl text-xs font-bold"
                onClick={() => setTheme("system")}
              >
                <HugeiconsIcon icon={ComputerIcon} className="mr-1.5 size-3.5 text-muted-foreground" />
                System
              </Button>
            </div>
          </div>

          <DrawerFooter className="border-t border-border/50 pt-4 px-0">
            <Button
              variant="destructive"
              className="w-full h-10 text-sm font-bold"
              onClick={handleSignOut}
              disabled={isSigningOut}
            >
              {isSigningOut ? (
                <>
                  <HugeiconsIcon
                    icon={Loading03Icon}
                    className="mr-2 size-4 animate-spin"
                  />
                  Signing out...
                </>
              ) : (
                <>
                  <HugeiconsIcon icon={Logout01Icon} className="mr-2 size-4" />
                  Sign Out
                </>
              )}
            </Button>
          </DrawerFooter>
        </DrawerContent>
      </Drawer>
    )
  }

  return (
    <DropdownMenu open={isOpen} onOpenChange={setIsOpen}>
      <DropdownMenuTrigger className={triggerClasses}>
        {triggerContent}
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="end"
        className="w-56 mt-1.5 p-1 rounded-xl shadow-md border border-border bg-popover text-popover-foreground"
      >
        <div className="px-2.5 py-2 flex flex-col gap-0.5 border-b border-border/40 select-none">
          <span className="text-sm font-bold text-foreground truncate">
            {user.full_name || "Account"}
          </span>
          <span className="text-xs text-muted-foreground truncate font-normal">
            {user.email}
          </span>
        </div>

        <div className="p-1">
          <DropdownMenuItem
            className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
            onClick={() => handleNavigate("/dashboard")}
          >
            <HugeiconsIcon
              icon={DashboardCircleIcon}
              className="size-4 text-muted-foreground group-hover/dropdown-menu-item:text-foreground"
            />
            <span>Dashboard</span>
          </DropdownMenuItem>

          <DropdownMenuItem
            className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
            onClick={() => handleNavigate("/profile")}
          >
            <HugeiconsIcon
              icon={Settings01Icon}
              className="size-4 text-muted-foreground group-hover/dropdown-menu-item:text-foreground"
            />
            <span>Settings</span>
          </DropdownMenuItem>

          <DropdownMenuSub>
            <DropdownMenuSubTrigger className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer">
              <HugeiconsIcon
                icon={theme === "light" ? SunIcon : theme === "dark" ? MoonIcon : ComputerIcon}
                className="size-4 text-muted-foreground"
              />
              <span>Theme</span>
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="w-36">
              <DropdownMenuItem
                className="flex items-center justify-between rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
                onClick={() => setTheme("light")}
              >
                <div className="flex items-center gap-2">
                  <HugeiconsIcon icon={SunIcon} className="size-4 text-muted-foreground" />
                  <span>Light</span>
                </div>
                {theme === "light" && <HugeiconsIcon icon={Tick02Icon} className="size-4 text-primary" />}
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex items-center justify-between rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
                onClick={() => setTheme("dark")}
              >
                <div className="flex items-center gap-2">
                  <HugeiconsIcon icon={MoonIcon} className="size-4 text-muted-foreground" />
                  <span>Dark</span>
                </div>
                {theme === "dark" && <HugeiconsIcon icon={Tick02Icon} className="size-4 text-primary" />}
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex items-center justify-between rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
                onClick={() => setTheme("system")}
              >
                <div className="flex items-center gap-2">
                  <HugeiconsIcon icon={ComputerIcon} className="size-4 text-muted-foreground" />
                  <span>System</span>
                </div>
                {theme === "system" && <HugeiconsIcon icon={Tick02Icon} className="size-4 text-primary" />}
              </DropdownMenuItem>
            </DropdownMenuSubContent>
          </DropdownMenuSub>
        </div>

        <DropdownMenuSeparator />

        <div className="p-1">
          <DropdownMenuItem
            variant="destructive"
            className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
            onClick={handleSignOut}
            disabled={isSigningOut}
          >
            {isSigningOut ? (
              <>
                <HugeiconsIcon
                  icon={Loading03Icon}
                  className="size-4 animate-spin text-destructive"
                />
                <span>Signing out...</span>
              </>
            ) : (
              <>
                <HugeiconsIcon icon={Logout01Icon} className="size-4 text-destructive" />
                <span>Sign Out</span>
              </>
            )}
          </DropdownMenuItem>
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
