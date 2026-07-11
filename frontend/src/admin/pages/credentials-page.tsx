import { type FC, useEffect, useState } from "react";
import { KeyRound, Loader2, Plus, RefreshCw, ShieldOff, Trash2, UserX } from "lucide-react";
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
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { AdminEmptyInline, AdminError, AdminSidebarSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import { useCredentials } from "@/hooks/use-credentials";

/**
 * Render the masked display form of a stored key hint (last-4).
 * Pure + exported for unit testing.
 */
export function maskedKey(hint: string): string {
  const tail = hint.trim();
  if (!tail) return "••••";
  return `••••${tail}`;
}

/** Format an ISO timestamp as a short, locale-stable label. Pure + exported. */
export function formatUpdated(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export const CredentialsPage: FC = () => {
  const { state, credentials, loading, error, saving, removing, load, save: saveCredential, remove } =
    useCredentials();

  useEffect(() => {
    void load();
  }, [load]);

  // ── Add / rotate dialog ──────────────────────────────────────────────────
  const [dialogOpen, setDialogOpen] = useState(false);
  const [rotateTarget, setRotateTarget] = useState<string | null>(null);
  const [form, setForm] = useState({ provider_id: "", api_key: "" });

  const openAdd = () => {
    setRotateTarget(null);
    setForm({ provider_id: "", api_key: "" });
    setDialogOpen(true);
  };
  const openRotate = (providerId: string) => {
    setRotateTarget(providerId);
    setForm({ provider_id: providerId, api_key: "" });
    setDialogOpen(true);
  };
  const closeDialog = () => {
    setDialogOpen(false);
    // Never retain the raw key in component state after the dialog closes.
    setForm({ provider_id: "", api_key: "" });
  };

  const save = async () => {
    const provider = form.provider_id.trim();
    const key = form.api_key;
    if (!provider || !key) return;
    if (await saveCredential(provider, key)) {
      closeDialog();
    }
  };

  // ── Delete confirm ───────────────────────────────────────────────────────
  const [removeTarget, setRemoveTarget] = useState<string | null>(null);
  const confirmDelete = async () => {
    if (!removeTarget) return;
    if (await remove(removeTarget)) {
      setRemoveTarget(null);
    }
  };

  const serviceActive = state === "ok";

  return (
    <div className="flex flex-1 flex-col overflow-hidden" data-testid="credentials-page">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div>
          <h2
            className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground"
            data-testid="credentials-heading"
          >
            Credentials
          </h2>
          <p className="font-mono text-xs text-muted-foreground">
            Your encrypted provider API keys · {credentials.length} stored
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6"
            onClick={() => void load()}
            aria-label="Refresh"
          >
            <RefreshCw size={12} className={cn(loading && "animate-spin")} />
          </Button>
          <Button
            size="sm"
            className="h-7 text-xs"
            onClick={openAdd}
            disabled={!serviceActive}
            data-testid="credentials-add"
          >
            <Plus size={12} />
            Add key
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {loading && <AdminSidebarSkeleton rows={4} />}
        {error && <AdminError error={error} />}

        {/* Service disabled (no CREDENTIAL_ENCRYPTION_KEY on the server). */}
        {!loading && state === "disabled" && (
          <StatusPanel
            icon={<ShieldOff size={20} className="text-amber-500" />}
            title="Multi-tenant credentials are disabled"
            body="This server was started without CREDENTIAL_ENCRYPTION_KEY, so per-user provider keys cannot be stored. The runtime uses the operator's environment/config keys instead."
          />
        )}

        {/* Not authenticated. */}
        {!loading && state === "unauthorized" && (
          <StatusPanel
            icon={<UserX size={20} className="text-muted-foreground" />}
            title="Sign in to manage credentials"
            body="Your provider keys are scoped to your account. Authenticate to view, add, or remove them."
          />
        )}

        {/* Active but empty. */}
        {!loading && serviceActive && credentials.length === 0 && !error && (
          <AdminEmptyInline>
            No provider keys yet. Click <strong>Add key</strong> to store one — it is encrypted at rest
            and only its last 4 characters are ever shown.
          </AdminEmptyInline>
        )}

        {/* Credential list. */}
        {!loading && serviceActive && credentials.length > 0 && (
          <div className="flex flex-col gap-1" data-testid="credentials-list">
            {credentials.map((c) => (
              <div
                key={c.provider_id}
                className="flex items-center gap-3 rounded-md border border-border/50 bg-card px-3 py-2.5"
                data-testid={`credential-row-${c.provider_id}`}
              >
                <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted">
                  <KeyRound size={14} className="text-muted-foreground" />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate font-display text-sm font-semibold text-foreground">
                    {c.provider_id}
                  </p>
                  <p className="font-mono text-xs text-muted-foreground">
                    {maskedKey(c.api_key_hint)} · updated {formatUpdated(c.updated_at)}
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  className="h-7 text-xs"
                  onClick={() => openRotate(c.provider_id)}
                >
                  Rotate
                </Button>
                <Button
                  variant="outline"
                  size="icon"
                  className="h-7 w-7 border-destructive/40 text-destructive hover:bg-destructive/10"
                  onClick={() => setRemoveTarget(c.provider_id)}
                  aria-label={`Delete ${c.provider_id} key`}
                >
                  <Trash2 size={13} />
                </Button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Add / rotate dialog */}
      <Dialog open={dialogOpen} onOpenChange={(open) => (open ? setDialogOpen(true) : closeDialog())}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{rotateTarget ? `Rotate ${rotateTarget} key` : "Add provider key"}</DialogTitle>
            <DialogDescription className="font-body text-xs">
              The raw key is encrypted server-side and never displayed again — only its last 4
              characters are shown afterward.
            </DialogDescription>
          </DialogHeader>
          <div className="flex flex-col gap-3">
            <div>
              <Label htmlFor="cred-provider" className="mb-1 block font-mono text-xs text-muted-foreground">
                Provider ID
              </Label>
              <Input
                id="cred-provider"
                list="cred-provider-suggestions"
                value={form.provider_id}
                disabled={rotateTarget !== null}
                onChange={(e) => setForm({ ...form, provider_id: e.target.value })}
                placeholder="openai"
                autoComplete="off"
              />
              <datalist id="cred-provider-suggestions">
                {["openai", "anthropic", "groq", "google", "mistral", "openrouter"].map((p) => (
                  <option key={p} value={p} />
                ))}
              </datalist>
            </div>
            <div>
              <Label htmlFor="cred-key" className="mb-1 block font-mono text-xs text-muted-foreground">
                API Key
              </Label>
              <Input
                id="cred-key"
                type="password"
                autoComplete="off"
                value={form.api_key}
                onChange={(e) => setForm({ ...form, api_key: e.target.value })}
                placeholder="sk-…"
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="ghost" onClick={closeDialog}>
              Cancel
            </Button>
            <Button
              onClick={() => void save()}
              disabled={saving || !form.provider_id.trim() || !form.api_key}
              data-testid="credentials-save"
            >
              {saving && <Loader2 size={14} className="animate-spin" />}
              {rotateTarget ? "Rotate" : "Save"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete confirm */}
      <AlertDialog open={removeTarget !== null} onOpenChange={(open) => !open && setRemoveTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete this provider key?</AlertDialogTitle>
            <AlertDialogDescription>
              The stored key for <strong>{removeTarget}</strong> will be permanently removed. Requests
              for this provider will fall back to the operator's environment/config key.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={removing}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => {
                e.preventDefault();
                void confirmDelete();
              }}
              disabled={removing}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {removing ? <Loader2 size={12} className="animate-spin" /> : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

function StatusPanel({
  icon,
  title,
  body,
}: {
  icon: React.ReactNode;
  title: string;
  body: string;
}) {
  return (
    <div className="flex flex-1 items-center justify-center p-6">
      <div className="max-w-sm text-center">
        <div className="mx-auto mb-4 flex size-12 items-center justify-center rounded-xl bg-muted">
          {icon}
        </div>
        <p className="font-display text-sm font-semibold text-foreground">{title}</p>
        <p className="mt-2 font-body text-xs leading-relaxed text-muted-foreground">{body}</p>
      </div>
    </div>
  );
}
