import { type FC, useEffect, useState } from "react";
import {
    Brain,
    Check,
    ChevronLeft,
    ChevronRight,
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
import { LoadingCursor } from "@/shared/ui/configuration/loading-cursor";
import { EmptyFrame } from "@/shared/ui/configuration/empty-frame";
import { ErrorBar } from "@/shared/ui/configuration/error-bar";
import { cn } from "@/lib/utils";
import { useMemory, useMemoryStats } from "../model/use-memory";
import type { MemoryItem } from "../model/memory-types";
import { useMemoryAdmin } from "../model/use-memory-admin";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// Terminal-styled scope chips — single palette, accent variants.
const SCOPE_TONE: Record<string, string> = {
    Global: "border-[color-mix(in_srgb,var(--color-phosphor)_50%,transparent)] text-[var(--color-phosphor)]",
    Agent: "border-[color-mix(in_srgb,var(--color-phosphor)_50%,transparent)] text-[var(--color-phosphor)]",
    User: "border-[color-mix(in_srgb,var(--color-terminal-fg)_40%,transparent)] text-[var(--color-terminal-fg)]",
    Session: "border-[color-mix(in_srgb,var(--color-amber)_50%,transparent)] text-[var(--color-amber)]",
    Task: "border-[color-mix(in_srgb,var(--color-signal-red)_50%,transparent)] text-[var(--color-signal-red)]",
};

function ScopeBadge({ scope }: { scope: string }) {
    const tone = SCOPE_TONE[scope] ?? "border-[var(--color-terminal-line-strong)] text-[var(--color-terminal-fg-dim)]";
    return (
        <span className={cn("inline-flex items-center rounded border bg-transparent px-2 py-0.5 font-mono text-xs lowercase tracking-tight", tone)}>
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

    // Reads from the entity graph (hydrated by loadMemoryIntoGraph; SSE keeps fresh).
    const view = useMemory();
    const items = view.items as MemoryItem[];
    const stats = useMemoryStats();
    const admin = useMemoryAdmin();

    // Local UI state
    const [selected, setSelected] = useState<MemoryItem | null>(null);
    const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
    const [showBulkDelete, setShowBulkDelete] = useState(false);
    const [savedFlash, setSavedFlash] = useState(false);

    // Pagination
    const [page, setPage] = useState(0);
    const PAGE_SIZE = 25;

    const runLoad = async () => {
        setPage(0);
        try {
            await admin.load({ userId, agentId, searchQ, searchMode });
        } catch { /* store surfaces error */ }
    };

    useEffect(() => {
        void runLoad();
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    const deleteOneRow = async (id: string) => {
        try {
            const item = items.find((candidate) => candidate.id === id);
            if (!item) return;
            await admin.remove(item);
            if (selected?.id === id) setSelected(null);
            setSavedFlash(true);
            setTimeout(() => setSavedFlash(false), 2500);
        } catch {
            /* store surfaces error */
        } finally {
            setDeleteConfirm(null);
        }
    };

    const runBulkDelete = async () => {
        try {
            await admin.removeVisible(items, userId, agentId);
            setSelected(null);
            setSavedFlash(true);
            setTimeout(() => setSavedFlash(false), 2500);
        } catch { /* store retains graph and surfaces error */ }
    };

    const start = page * PAGE_SIZE;
    const end = start + PAGE_SIZE;
    const pageItems = items.slice(start, end);
    const totalPages = Math.max(1, Math.ceil(items.length / PAGE_SIZE));

    return (
        <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[var(--color-terminal-fg)]">
            {/* Header */}
            <div className="flex items-center justify-between border-b border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] px-6 py-4">
                <div className="flex items-center gap-3">
                    <div className="flex size-8 items-center justify-center border border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-bg)]">
                        <Brain size={16} className="text-[var(--color-phosphor)]" />
                    </div>
                    <div>
                        <h2 className="text-[20px] font-medium tracking-tight">memory browser</h2>
                        <p className="text-xs text-[var(--color-terminal-fg-dim)]">
                            {stats ? `${stats.total} total memories` : "loading stats"}
                            {admin.loading && <LoadingCursor className="ml-2" />}
                        </p>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => void runLoad()}
                        disabled={admin.loading}
                        className="gap-1.5 border border-[var(--color-terminal-line-strong)] bg-transparent text-[var(--color-terminal-fg)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_8%,transparent)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]"
                    >
                        <RefreshCw size={13} className={cn(admin.loading && "animate-spin")} />refresh
                    </Button>
                    <Button
                        size="sm"
                        onClick={() => setShowBulkDelete(true)}
                        disabled={admin.deleting || items.length === 0}
                        className="gap-1.5 border border-[var(--color-signal-red)] bg-[color-mix(in_srgb,var(--color-signal-red)_12%,transparent)] text-[var(--color-signal-red)] hover:bg-[color-mix(in_srgb,var(--color-signal-red)_18%,transparent)]"
                    >
                        <Trash2 size={13} />delete all visible
                    </Button>
                </div>
            </div>

            {/* Stats Bar */}
            {stats?.by_scope != null && Object.keys(stats.by_scope).length > 0 && (
                <div className="flex items-center gap-3 border-b border-[var(--color-terminal-line)] bg-[var(--color-terminal-bg)] px-6 py-2">
                    {Object.entries(stats.by_scope).map(([scope, count]) => (
                        <div key={scope} className="flex items-center gap-1.5">
                            <ScopeBadge scope={scope} />
                            <span className="text-xs text-[var(--color-terminal-fg-dim)]">{count}</span>
                        </div>
                    ))}
                </div>
            )}

            {/* Filters */}
            <div className="flex flex-wrap items-center gap-2 border-b border-[var(--color-terminal-line)] bg-[var(--color-terminal-bg)] px-4 py-3 sm:gap-3 sm:px-6">
                <div className="flex flex-1 min-w-[200px] max-w-xs items-center gap-2 border border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] px-3 py-1.5">
                    <Search size={13} className="shrink-0 text-[var(--color-terminal-fg-dim)]" />
                    <input
                        value={searchQ}
                        onChange={(e) => setSearchQ(e.target.value)}
                        onKeyDown={(e) => { if (e.key === "Enter") { setSearchMode(!!searchQ); void runLoad(); } }}
                        placeholder="search memories by meaning…"
                        className="flex-1 bg-transparent font-mono text-xs text-[var(--color-terminal-fg)] outline-none placeholder:text-[var(--color-terminal-fg-dim)]"
                    />
                    {searchQ && (
                        <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            className="h-6 w-6 shrink-0 text-[var(--color-terminal-fg-dim)] hover:text-[var(--color-terminal-fg)]"
                            onClick={() => { setSearchQ(""); setSearchMode(false); void runLoad(); }}
                            aria-label="Clear search"
                        >
                            ×
                        </Button>
                    )}
                </div>
                <Input
                    value={userId}
                    onChange={(e) => setUserId(e.target.value)}
                    placeholder="filter by user id"
                    className="min-w-[140px] max-w-[180px] border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] font-mono text-xs text-[var(--color-terminal-fg)] placeholder:text-[var(--color-terminal-fg-dim)]"
                />
                <Input
                    value={agentId}
                    onChange={(e) => setAgentId(e.target.value)}
                    placeholder="filter by agent id"
                    className="min-w-[140px] max-w-[180px] border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] font-mono text-xs text-[var(--color-terminal-fg)] placeholder:text-[var(--color-terminal-fg-dim)]"
                />
                <Button
                    size="sm"
                    onClick={() => { setSearchMode(!!searchQ); void runLoad(); }}
                    className="gap-1.5 border border-[var(--color-phosphor)] bg-[color-mix(in_srgb,var(--color-phosphor)_12%,transparent)] text-[var(--color-phosphor)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_18%,transparent)]"
                >
                    apply
                </Button>
            </div>

            {/* Banners */}
            {savedFlash && (
                <div className="mx-6 mt-3 flex items-center gap-2 border border-[color-mix(in_srgb,var(--color-phosphor)_40%,transparent)] bg-[color-mix(in_srgb,var(--color-phosphor)_8%,transparent)] px-4 py-2 text-xs text-[var(--color-phosphor)]">
                    <Check size={13} />deleted successfully
                </div>
            )}
            {admin.error && <ErrorBar code="MEMORY" message={admin.error} onDismiss={admin.clearError} className="mx-6 mt-3" />}

            {/* Content: table + detail panel */}
            <div className="flex flex-1 overflow-hidden">
                <div className="flex flex-1 flex-col overflow-hidden">
                    {admin.loading && items.length === 0 ? (
                        <div className="p-6"><LoadingCursor label="loading memories" /></div>
                    ) : pageItems.length === 0 ? (
                        <div className="flex flex-1 items-center justify-center">
                            <EmptyFrame
                                title={items.length === 0 ? "no memories found" : "no items on this page"}
                                hint={items.length === 0 ? "adjust filters above or wait for agents to write memories" : "navigate back to a populated page"}
                            />
                        </div>
                    ) : (
                        <>
                            <div className="flex-1 overflow-auto">
                                <table className="min-w-[700px] w-full text-left">
                                    <thead className="sticky top-0 border-b border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)]">
                                        <tr>
                                            {["scope", "content", "agent", "user", "importance", "created", ""].map((h, i) => (
                                                <th key={i} className="px-4 py-2 font-mono text-xs font-medium uppercase tracking-wider text-[var(--color-terminal-fg-dim)]">{h}</th>
                                            ))}
                                        </tr>
                                    </thead>
                                    <tbody className="divide-y divide-[var(--color-terminal-line)]">
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
                                                        "cursor-pointer transition-colors duration-[160ms] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-[-2px] focus-visible:outline-[var(--color-ember)]",
                                                        isRowSelected ? "bg-[color-mix(in_srgb,var(--color-phosphor)_8%,transparent)]" : "hover:bg-[color-mix(in_srgb,var(--color-terminal-fg)_4%,transparent)]",
                                                    )}
                                                >
                                                    <td className="px-4 py-2.5"><ScopeBadge scope={m.scope} /></td>
                                                    <td className="max-w-xs px-4 py-2.5 text-sm text-[var(--color-terminal-fg)]">{truncate(m.content, 80)}</td>
                                                    <td className="px-4 py-2.5 text-xs text-[var(--color-terminal-fg-dim)]">{m.agent_id ?? "—"}</td>
                                                    <td className="px-4 py-2.5 text-xs text-[var(--color-terminal-fg-dim)]">{m.user_id ?? "—"}</td>
                                                    <td className="px-4 py-2.5 text-xs text-[var(--color-terminal-fg-dim)]">{m.importance.toFixed(2)}</td>
                                                    <td className="whitespace-nowrap px-4 py-2.5 text-xs text-[var(--color-terminal-fg-dim)]">{new Date(m.created_at).toLocaleDateString()}</td>
                                                    <td className="px-4 py-2.5">
                                                        {deleteConfirm === m.id ? (
                                                            <div className="flex items-center gap-1">
                                                                <Button
                                                                    size="sm"
                                                                    className="h-6 border border-[var(--color-signal-red)] bg-[color-mix(in_srgb,var(--color-signal-red)_12%,transparent)] px-2 text-xs text-[var(--color-signal-red)]"
                                                                    disabled={admin.deleting}
                                                                    onClick={(e) => { e.stopPropagation(); void deleteOneRow(m.id); }}
                                                                >
                                                                    confirm
                                                                </Button>
                                                                <Button size="sm" variant="ghost" className="h-6 px-2 text-xs text-[var(--color-terminal-fg-dim)]" onClick={(e) => { e.stopPropagation(); setDeleteConfirm(null); }}>cancel</Button>
                                                            </div>
                                                        ) : (
                                                            <Button
                                                                size="icon"
                                                                variant="ghost"
                                                                className="h-7 w-7 text-[var(--color-terminal-fg-dim)] hover:text-[var(--color-signal-red)]"
                                                                onClick={(e) => { e.stopPropagation(); setDeleteConfirm(m.id); }}
                                                                aria-label="Delete memory"
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

                            <div className="flex items-center justify-between border-t border-[var(--color-terminal-line)] px-4 py-2">
                                <span className="text-xs text-[var(--color-terminal-fg-dim)]">{start + 1}–{Math.min(end, items.length)} of {items.length}</span>
                                <div className="flex items-center gap-1">
                                    <Button size="icon" variant="ghost" className="h-7 w-7 text-[var(--color-terminal-fg-dim)]" disabled={page === 0} onClick={() => setPage((p) => p - 1)} aria-label="Previous page">
                                        <ChevronLeft size={13} />
                                    </Button>
                                    <span className="px-2 text-xs text-[var(--color-terminal-fg-dim)]">{page + 1} / {totalPages}</span>
                                    <Button size="icon" variant="ghost" className="h-7 w-7 text-[var(--color-terminal-fg-dim)]" disabled={page >= totalPages - 1} onClick={() => setPage((p) => p + 1)} aria-label="Next page">
                                        <ChevronRight size={13} />
                                    </Button>
                                </div>
                            </div>
                        </>
                    )}
                </div>

                {/* Detail panel */}
                {selected && (
                    <div className="w-80 shrink-0 overflow-y-auto border-l border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] p-5">
                        <div className="mb-4 flex items-start justify-between">
                            <p className="font-mono text-xs font-medium uppercase tracking-wider text-[var(--color-terminal-fg-dim)]">memory detail</p>
                            <Button type="button" variant="ghost" size="icon" className="h-6 w-6 text-[var(--color-terminal-fg-dim)] hover:text-[var(--color-terminal-fg)]" onClick={() => setSelected(null)} aria-label="Close detail panel">×</Button>
                        </div>
                        <div className="space-y-3">
                            <div>
                                <p className="mb-1 font-mono text-xs uppercase text-[var(--color-terminal-fg-dim)]">scope</p>
                                <ScopeBadge scope={selected.scope} />
                            </div>
                            <div>
                                <p className="mb-1 font-mono text-xs uppercase text-[var(--color-terminal-fg-dim)]">content</p>
                                <p className="whitespace-pre-wrap text-sm text-[var(--color-terminal-fg)]">{selected.content}</p>
                            </div>
                            {selected.categories.length > 0 && (
                                <div>
                                    <p className="mb-1 font-mono text-xs uppercase text-[var(--color-terminal-fg-dim)]">categories</p>
                                    <div className="flex flex-wrap gap-1">
                                        {selected.categories.map((c) => (
                                            <span key={c} className="rounded border border-[var(--color-terminal-line-strong)] bg-transparent px-2 py-0.5 font-mono text-xs text-[var(--color-terminal-fg-dim)]">{c}</span>
                                        ))}
                                    </div>
                                </div>
                            )}
                            {selected.agent_id && <InfoRow label="agent id" value={selected.agent_id} />}
                            {selected.user_id && <InfoRow label="user id" value={selected.user_id} />}
                            {selected.session_id && <InfoRow label="session id" value={selected.session_id} />}
                            <InfoRow label="importance" value={selected.importance.toFixed(3)} />
                            <InfoRow label="type" value={selected.memory_type} />
                            <InfoRow label="created" value={new Date(selected.created_at).toLocaleString()} />
                            <InfoRow label="id" value={selected.id} mono />
                        </div>
                        <div className="mt-6">
                            {deleteConfirm === selected.id ? (
                                <div className="flex gap-2">
                                    <Button
                                        size="sm"
                                        className="flex-1 border border-[var(--color-signal-red)] bg-[color-mix(in_srgb,var(--color-signal-red)_12%,transparent)] text-[var(--color-signal-red)]"
                                        disabled={admin.deleting}
                                        onClick={() => void deleteOneRow(selected.id)}
                                    >
                                        {admin.deleting ? <LoadingCursor /> : "confirm delete"}
                                    </Button>
                                    <Button variant="ghost" size="sm" className="border border-[var(--color-terminal-line-strong)] text-[var(--color-terminal-fg)]" onClick={() => setDeleteConfirm(null)}>cancel</Button>
                                </div>
                            ) : (
                                <Button
                                    size="sm"
                                    className="w-full gap-1.5 border border-[var(--color-signal-red)] bg-[color-mix(in_srgb,var(--color-signal-red)_12%,transparent)] text-[var(--color-signal-red)] hover:bg-[color-mix(in_srgb,var(--color-signal-red)_18%,transparent)]"
                                    onClick={() => setDeleteConfirm(selected.id)}
                                >
                                    <Trash2 size={13} />delete memory
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
                        <AlertDialogCancel disabled={admin.deleting}>Cancel</AlertDialogCancel>
                        <AlertDialogAction
                            onClick={(e) => { e.preventDefault(); setShowBulkDelete(false); void runBulkDelete(); }}
                            disabled={admin.deleting}
                            className="border border-[var(--color-signal-red)] bg-[color-mix(in_srgb,var(--color-signal-red)_12%,transparent)] text-[var(--color-signal-red)] hover:bg-[color-mix(in_srgb,var(--color-signal-red)_18%,transparent)]"
                        >
                            {admin.deleting ? <LoadingCursor /> : "delete all"}
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
            <p className="mb-0.5 font-mono text-xs uppercase text-[var(--color-terminal-fg-dim)]">{label}</p>
            <p className={cn("break-all text-xs text-[var(--color-terminal-fg)]", mono && "font-mono")}>{value}</p>
        </div>
    );
}
