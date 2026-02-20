import { type FC, useCallback, useEffect, useState } from "react";
import { Eye, EyeOff, Key, Loader2, Plus, RefreshCw, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import type { UarApiKey } from "@/types";

export const AuthPage: FC = () => {
  const [keys, setKeys] = useState<UarApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [newKeyName, setNewKeyName] = useState("");
  const [createdKey, setCreatedKey] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [showKey, setShowKey] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/auth/keys");
      if (!res.ok) throw new Error(`${res.status}`);
      const data = await res.json() as { keys?: UarApiKey[]; data?: { keys?: UarApiKey[] } } | UarApiKey[];
      const list = Array.isArray(data) ? data : (data.data?.keys ?? (data as { keys?: UarApiKey[] }).keys ?? []);
      setKeys(list);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleCreate = async () => {
    setSaving(true);
    try {
      const res = await fetch("/api/auth/keys", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ name: newKeyName }) });
      if (!res.ok) throw new Error(`${res.status}`);
      const data = await res.json() as { key?: string; raw_key?: string };
      setCreatedKey(data.key ?? data.raw_key ?? null);
      setShowAdd(false);
      setNewKeyName("");
      await load();
    } catch (e) { setError((e as Error).message); } finally { setSaving(false); }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Revoke this API key?")) return;
    try {
      await fetch(`/api/auth/keys/${id}`, { method: "DELETE" });
      setKeys((prev) => prev.filter((k) => k.id !== id));
    } catch { /**/ }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">API Keys</h2>
          <p className="font-mono text-[11px] text-muted-foreground">{keys.length} keys</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => void load()} className="gap-1.5">
            <RefreshCw size={13} className={cn(loading && "animate-spin")} />Refresh
          </Button>
          <Button size="sm" onClick={() => setShowAdd(true)} className="gap-1.5"><Plus size={13} />New key</Button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-6">
        {loading && <div className="flex items-center gap-2"><Loader2 size={16} className="animate-spin text-muted-foreground" /></div>}
        {error && <p className="font-mono text-[11px] text-destructive">Error: {error}</p>}
        {createdKey && (
          <div className="mb-4 rounded-lg border border-success/40 bg-success/10 p-4">
            <p className="mb-2 font-mono text-[11px] text-success font-medium">New API key — copy it now, it won't be shown again</p>
            <div className="flex items-center gap-2">
              <code className="flex-1 font-mono text-[12px] text-foreground break-all">{showKey === "created" ? createdKey : createdKey.replace(/./g, "•")}</code>
              <Button variant="ghost" size="icon" className="h-6 w-6 shrink-0" onClick={() => setShowKey(showKey === "created" ? null : "created")}>{showKey === "created" ? <EyeOff size={12} /> : <Eye size={12} />}</Button>
              <Button variant="outline" size="sm" className="h-6 text-xs shrink-0" onClick={() => void navigator.clipboard.writeText(createdKey)}>Copy</Button>
            </div>
          </div>
        )}
        {!loading && keys.length === 0 && !createdKey && <p className="font-mono text-[11px] text-muted-foreground">No API keys configured</p>}
        <div className="flex flex-col gap-2">
          {keys.map((k) => (
            <div key={k.id} className="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
              <div className="flex size-8 items-center justify-center rounded-md bg-muted"><Key size={14} className="text-muted-foreground" /></div>
              <div className="min-w-0 flex-1">
                <p className="font-display text-[13px] font-semibold text-foreground">{k.name ?? k.id}</p>
                {k.prefix && <p className="font-mono text-[11px] text-muted-foreground">{k.prefix}•••••••</p>}
                {k.created_at && <p className="font-mono text-[10px] text-muted-foreground/60">Created {new Date(k.created_at).toLocaleDateString()}</p>}
              </div>
              <Button variant="ghost" size="icon" className="h-7 w-7 text-muted-foreground hover:text-destructive" onClick={() => void handleDelete(k.id)} aria-label={`Revoke ${k.name ?? k.id}`}>
                <Trash2 size={13} />
              </Button>
            </div>
          ))}
        </div>
      </div>
      <Dialog open={showAdd} onOpenChange={setShowAdd}>
        <DialogContent>
          <DialogHeader><DialogTitle>Create API Key</DialogTitle></DialogHeader>
          <div><label className="mb-1 block font-mono text-[11px] text-muted-foreground">Key name</label><Input value={newKeyName} onChange={(e) => setNewKeyName(e.target.value)} placeholder="My App" /></div>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setShowAdd(false)}>Cancel</Button>
            <Button onClick={() => void handleCreate()} disabled={saving || !newKeyName}>
              {saving && <Loader2 size={14} className="animate-spin" />}Create
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};
