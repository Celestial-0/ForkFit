"use client";

import { useAuthStore } from "@/store/auth-store";
import { useTheme } from "next-themes";
import { LogOut, Sun, Moon, Laptop, User, ChevronsUpDown, LayoutDashboard } from "lucide-react";
import { Avatar, AvatarImage, AvatarFallback } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubTrigger,
  DropdownMenuSubContent,
} from "@/components/ui/dropdown-menu";
import { useRouter } from "next/navigation";

export function ProfileFooter() {
  const user = useAuthStore((state) => state.user);
  const logout = useAuthStore((state) => state.logout);
  const { theme, setTheme } = useTheme();
  const router = useRouter();

  const handleLogout = async () => {
    if (confirm("Are you sure you want to sign out?")) {
      await logout();
      router.push("/auth");
    }
  };

  const getInitials = (name?: string, email?: string) => {
    if (name) {
      return name
        .split(" ")
        .map((n) => n[0])
        .join("")
        .toUpperCase()
        .substring(0, 2);
    }
    if (email) {
      return email.substring(0, 2).toUpperCase();
    }
    return "US";
  };

  return (
    <div className="border-t p-3 bg-muted/20 flex flex-col mt-auto">
      <DropdownMenu>
        <DropdownMenuTrigger className="flex w-full items-center gap-3 rounded-lg p-2 text-left hover:bg-muted/65 transition-all outline-none cursor-pointer group active:scale-[0.98]">
          <Avatar className="size-9 border bg-secondary shrink-0 group-hover:scale-105 transition-transform duration-300">
            <AvatarImage
              src={user?.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${encodeURIComponent(user?.full_name || user?.email || "user")}`}
              alt={user?.full_name || "User Avatar"}
            />
            <AvatarFallback className="text-xs font-semibold">
              {getInitials(user?.full_name, user?.email)}
            </AvatarFallback>
          </Avatar>
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium text-foreground truncate">
              {user?.full_name || user?.email || "User"}
            </p>
            <p className="text-xs text-muted-foreground truncate">
              {user?.email || "No email info"}
            </p>
          </div>
          <ChevronsUpDown className="size-4 text-muted-foreground/60 group-hover:text-foreground transition-colors shrink-0" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" side="top" className="w-56 mb-2">
          <div className="px-2.5 py-2 flex flex-col gap-0.5 border-b border-border/40 select-none">
            <span className="text-sm font-bold text-foreground truncate">
              {user?.full_name || user?.email || "User"}
            </span>
            <span className="text-xs text-muted-foreground truncate font-normal">
              {user?.email || "No email info"}
            </span>
          </div>

          <div className="p-1">
            <DropdownMenuItem
              className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
              onClick={() => router.push("/dashboard?tab=logs")}
            >
              <LayoutDashboard className="size-4 text-muted-foreground" />
              <span>Dashboard</span>
            </DropdownMenuItem>

            <DropdownMenuItem
              className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
              onClick={() => router.push("/dashboard?tab=profile")}
            >
              <User className="size-4 text-muted-foreground" />
              <span>Profile Settings</span>
            </DropdownMenuItem>

            <DropdownMenuSub>
              <DropdownMenuSubTrigger className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer">
                {theme === "light" && <Sun className="size-4 text-muted-foreground" />}
                {theme === "dark" && <Moon className="size-4 text-muted-foreground" />}
                {theme === "system" && <Laptop className="size-4 text-muted-foreground" />}
                <span>Theme</span>
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent className="w-36">
                <DropdownMenuItem
                  className="flex items-center justify-between rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
                  onClick={() => setTheme("light")}
                >
                  <div className="flex items-center gap-2">
                    <Sun className="size-4 text-muted-foreground" />
                    <span>Light</span>
                  </div>
                  {theme === "light" && <span className="text-xs text-primary font-bold">✓</span>}
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="flex items-center justify-between rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
                  onClick={() => setTheme("dark")}
                >
                  <div className="flex items-center gap-2">
                    <Moon className="size-4 text-muted-foreground" />
                    <span>Dark</span>
                  </div>
                  {theme === "dark" && <span className="text-xs text-primary font-bold">✓</span>}
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="flex items-center justify-between rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
                  onClick={() => setTheme("system")}
                >
                  <div className="flex items-center gap-2">
                    <Laptop className="size-4 text-muted-foreground" />
                    <span>System</span>
                  </div>
                  {theme === "system" && <span className="text-xs text-primary font-bold">✓</span>}
                </DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuSub>
          </div>

          <DropdownMenuSeparator />

          <div className="p-1">
            <DropdownMenuItem
              variant="destructive"
              className="flex items-center gap-2 rounded-lg px-2.5 py-1.5 text-sm cursor-pointer"
              onClick={handleLogout}
            >
              <LogOut className="size-4 text-destructive" />
              <span>Sign Out</span>
            </DropdownMenuItem>
          </div>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

