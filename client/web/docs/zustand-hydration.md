# Zustand Persist Hydration Pattern for Next.js App Router

This document serves as a persistent project memory and guide for safely subscribing to Zustand persisted stores in Next.js (App Router/SSR) without triggering hydration mismatch errors or React compilation warnings.

## The Problem
When using Zustand's `persist` middleware, state is synced with client-side storage (e.g., `localStorage`).
- **During SSR (Server-Side)**: Node.js has no access to `localStorage` and renders the HTML using the store's **initial default values**.
- **During Client-Side Hydration (Initial Render)**: The browser reads the persisted state from `localStorage`. If it differs from the server-rendered default values, React detects a mismatch between server and client HTML, throwing a **Hydration Mismatch Error**.

### Why the Legacy "Mounted State" is Discouraged
A common workaround is to use a `mounted` state:
```tsx
const [mounted, setMounted] = useState(false);
useEffect(() => setMounted(true), []);
const showUI = mounted && stateValue;
```
While this works, calling `setMounted(true)` synchronously inside the `useEffect` body triggers a **cascading render** immediately after mounting. This hurts performance and is flagged as a warning by modern React compilers and linters:
> *Calling setState synchronously within an effect body causes cascading renders that can hurt performance, and is not recommended.*

---

## The Production-Grade Solution: `useSyncExternalStore`
React 18 introduced `useSyncExternalStore` specifically to synchronize React components with external, non-React data sources (like a Zustand store).

We implement a custom hook `useHydratedStore` that delegates rendering synchronization directly to React's scheduler:
1. **Server Render & Initial Client Render (Hydration)**: React calls the third argument (`getServerSnapshot`), which returns the store's initial, default state using `useStore.getInitialState()`.
2. **Post-Hydration Client Render**: Once the component is mounted, React runs the subscription and reads the second argument (`getSnapshot`), returning the actual client state (loaded from `localStorage`). If the values differ, React schedules a safe, clean re-render with the client-hydrated values.

This guarantees that the initial client render exactly matches the server render, completely preventing hydration errors while avoiding the cascading renders warning.

---

## Reusable API & Code

### The Hook: `@/hooks/use-hydrated-store.ts`
```typescript
import { useSyncExternalStore } from "react"

export function useHydratedStore<T, U>(
  useStore: {
    subscribe: (listener: () => void) => () => void
    getState: () => T
    getInitialState: () => T
  },
  selector: (state: T) => U
): U {
  return useSyncExternalStore(
    useStore.subscribe,
    () => selector(useStore.getState()),
    () => selector(useStore.getInitialState())
  )
}
```

### Usage Example: Component
Instead of tracking `mounted` state with `useEffect`, read persisted state directly through the hook:

```tsx
"use client"

import { useHydratedStore } from "@/hooks/use-hydrated-store"
import { useAuthStore } from "@/store/auth-store"

export function HeaderProfile() {
  // Safely read persisted state - returns default/null on server/hydration, and actual state afterwards
  const isAuthenticated = useHydratedStore(useAuthStore, (state) => state.isAuthenticated)
  const user = useHydratedStore(useAuthStore, (state) => state.user)
  
  // Non-persisted actions (like methods/logout) do not require useHydratedStore and can be read normally:
  const logout = useAuthStore((state) => state.logout)

  if (!isAuthenticated) {
    return <button>Sign In</button>
  }

  return <div>Welcome, {user?.email}</div>
}
```
