import { createContext, type ReactNode, useContext, useMemo, useState } from "react";

/** Holds the per-thread memory toggle state for the chat UI. */
export interface MemoryContextValue {
    /** Whether memory is enabled for the current thread's next message. */
    memoryEnabled: boolean;
    setMemoryEnabled: (v: boolean) => void;
}

const MemoryCtx = createContext<MemoryContextValue>({
    memoryEnabled: true,
    setMemoryEnabled: () => undefined,
});

export function MemoryContextProvider({ children }: { children: ReactNode }) {
    const [memoryEnabled, setMemoryEnabled] = useState(true);
    const value = useMemo(() => ({ memoryEnabled, setMemoryEnabled }), [memoryEnabled]);
    return <MemoryCtx.Provider value={value}>{children}</MemoryCtx.Provider>;
}

export function useMemoryContext(): MemoryContextValue {
    return useContext(MemoryCtx);
}

export const MemoryContext = MemoryCtx;
