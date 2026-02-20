import { type FC, useCallback, useEffect, useState } from "react";
import { BookOpen, Loader2, Plus, RefreshCw, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import type { UarKnowledgeBase } from "@/types";

export const KnowledgePage: FC = () => {
  const [bases, setBases] = useState<UarKnowledgeBase[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({ name: "", description: "" });
  const [saving, setSaving] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/knowledge");
      if (!res.ok) throw new Error(`${res.status}`);
      const data = await res.json() as { knowledge_bases?: UarKnowledgeBase[]; data?: { knowledge_bases?: UarKnowledgeBase[] } } | UarKnowledgeBase[];
      const list = Array.isArray(data) ? data : (data.data?.knowledge_bases ?? (data as { knowledge_bases?: UarKnowledgeBase[] }).knowledge_bases ?? []);
      setBases(list);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleAdd = async () => {
    setSaving(true);
    try {
      const res = await fetch("/api/knowledge", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(form) });
      if (!res.ok) throw new Error(`${res.status}`);
      setShowAdd(false);
      setForm({ name: "", description: "" });
      await load();
    } catch (e) { setError((e as Error).message); } finally { setSaving(false); }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Delete this knowledge base?")) return;
    try { await fetch(`/api/knowledge/${id}`, { method: "DELETE" }); await load(); } catch { /**/ }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">Knowledge Bases</h2>
          <p className="font-mono text-[11px] text-muted-foreground">{bases.length} bases</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => void load()} className="gap-1.5"><RefreshCw size={13} className={cn(loading && "animate-spin")} />Refresh</Button>
          <Button size="sm" onClick={() => setShowAdd(true)} className="gap-1.5"><Plus size={13} />New</Button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto p-6">
        {loading && <div className="flex items-center gap-2"><Loader2 size={16} className="animate-spin text-muted-foreground" /></div>}
        {error && <p className="font-mono text-[11px] text-destructive">Error: {error}</p>}
        {!loading && bases.length === 0 && <p className="font-mono text-[11px] text-muted-foreground">No knowledge bases configured</p>}
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {bases.map((kb) => (
            <div key={kb.id} className="flex flex-col rounded-lg border border-border bg-card p-4">
              <div className="mb-3 flex items-start justify-between">
                <div className="flex size-8 items-center justify-center rounded-md bg-primary/15"><BookOpen size={14} className="text-primary" /></div>
                <Button variant="ghost" size="icon" className="h-6 w-6 text-muted-foreground hover:text-destructive" onClick={() => void handleDelete(kb.id)} aria-label={`Delete ${kb.name}`}><Trash2 size={12} /></Button>
              </div>
              <p className="font-display text-[13px] font-semibold text-foreground">{kb.name}</p>
              {kb.description && <p className="mt-1 font-body text-xs text-muted-foreground line-clamp-2">{kb.description}</p>}
              {kb.document_count !== undefined && <p className="mt-2 font-mono text-[10px] text-muted-foreground/60">{kb.document_count} documents</p>}
            </div>
          ))}
        </div>
      </div>
      <Dialog open={showAdd} onOpenChange={setShowAdd}>
        <DialogContent>
          <DialogHeader><DialogTitle>Add Knowledge Base</DialogTitle></DialogHeader>
          <div className="flex flex-col gap-3">
            <div><label className="mb-1 block font-mono text-[11px] text-muted-foreground">Name</label><Input value={form.name} onChange={(e) => setForm({ ...form, name: e.target.value })} placeholder="My Knowledge Base" /></div>
            <div><label className="mb-1 block font-mono text-[11px] text-muted-foreground">Description</label><Textarea value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} placeholder="Optional description…" rows={3} /></div>
          </div>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setShowAdd(false)}>Cancel</Button>
            <Button onClick={() => void handleAdd()} disabled={saving || !form.name}>{saving && <Loader2 size={14} className="animate-spin" />}Create</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};
