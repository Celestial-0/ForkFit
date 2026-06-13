import { cn } from "@/lib/utils";
import React from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Portal, PortalBackdrop } from "./portal";
import { navLinks } from "./index";
import { HugeiconsIcon } from "@hugeicons/react";
import { Cancel01Icon, Menu01Icon } from "@hugeicons/core-free-icons";
import { useHydratedStore } from "@/hooks/use-hydrated-store";
import { useAuthStore } from "@/store/auth-store";

export function MobileNav() {
	const [open, setOpen] = React.useState(false);
	const isAuthenticated = useHydratedStore(useAuthStore, (state) => state.isAuthenticated);

	const showAuth = isAuthenticated;

	return (
		<div className="md:hidden">
			<Button
				aria-controls="mobile-menu"
				aria-expanded={open}
				aria-label="Toggle menu"
				className="md:hidden"
				onClick={() => setOpen(!open)}
				size="icon"
				variant="outline"
			>
				{open ? (
					<HugeiconsIcon icon={Cancel01Icon} strokeWidth={2} className="size-4.5" />
				) : (
					<HugeiconsIcon icon={Menu01Icon} strokeWidth={2} className="size-4.5" />
				)}
			</Button>
			{open && (
				<Portal className="top-14" id="mobile-menu">
					<PortalBackdrop />
					<div
						className={cn(
							"data-[slot=open]:zoom-in-97 ease-out data-[slot=open]:animate-in",
							"size-full p-4"
						)}
						data-slot={open ? "open" : "closed"}
					>
						<div className="grid gap-y-2">
							{navLinks.map((link) => (
								<Button
									className="justify-start"
									key={link.label}
									variant="ghost"
									render={<a href={link.href} />}
									nativeButton={false}
									onClick={() => setOpen(false)}
								>
									{link.label}
								</Button>
							))}
						</div>
						{!showAuth && (
							<div className="mt-12 flex flex-col gap-2">
								<Button
									className="w-full"
									variant="outline"
									render={<Link href="/auth" />}
									nativeButton={false}
									onClick={() => setOpen(false)}
								>
									Sign In
								</Button>
								<Button
									className="w-full"
									render={<Link href="/auth" />}
									nativeButton={false}
									onClick={() => setOpen(false)}
								>
									Get Started
								</Button>
							</div>
						)}
					</div>
				</Portal>
			)}
		</div>
	);
}
