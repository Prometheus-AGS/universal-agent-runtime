/**
 * Shared loading, error, and empty state components for admin pages.
 *
 * These enforce consistent patterns across all admin sections:
 * - Loading: Skeleton screens for initial load, spinner for refresh
 * - Error: Inline friendly message
 * - Empty: Centered icon + title + description + optional CTA
 */

import { type FC, type ReactNode } from "react";
import { Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { friendlyError } from "@/lib/utils";

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/** Skeleton rows for list/table loading states. */
export const AdminListSkeleton: FC<{ rows?: number }> = ({ rows = 5 }) => (
  <div className="flex flex-col gap-2">
    {Array.from({ length: rows }).map((_, i) => (
      <div key={i} className="flex items-center gap-3 rounded-lg border border-border bg-card p-4">
        <Skeleton className="size-9 shrink-0 rounded-md" />
        <div className="flex-1 space-y-2">
          <Skeleton className="h-3.5 w-1/3" />
          <Skeleton className="h-3 w-1/2" />
        </div>
      </div>
    ))}
  </div>
);

/** Skeleton cards for grid loading states. */
export const AdminGridSkeleton: FC<{ count?: number }> = ({ count = 6 }) => (
  <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
    {Array.from({ length: count }).map((_, i) => (
      <div key={i} className="flex flex-col rounded-lg border border-border bg-card p-4">
        <div className="mb-3 flex items-start justify-between">
          <Skeleton className="size-8 rounded-md" />
          <Skeleton className="size-6 rounded" />
        </div>
        <Skeleton className="h-4 w-2/3" />
        <Skeleton className="mt-2 h-3 w-full" />
        <Skeleton className="mt-2 h-3 w-1/3" />
      </div>
    ))}
  </div>
);

/** Skeleton table for tabular loading states. */
export const AdminTableSkeleton: FC<{ rows?: number; cols?: number }> = ({
  rows = 6,
  cols = 5,
}) => (
  <div className="space-y-1.5">
    {/* Header row */}
    <div className="flex gap-4 px-4 py-2">
      {Array.from({ length: cols }).map((_, i) => (
        <Skeleton key={i} className="h-3 flex-1" />
      ))}
    </div>
    {/* Data rows */}
    {Array.from({ length: rows }).map((_, i) => (
      <div key={i} className="flex items-center gap-4 rounded-lg border border-border/50 bg-card px-4 py-3">
        {Array.from({ length: cols }).map((_, j) => (
          <Skeleton key={j} className="h-3.5 flex-1" />
        ))}
      </div>
    ))}
  </div>
);

/** Sidebar list skeleton for master/detail pages. */
export const AdminSidebarSkeleton: FC<{ rows?: number }> = ({ rows = 6 }) => (
  <div className="flex flex-col gap-0.5 py-2">
    {Array.from({ length: rows }).map((_, i) => (
      <div key={i} className="flex items-center gap-3 px-4 py-2.5">
        <Skeleton className="size-8 shrink-0 rounded-md" />
        <div className="flex-1 space-y-1.5">
          <Skeleton className="h-3.5 w-2/3" />
          <Skeleton className="h-2.5 w-1/3" />
        </div>
      </div>
    ))}
  </div>
);

/** Inline refresh spinner — used when data is already loaded and user triggers refresh. */
export const AdminRefreshSpinner: FC = () => (
  <div className="flex items-center gap-2 py-8 justify-center">
    <Loader2 size={16} className="animate-spin text-muted-foreground" />
    <span className="font-mono text-xs text-muted-foreground">Loading...</span>
  </div>
);

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/** Standard inline error banner for admin pages. */
export const AdminError: FC<{ error: string | null }> = ({ error }) => {
  if (!error) return null;
  return (
    <p className="font-mono text-xs text-destructive">
      {friendlyError(error)}
    </p>
  );
};

// ---------------------------------------------------------------------------
// Empty State
// ---------------------------------------------------------------------------

interface AdminEmptyStateProps {
  icon: FC<{ size?: number; className?: string }>;
  title: string;
  description: string;
  action?: {
    label: string;
    icon?: FC<{ size?: number; className?: string }>;
    onClick: () => void;
  };
}

/** Centered empty state with icon, title, description, and optional CTA. */
export const AdminEmptyState: FC<AdminEmptyStateProps> = ({
  icon: Icon,
  title,
  description,
  action,
}) => (
  <div className="flex flex-col items-center gap-3 py-16 text-center">
    <div className="flex size-12 items-center justify-center rounded-xl bg-primary/10">
      <Icon size={20} className="text-primary" />
    </div>
    <p className="font-display text-sm font-medium text-foreground">{title}</p>
    <p className="max-w-xs font-body text-xs text-muted-foreground">
      {description}
    </p>
    {action && (
      <Button
        size="sm"
        onClick={action.onClick}
        className="mt-2 gap-1.5"
      >
        {action.icon && <action.icon size={13} />}
        {action.label}
      </Button>
    )}
  </div>
);

/** Minimal inline empty state — for constrained spaces like sidebars. */
export const AdminEmptyInline: FC<{ children: ReactNode }> = ({ children }) => (
  <p className="px-4 py-4 font-mono text-xs text-muted-foreground">
    {children}
  </p>
);
