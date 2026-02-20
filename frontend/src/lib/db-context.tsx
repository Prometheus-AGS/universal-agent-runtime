import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { UarDb, setDbInstance } from "@/lib/db";

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

type DbContextValue =
  | { ready: false; db: null }
  | { ready: true; db: UarDb };

const DbContext = createContext<DbContextValue>({ ready: false, db: null });

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

interface DbProviderProps {
  children: ReactNode;
}

export function DbProvider({ children }: DbProviderProps) {
  const [value, setValue] = useState<DbContextValue>({ ready: false, db: null });

  useEffect(() => {
    let cancelled = false;
    UarDb.open()
      .then((db) => {
        if (!cancelled) {
          setDbInstance(db);
          setValue({ ready: true, db });
        }
      })
      .catch((err) => {
        console.error("[UarDb] Failed to open database:", err);
      });
    return () => { cancelled = true; };
  }, []);

  if (!value.ready) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <p className="font-mono text-xs text-muted-foreground">Initializing database…</p>
      </div>
    );
  }

  return <DbContext.Provider value={value}>{children}</DbContext.Provider>;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/** Returns the fully-initialized UarDb instance. Only callable inside <DbProvider>. */
export function useDb(): UarDb {
  const ctx = useContext(DbContext);
  if (!ctx.ready) throw new Error("useDb() called before database is ready");
  return ctx.db;
}
