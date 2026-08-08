import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import { UarDb, setDbInstance, type OnStatusFn } from "@/platform/pglite/client";
import { bootstrapDurableEntityGraph } from "@/entities/bootstrap";

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

type DbContextValue =
  | { ready: false; db: null }
  | { ready: true; db: UarDb };

const DbContext = createContext<DbContextValue>({ ready: false, db: null });

// ---------------------------------------------------------------------------
// Loading screen
// ---------------------------------------------------------------------------

interface DbLoadingScreenProps {
  steps: string[];
  current: string;
  error: string | null;
}

function Spinner() {
  return (
    <svg
      className="h-8 w-8 animate-spin text-primary"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <circle
        className="opacity-20"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="3"
      />
      <path
        className="opacity-80"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
      />
    </svg>
  );
}

function ErrorIcon() {
  return (
    <svg
      className="h-8 w-8 text-destructive"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      stroke="currentColor"
      strokeWidth={2}
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="8" x2="12" y2="12" />
      <line x1="12" y1="16" x2="12.01" y2="16" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg
      className="h-3 w-3 shrink-0 text-primary"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 20 20"
      fill="currentColor"
      aria-hidden="true"
    >
      <path
        fillRule="evenodd"
        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
        clipRule="evenodd"
      />
    </svg>
  );
}

function DbLoadingScreen({ steps, current, error }: DbLoadingScreenProps) {
  const logRef = useRef<HTMLDivElement>(null);

  // Auto-scroll log to bottom as steps arrive
  const stepCount = steps.length;
  useEffect(() => {
    if (stepCount === 0) return;
    const el = logRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [stepCount]);

  // All steps except the very last one (which is the "current" step in progress)
  const completedSteps = steps.slice(0, -1);

  return (
    <div className="flex h-screen flex-col items-center justify-center gap-6 bg-background px-4">
      {/* Brand */}
      <div className="flex flex-col items-center gap-1 text-center">
        <p className="font-mono text-[10px] uppercase tracking-[0.25em] text-muted-foreground">
          Universal Agent Runtime
        </p>
        <h1 className="font-display text-lg font-semibold tracking-tight text-foreground">
          {error ? "Initialization Failed" : "Starting up…"}
        </h1>
      </div>

      {/* Spinner / Error icon */}
      <div className="flex items-center justify-center">
        {error ? <ErrorIcon /> : <Spinner />}
      </div>

      {/* Current status */}
      <div className="flex min-h-[1.5rem] items-center justify-center">
        {error ? (
          <p className="max-w-sm text-center font-mono text-xs text-destructive">{error}</p>
        ) : (
          <p className="font-mono text-xs text-primary animate-pulse">{current}</p>
        )}
      </div>

      {/* Step log */}
      {completedSteps.length > 0 && (
        <div
          ref={logRef}
          className="w-full max-w-sm overflow-y-auto rounded-lg bg-card px-4 py-3"
          style={{ maxHeight: "160px" }}
          aria-label="Initialization log"
          role="log"
          aria-live="polite"
        >
          <ul className="flex flex-col gap-1.5">
            {completedSteps.map((step, index) => (
              <li
                key={`${index}-${step}`}
                className="flex items-start gap-2 font-mono text-[11px] text-muted-foreground"
              >
                <span className="mt-px flex h-3.5 w-3.5 shrink-0 items-center justify-center rounded-full bg-primary/15">
                  <CheckIcon />
                </span>
                <span>{step}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Reload hint on error */}
      {error && (
        <button
          type="button"
          onClick={() => window.location.reload()}
          className="rounded-md border border-border bg-card px-4 py-2 font-mono text-xs text-foreground transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-3 focus-visible:ring-ring"
        >
          Reload page
        </button>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

interface DbProviderProps {
  children: ReactNode;
}

export function DbProvider({ children }: DbProviderProps) {
  const [value, setValue] = useState<DbContextValue>({ ready: false, db: null });
  const [steps, setSteps] = useState<string[]>([]);
  const [current, setCurrent] = useState("Opening local database…");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const onStatus: OnStatusFn = (msg) => {
      if (cancelled) return;
      performance.mark(`uar-db:${msg}`);
      setCurrent(msg);
      setSteps((prev) => [...prev, msg]);
    };

    UarDb.open(onStatus)
      .then(async (db) => {
        onStatus("Hydrating local entity graph…");
        await bootstrapDurableEntityGraph(db.getPersistenceClient());
        onStatus("Local entity graph ready");
        if (!cancelled) {
          setDbInstance(db);
          setValue({ ready: true, db });
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err);
          setError(message);
          setSteps((prev) => [...prev, `Error: ${message}`]);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  if (!value.ready) {
    return <DbLoadingScreen steps={steps} current={current} error={error} />;
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
