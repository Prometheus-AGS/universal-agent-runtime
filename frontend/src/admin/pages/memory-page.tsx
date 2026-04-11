import { type FC, useEffect, useState } from "react";
import {
    Brain,
    Check,
    ChevronLeft,
    ChevronRight,
    Loader2,
    RefreshCw,
    Search,
    Trash2,
} from "lucide-react";
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminEmptyInline, AdminError, AdminTableSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import { useMemoryAdmin } from "@/hooks/use-memory-admin";
import type { MemoryItem } from "@/services/memory-api";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const SCOPE_COLORS: Record<string, string> = {
    Global: "bg-purple-500/15 text-purple-400 border-purple-500/30",
    Agent: "bg-blue-500/15 text-blue-400 border-blue-500/30",
    User: "bg-green-500/15 text-green-400 border-green-500/30",
    Session: "bg-amber-500/15 text-amber-400 border-amber-500/30",
    Task: "bg-rose-500/15 text-rose-400 border-rose-500/30",
};

function ScopeBadge({ scope }: { scope: string }) {
    const cls = SCOPE_COLORS[scope] ?? "bg-muted text-muted-foreground border-border";
    return (
        <span className={cn("inline-flex items-center rounded-full border px-2 py-0.5 font-mono text-xs font-medium", cls)}>
            {scope}
        </span>
    );
}

function truncate(s: string, n: number) {
    return s.length > n ? `${s.slice(0, n)}…` : s;
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export const MemoryPage: FC = () => {
    // Filter state
    const [userId, setUserId] = useState("");
    const [agentId, setAgentId] = useState("");
    const [searchQ, setSearchQ] = useState("");
    const [searchMode, setSearchMode] = useState(false);

    const {
        items,
        stats,
        loading,
        error,
        deleting,
        load,
        loadStats,
        deleteOne,
        bulkDelete,
    } = useMemoryAdmin();

    // Selection / detail
    const [selected, setSelected] = useState<MemoryItem | null>(null);
    const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
    const [showBulkDelete, setShowBulkDelete] = useState(false);
    const [savedFlash, setSavedFlash] = useState(false);

    // Pagination
    const [page, setPage] = useState(0);
    const PAGE_SIZE = 25;

    const runLoad = () => {
        setPage(0);
        void load({ userId, agentId, searchQ, searchMode });
    };

    useEffect(() => {
        void load({ userId: "", agentId: "", searchQ: "", searchMode: false });
        void loadStats();
    }, [load, loadStats]);

    const deleteOneRow = async (id: string) => {
        try {
            await deleteOne(id);
            if (selected?.id === id) setSelected(null);
            setSavedFlash(true);
            setTimeout(() => setSavedFlash(false), 2500);
        } catch {
            /* store sets error */
        } finally {
            setDeleteConfirm(null);
        }
    };

    const runBulkDelete = async () => {
        try {
            await bulkDelete(userId, agentId);
            setSelected(null);
            setSavedFlash(true);
            setTimeout(() => setSavedFlash(false), 2500);
        } catch {
            /* store sets error */
        }
    };

    // -----------------------------------------------------------------------
    // Pagination slice
    // -----------------------------------------------------------------------
    const start = page * PAGE_SIZE;
    const end = start + PAGE_SIZE;
    const pageItems = items.slice(start, end);
    const totalPages = Math.max(1, Math.ceil(items.length / PAGE_SIZE));

    // -----------------------------------------------------------------------
    // Render
    // -----------------------------------------------------------------------

    return (
        <div className="flex flex-1 flex-col overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
                <div className="flex items-center gap-3">
                    <div className="flex size-8 items-center justify-center rounded-lg bg-primary/15">
                        <Brain size={16} className="text-primary" />
                    </div>
                    <div>
                        <h2 className="font-display text-lg font-semibold text-foreground">Memory Browser</h2>
                        <p className="font-mono text-xs text-muted-foreground">
                            {stats ? `${stats.total} total memories` : "Loading stats…"}
                        </p>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <Button variant="outline" size="sm" onClick={() => runLoad()} disabled={loading} className="gap-1.5">
                        <RefreshCw size={13} className={cn(loading && "animate-spin")} />
                        Refresh
                    </Button>
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => setShowBulkDelete(true)}
                        disabled={deleting || items.length === 0}
                        className="gap-1.5"
                    >
                        <Trash2 size={13} />
                        Delete All Visible
                    </Button>
                </div>
            </div>

            {/* Stats Bar */}
            {stats?.by_scope != null && Object.keys(stats.by_scope).length > 0 && (
                <div className="flex items-center gap-3 border-b border-border bg-muted/30 px-6 py-2">
                    {Object.entries(stats.by_scope).map(([scope, count]) => (
                        <div key={scope} className="flex items-center gap-1.5">
                            <ScopeBadge scope={scope} />
                            <span className="font-mono text-xs text-muted-foreground">{count}</span>
                        </div>
                    ))}
                </div>
            )}

            {/* Filters */}
            <div className="flex flex-wrap items-center gap-2 border-b border-border px-4 py-3 sm:gap-3 sm:px-6">
                <div className="flex items-center gap-2 rounded-lg border border-border bg-card px-3 py-1.5 flex-1 min-w-[200px] max-w-xs">
                    <Search size={13} className="shrink-0 text-muted-foreground" />
                    <input
                        value={searchQ}
                        onChange={(e) => setSearchQ(e.target.value)}
                        onKeyDown={(e) => { if (e.key === "Enter") { setSearchMode(!!searchQ); runLoad(); } }}
                        placeholder="Search memories by meaning..."
                        className="flex-1 bg-transparent font-mono text-xs outline-none placeholder:text-muted-foreground/50"
                    />
                    {searchQ && (
                        <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground"
                            onClick={() => { setSearchQ(""); setSearchMode(false); runLoad(); }}
                            aria-label="Clear search"
                        >
                            ×
                        </Button>
                    )}
                </div>
                <Input
                    value={userId}
                    onChange={(e) => setUserId(e.target.value)}
                    placeholder="Filter by User ID"
                    className="min-w-[140px] max-w-[180px] font-mono text-xs"
                />
                <Input
                    value={agentId}
                    onChange={(e) => setAgentId(e.target.value)}
                    placeholder="Filter by Agent ID"
                    className="min-w-[140px] max-w-[180px] font-mono text-xs"
                />
                <Button size="sm" onClick={() => { setSearchMode(!!searchQ); runLoad(); }} className="gap-1.5">
                    Apply
                </Button>
            </div>

            {/* Banners */}
            {savedFlash && (
                <div className="mx-6 mt-3 flex items-center gap-2 rounded-lg border border-success/40 bg-success/10 px-4 py-2">
                    <Check size={13} className="text-success" />
                    <span className="font-mono text-xs text-success">Deleted successfully</span>
                </div>
            )}
            {error && (
                <div className="mx-6 mt-3"><AdminError error={error} /></div>
            )}

            {/* Content: table + detail panel */}
            <div className="flex flex-1 overflow-hidden">
                {/* Table */}
                <div className="flex flex-1 flex-col overflow-hidden">
                    {loading && items.length === 0 ? (
                        <div className="p-6">
                            <AdminTableSkeleton rows={6} cols={6} />
                        </div>
                    ) : pageItems.length === 0 ? (
                        <div className="flex flex-1 items-center justify-center">
                            <AdminEmptyInline>
                                {items.length === 0 ? "No memories found" : "No items on this page"}
                            </AdminEmptyInline>
                        </div>
                    ) : (
                        <>
                            <div className="flex-1 overflow-auto">
                                <table className="min-w-[700px] w-full text-left">
                                    <thead className="sticky top-0 bg-card border-b border-border">
                                        <tr>
                                            <th className="px-4 py-2 font-mono text-xs font-semibold uppercase tracking-wider text-muted-foreground">Scope</th>
                                            <th className="px-4 py-2 font-mono text-xs font-semibold uppercase tracking-wider text-muted-foreground">Content</th>
                                            <th className="px-4 py-2 font-mono text-xs font-semibold uppercase tracking-wider text-muted-foreground">Agent</th>
                                            <th className="px-4 py-2 font-mono text-xs font-semibold uppercase tracking-wider text-muted-foreground">User</th>
                                            <th className="px-4 py-2 font-mono text-xs font-semibold uppercase tracking-wider text-muted-foreground">Importance</th>
                                            <th className="px-4 py-2 font-mono text-xs font-semibold uppercase tracking-wider text-muted-foreground">Created</th>
                                            <th className="px-4 py-2" />
                                        </tr>
                                    </thead>
                                    <tbody className="divide-y divide-border">
                                        {pageItems.map((m) => {
                                            const isRowSelected = selected?.id === m.id;
                                            return (
                                            <tr
                                                key={m.id}
                                                onClick={() => setSelected(isRowSelected ? null : m)}
                                                onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); setSelected(isRowSelected ? null : m); } }}
                                                tabIndex={0}
                                                aria-current={isRowSelected ? "true" : undefined}
                                                className={cn(
                                                    "cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/50",
                                                    isRowSelected ? "bg-accent" : "hover:bg-muted/40"
                                                )}
                                            >
                                                <td className="px-4 py-2.5"><ScopeBadge scope={m.scope} /></td>
                                                <td className="px-4 py-2.5 font-body text-sm text-foreground max-w-xs">
                                                    {truncate(m.content, 80)}
                                                </td>
                                                <td className="px-4 py-2.5 font-mono text-xs text-muted-foreground">{m.agent_id ?? "—"}</td>
                                                <td className="px-4 py-2.5 font-mono text-xs text-muted-foreground">{m.user_id ?? "—"}</td>
                                                <td className="px-4 py-2.5 font-mono text-xs text-muted-foreground">
                                                    {m.importance.toFixed(2)}
                                                </td>
                                                <td className="px-4 py-2.5 font-mono text-xs text-muted-foreground whitespace-nowrap">
                                                    {new Date(m.created_at).toLocaleDateString()}
                                                </td>
                                                <td className="px-4 py-2.5">
                                                    {deleteConfirm === m.id ? (
                                                        <div className="flex items-center gap-1">
                                                            <Button
                                                                size="sm"
                                                                variant="destructive"
                                                                className="h-6 px-2 text-xs"
                                                                disabled={deleting}
                                                                onClick={(e) => { e.stopPropagation(); void deleteOneRow(m.id); }}
                                                            >
                                                                Confirm
                                                            </Button>
                                                            <Button
                                                                size="sm"
                                                                variant="ghost"
                                                                className="h-6 px-2 text-xs"
                                                                onClick={(e) => { e.stopPropagation(); setDeleteConfirm(null); }}
                                                            >
                                                                Cancel
                                                            </Button>
                                                        </div>
                                                    ) : (
                                                        <Button
                                                            size="icon"
                                                            variant="ghost"
                                                            className="h-7 w-7 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive"
                                                            onClick={(e) => { e.stopPropagation(); setDeleteConfirm(m.id); }}
                                                        >
                                                            <Trash2 size={13} />
                                                        </Button>
                                                    )}
                                                </td>
                                            </tr>
                                            );
                                        })}
                                    </tbody>
                                </table>
                            </div>

                            {/* Pagination */}
                            <div className="flex items-center justify-between border-t border-border px-4 py-2">
                                <span className="font-mono text-xs text-muted-foreground">
                                    {start + 1}–{Math.min(end, items.length)} of {items.length}
                                </span>
                                <div className="flex items-center gap-1">
                                    <Button size="icon" variant="ghost" className="h-7 w-7" disabled={page === 0} onClick={() => setPage((p) => p - 1)} aria-label="Previous page">
                                        <ChevronLeft size={13} />
                                    </Button>
                                    <span className="font-mono text-xs text-muted-foreground px-2">
                                        {page + 1} / {totalPages}
                                    </span>
                                    <Button size="icon" variant="ghost" className="h-7 w-7" disabled={page >= totalPages - 1} onClick={() => setPage((p) => p + 1)} aria-label="Next page">
                                        <ChevronRight size={13} />
                                    </Button>
                                </div>
                            </div>
                        </>
                    )}
                </div>

                {/* Detail panel */}
                {selected && (
                    <div className="w-80 shrink-0 overflow-y-auto border-l border-border bg-card p-5">
                        <div className="flex items-start justify-between mb-4">
                            <p className="font-mono text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                                Memory Detail
                            </p>
                            <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                className="h-6 w-6 text-muted-foreground hover:text-foreground"
                                onClick={() => setSelected(null)}
                                aria-label="Close detail panel"
                            >
                                ×
                            </Button>
                        </div>
                        <div className="space-y-3">
                            <div>
                                <p className="font-mono text-xs text-muted-foreground uppercase mb-1">Scope</p>
                                <ScopeBadge scope={selected.scope} />
                            </div>
                            <div>
                                <p className="font-mono text-xs text-muted-foreground uppercase mb-1">Content</p>
                                <p className="font-body text-sm text-foreground whitespace-pre-wrap">{selected.content}</p>
                            </div>
                            {selected.categories.length > 0 && (
                                <div>
                                    <p className="font-mono text-xs text-muted-foreground uppercase mb-1">Categories</p>
                                    <div className="flex flex-wrap gap-1">
                                        {selected.categories.map((c) => (
                                            <span key={c} className="rounded-full bg-muted px-2 py-0.5 font-mono text-xs text-muted-foreground">{c}</span>
                                        ))}
                                    </div>
                                </div>
                            )}
                            {selected.agent_id && <InfoRow label="Agent ID" value={selected.agent_id} />}
                            {selected.user_id && <InfoRow label="User ID" value={selected.user_id} />}
                            {selected.session_id && <InfoRow label="Session ID" value={selected.session_id} />}
                            <InfoRow label="Importance" value={selected.importance.toFixed(3)} />
                            <InfoRow label="Type" value={selected.memory_type} />
                            <InfoRow label="Created" value={new Date(selected.created_at).toLocaleString()} />
                            <InfoRow label="ID" value={selected.id} mono />
                        </div>
                        <div className="mt-6">
                            {deleteConfirm === selected.id ? (
                                <div className="flex gap-2">
                                    <Button
                                        variant="destructive"
                                        size="sm"
                                        className="flex-1"
                                        disabled={deleting}
                                        onClick={() => void deleteOneRow(selected.id)}
                                    >
                                        {deleting ? <Loader2 size={13} className="animate-spin" /> : "Confirm Delete"}
                                    </Button>
                                    <Button variant="outline" size="sm" onClick={() => setDeleteConfirm(null)}>Cancel</Button>
                                </div>
                            ) : (
                                <Button
                                    variant="destructive"
                                    size="sm"
                                    className="w-full gap-1.5"
                                    onClick={() => setDeleteConfirm(selected.id)}
                                >
                                    <Trash2 size={13} />
                                    Delete Memory
                                </Button>
                            )}
                        </div>
                    </div>
                )}
            </div>
            <AlertDialog open={showBulkDelete} onOpenChange={setShowBulkDelete}>
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>Delete all visible memories?</AlertDialogTitle>
                        <AlertDialogDescription>
                            This will permanently delete {items.length} memory {items.length === 1 ? "item" : "items"} matching the current filters. This action cannot be undone.
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
                        <AlertDialogAction
                            onClick={(e) => { e.preventDefault(); setShowBulkDelete(false); void runBulkDelete(); }}
                            disabled={deleting}
                            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
                        >
                            {deleting ? <Loader2 size={12} className="animate-spin" /> : "Delete All"}
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </div>
    );
};

function InfoRow({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
    return (
        <div>
            <p className="font-mono text-xs text-muted-foreground uppercase mb-0.5">{label}</p>
            <p className={cn("text-xs text-foreground break-all", mono ? "font-mono" : "font-body")}>{value}</p>
        </div>
    );
}
