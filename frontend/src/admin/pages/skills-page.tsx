import { type FC, useCallback, useEffect, useState } from "react";
import { Loader2, RefreshCw, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { UarSkill } from "@/types";

export const SkillsPage: FC = () => {
  const [skills, setSkills] = useState<UarSkill[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/skills");
      if (!res.ok) throw new Error(`${res.status}`);
      const data = await res.json() as { skills?: UarSkill[]; data?: { skills?: UarSkill[] } } | UarSkill[];
      const list = Array.isArray(data) ? data : (data.data?.skills ?? (data as { skills?: UarSkill[] }).skills ?? []);
      setSkills(list);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { void load(); }, [load]);

  const handleToggle = async (id: string, enabled: boolean) => {
    try {
      await fetch(`/api/skills/${id}`, { method: "PATCH", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ enabled }) });
      setSkills((prev) => prev.map((s) => s.skill_id === id ? { ...s, enabled } : s));
    } catch { /**/ }
  };

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">Skills</h2>
          <p className="font-mono text-[11px] text-muted-foreground">{skills.length} skills</p>
        </div>
        <Button variant="outline" size="sm" onClick={() => void load()} className="gap-1.5">
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />Refresh
        </Button>
      </div>
      <div className="flex-1 overflow-y-auto p-6">
        {loading && <div className="flex items-center gap-2"><Loader2 size={16} className="animate-spin text-muted-foreground" /></div>}
        {error && <p className="font-mono text-[11px] text-destructive">Error: {error}</p>}
        {!loading && skills.length === 0 && <p className="font-mono text-[11px] text-muted-foreground">No skills configured</p>}
        <div className="flex flex-col gap-2">
          {skills.map((s) => (
            <div key={s.skill_id} className="flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3">
              <div className={cn("flex size-8 items-center justify-center rounded-md", s.enabled !== false ? "bg-primary/15" : "bg-muted")}>
                <Zap size={14} className={s.enabled !== false ? "text-primary" : "text-muted-foreground"} />
              </div>
              <div className="min-w-0 flex-1">
                <p className="font-display text-[13px] font-semibold text-foreground">{s.title}</p>
                {s.description && <p className="font-body text-xs text-muted-foreground line-clamp-1">{s.description}</p>}
                {s.provider_id && <p className="font-mono text-[10px] text-muted-foreground">provider: {s.provider_id}</p>}
              </div>
              <Button variant="outline" size="sm" className="h-7 text-xs" onClick={() => void handleToggle(s.skill_id, !(s.enabled !== false))}>
                {s.enabled !== false ? "Disable" : "Enable"}
              </Button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
