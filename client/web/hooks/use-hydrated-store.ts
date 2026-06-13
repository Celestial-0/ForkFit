import { useSyncExternalStore } from "react"

/**
 * A custom React hook that safely subscribes to a Zustand persisted store
 * during Next.js SSR and client hydration.
 *
 * It prevents hydration mismatch errors by returning the initial state on the server
 * and during the first client render, then triggers a client-side update with the
 * persisted state once hydration completes.
 *
 * @param useStore The Zustand store hook instance (must support persist)
 * @param selector A selector function to pick state slices
 */
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

const emptySubscribe = () => () => {}

/**
 * A hook to check if the component has finished hydration and mounted on the client.
 * Leverages React 18's useSyncExternalStore to avoid synchronous useEffect state updates
 * and cascading renders.
 */
export function useIsHydrated(): boolean {
  return useSyncExternalStore(
    emptySubscribe,
    () => true,
    () => false
  )
}
