import { type FC, useMemo, useState } from "react";
import { Loader2, Pencil, Plus, RefreshCw, Trash2, Zap } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
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
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { AdminEmptyState, AdminError, AdminListSkeleton } from "@/admin/components/admin-states";
import { cn } from "@/lib/utils";
import { useSkillsAdmin } from "@/hooks/use-skills-admin";
import type { UarSkill } from "@/types";
import {
  buildCreateSkillRequest,
  buildUpdateSkillRequest,
  DEFAULT_SKILL_FORM,
  joinCommaSeparated,
  type SkillEditorFormState,
} from "./skills-page.utils";

interface MarkdownEditorFieldProps {
  id: string;
  label: string;
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
}

const MarkdownEditorField: FC<MarkdownEditorFieldProps> = ({
  id,
  label,
  value,
  placeholder,
  onChange,
}) => {
  return (
    <div className="space-y-2">
      <Label htmlFor={id} className="font-mono text-xs text-muted-foreground">
        {label}
      </Label>
      <Tabs defaultValue="write" className="w-full">
        <TabsList className="grid h-8 w-full grid-cols-2">
          <TabsTrigger value="write" className="h-6 cursor-pointer text-xs">
            Write
          </TabsTrigger>
          <TabsTrigger value="preview" className="h-6 cursor-pointer text-xs">
            Preview
          </TabsTrigger>
        </TabsList>
        <TabsContent value="write" className="mt-2">
          <Textarea
            id={id}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder={placeholder}
            rows={7}
            className="font-mono text-xs"
          />
        </TabsContent>
        <TabsContent value="preview" className="mt-2">
          <div className="min-h-[9.6rem] rounded-md border border-border bg-muted/20 p-3">
            {value.trim().length === 0 ? (
              <p className="font-mono text-xs text-muted-foreground">Nothing to preview</p>
            ) : (
              <div className="prose prose-sm max-w-none dark:prose-invert">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>{value}</ReactMarkdown>
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export const SkillsPage: FC = () => {
  const {
    skills,
    loading,
    error,
    actionSkillId,
    saving,
    deleting,
    load,
    setError,
    toggle,
    remove,
    save,
  } = useSkillsAdmin();

  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [editingSkillId, setEditingSkillId] = useState<string | null>(null);
  const [form, setForm] = useState<SkillEditorFormState>(DEFAULT_SKILL_FORM);

  const [deleteTarget, setDeleteTarget] = useState<UarSkill | null>(null);

  const resetDialogState = () => {
    setForm(DEFAULT_SKILL_FORM);
    setEditingSkillId(null);
  };

  const openCreate = () => {
    resetDialogState();
    setIsDialogOpen(true);
  };

  const openEdit = (skill: UarSkill) => {
    setEditingSkillId(skill.skill_id);
    setForm({
      title: skill.title ?? "",
      version: skill.version ?? "1.0.0",
      description: skill.description ?? "",
      promptOverlay: skill.prompt_overlay ?? "",
      keywords: joinCommaSeparated(skill.triggers?.keywords),
      preferredTools: joinCommaSeparated(skill.preferred_tools),
      enabled: skill.enabled !== false,
    });
    setIsDialogOpen(true);
  };

  const handleToggle = (skill: UarSkill, enabled: boolean) => {
    void toggle(skill, enabled);
  };

  const handleDeleteConfirm = async () => {
    if (!deleteTarget) return;
    try {
      await remove(deleteTarget);
      setDeleteTarget(null);
    } catch {
      /* error surfaced via store */
    }
  };

  const handleSave = async () => {
    const title = form.title.trim();
    if (!title) {
      setError("Skill title is required.");
      return;
    }

    setError(null);
    try {
      await save(
        editingSkillId,
        buildCreateSkillRequest(form),
        buildUpdateSkillRequest(form),
      );
      setIsDialogOpen(false);
      resetDialogState();
    } catch {
      /* error surfaced via store */
    }
  };

  const dialogTitle = editingSkillId ? "Edit Skill" : "Create Skill";
  const dialogCta = editingSkillId ? "Save Changes" : "Create Skill";

  const sortedSkills = useMemo(
    () => [...skills].sort((a, b) => (a.title || a.skill_id).localeCompare(b.title || b.skill_id)),
    [skills],
  );

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
        <div>
          <h2 className="font-display text-lg font-semibold text-foreground">Skills</h2>
          <p className="font-mono text-xs text-muted-foreground">{skills.length} skills</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => void load()} className="gap-1.5">
            <RefreshCw size={13} className={cn(loading && "animate-spin")} />
            Refresh
          </Button>
          <Button size="sm" onClick={openCreate} className="gap-1.5">
            <Plus size={13} />
            New Skill
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {loading && sortedSkills.length === 0 && <AdminListSkeleton rows={4} />}
        <AdminError error={error} />
        {!loading && sortedSkills.length === 0 && !error && (
          <AdminEmptyState
            icon={Zap}
            title="No skills configured"
            description="Skills give agents specialized capabilities. Create a skill to define custom behaviors and prompts."
            action={{ label: "New Skill", icon: Plus, onClick: () => { resetDialogState(); setIsDialogOpen(true); } }}
          />
        )}

        <div className="flex flex-col gap-2.5">
          {sortedSkills.map((skill) => {
            const isEnabled = skill.enabled !== false;
            const isBusy = actionSkillId === skill.skill_id;
            return (
              <div
                key={skill.skill_id}
                className={cn(
                  "flex items-center gap-3 rounded-lg border border-border bg-card px-4 py-3",
                  !isEnabled && "opacity-75",
                )}
              >
                <div
                  className={cn(
                    "flex size-8 items-center justify-center rounded-md",
                    isEnabled ? "bg-primary/15" : "bg-muted",
                  )}
                >
                  <Zap size={14} className={isEnabled ? "text-primary" : "text-muted-foreground"} />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="font-display text-sm font-semibold text-foreground">
                    {skill.title || skill.skill_id}
                  </p>
                  {skill.description && (
                    <p className="line-clamp-1 font-body text-xs text-muted-foreground">{skill.description}</p>
                  )}
                  <p className="font-mono text-[10px] text-muted-foreground">
                    {skill.provider_id ? `provider: ${skill.provider_id}` : `id: ${skill.skill_id}`}
                  </p>
                </div>
                <div className="flex items-center gap-1.5">
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 cursor-pointer px-2 text-xs"
                    onClick={() => openEdit(skill)}
                  >
                    <Pencil size={12} className="mr-1" />
                    Edit
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 cursor-pointer border-destructive/50 px-2 text-xs text-destructive hover:bg-destructive/10"
                    onClick={() => setDeleteTarget(skill)}
                  >
                    <Trash2 size={12} className="mr-1" />
                    Delete
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 min-w-[82px] cursor-pointer text-xs"
                    disabled={isBusy}
                    onClick={() => void handleToggle(skill, !isEnabled)}
                  >
                    {isBusy ? (
                      <Loader2 size={12} className="animate-spin" />
                    ) : isEnabled ? (
                      "Disable"
                    ) : (
                      "Enable"
                    )}
                  </Button>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <Dialog
        open={isDialogOpen}
        onOpenChange={(open) => {
          setIsDialogOpen(open);
          if (!open) resetDialogState();
        }}
      >
        <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>{dialogTitle}</DialogTitle>
          </DialogHeader>
          <div className="space-y-3">
            <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
              <div className="space-y-1.5">
                <Label htmlFor="skill-title" className="font-mono text-xs text-muted-foreground">
                  Skill title
                </Label>
                <Input
                  id="skill-title"
                  value={form.title}
                  onChange={(e) => setForm((prev) => ({ ...prev, title: e.target.value }))}
                  placeholder="Customer Success Coach"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="skill-version" className="font-mono text-xs text-muted-foreground">
                  Version
                </Label>
                <Input
                  id="skill-version"
                  value={form.version}
                  onChange={(e) => setForm((prev) => ({ ...prev, version: e.target.value }))}
                  placeholder="1.0.0"
                />
              </div>
            </div>

            <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
              <div className="space-y-1.5">
                <Label htmlFor="skill-keywords" className="font-mono text-xs text-muted-foreground">
                  Keywords (comma-separated)
                </Label>
                <Input
                  id="skill-keywords"
                  value={form.keywords}
                  onChange={(e) => setForm((prev) => ({ ...prev, keywords: e.target.value }))}
                  placeholder="support, escalation, CSAT"
                />
              </div>
              <div className="space-y-1.5">
                <Label
                  htmlFor="skill-preferred-tools"
                  className="font-mono text-xs text-muted-foreground"
                >
                  Preferred tools (comma-separated)
                </Label>
                <Input
                  id="skill-preferred-tools"
                  value={form.preferredTools}
                  onChange={(e) => setForm((prev) => ({ ...prev, preferredTools: e.target.value }))}
                  placeholder="search, memory, compiler"
                />
              </div>
            </div>

            <div className="flex items-center justify-between rounded-md border border-border bg-muted/20 px-3 py-2">
              <div>
                <p className="font-display text-sm font-medium text-foreground">Enabled</p>
                <p className="font-mono text-xs text-muted-foreground">
                  Disabled skills remain installed but cannot be selected.
                </p>
              </div>
              <Switch
                checked={form.enabled}
                onCheckedChange={(checked) => setForm((prev) => ({ ...prev, enabled: checked }))}
                aria-label="Enable skill"
              />
            </div>

            <MarkdownEditorField
              id="skill-description"
              label="Description (Markdown)"
              value={form.description}
              onChange={(value) => setForm((prev) => ({ ...prev, description: value }))}
              placeholder="Describe this skill's purpose and usage."
            />

            <MarkdownEditorField
              id="skill-prompt-overlay"
              label="Prompt Overlay (Markdown)"
              value={form.promptOverlay}
              onChange={(value) => setForm((prev) => ({ ...prev, promptOverlay: value }))}
              placeholder="Write the skill prompt/overlay markdown."
            />
          </div>
          <DialogFooter>
            <Button
              variant="ghost"
              onClick={() => {
                setIsDialogOpen(false);
                resetDialogState();
              }}
            >
              Cancel
            </Button>
            <Button onClick={() => void handleSave()} disabled={saving || !form.title.trim()}>
              {saving && <Loader2 size={14} className="mr-1 animate-spin" />}
              {dialogCta}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog open={deleteTarget !== null} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete skill?</AlertDialogTitle>
            <AlertDialogDescription>
              {deleteTarget
                ? `This permanently deletes "${deleteTarget.title || deleteTarget.skill_id}".`
                : "This permanently deletes the selected skill."}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={(event) => {
                event.preventDefault();
                void handleDeleteConfirm();
              }}
              disabled={deleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {deleting ? <Loader2 size={12} className="animate-spin" /> : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
