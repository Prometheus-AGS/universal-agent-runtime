import { type FC, useEffect, useMemo, useState } from "react";
import { FolderOpen, Loader2, Pencil, Plus, RefreshCw, Shield, Trash2, Zap } from "lucide-react";
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
import { SkillImportDialog } from "./skill-import-dialog";
import { ModelSelector } from "@/features/models/model-selector";
import { LoadingCursor } from "@/shared/ui/configuration/loading-cursor";
import { EmptyFrame } from "@/shared/ui/configuration/empty-frame";
import { ErrorBar } from "@/shared/ui/configuration/error-bar";
import { cn } from "@/lib/utils";
import { MarkdownBubble } from "@/shared/markdown";
import { useSkills } from "../model/use-skills";
import { useSkillsAdmin } from "../model/use-skills-admin";
import type { UarSkill } from "@/types";
import {
  buildCreateSkillRequest,
  buildUpdateSkillRequest,
  DEFAULT_SKILL_FORM,
  joinCommaSeparated,
  type SkillEditorFormState,
} from "../model/skills-page.utils";

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
              <MarkdownBubble source={value} />
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export const SkillsPage: FC = () => {
  // Reads from the entity graph (hydrated below; SSE keeps fresh).
  const view = useSkills();
  const skills = view.items as unknown as UarSkill[];

  const admin = useSkillsAdmin();
  const { load } = admin;
  const [validationError, setValidationError] = useState<string | null>(null);
  const error = validationError ?? admin.error;

  useEffect(() => {
    void load().catch(() => undefined);
  }, [load]);

  const loading = admin.loading && skills.length === 0;

  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [editingSkillId, setEditingSkillId] = useState<string | null>(null);
  const [form, setForm] = useState<SkillEditorFormState>(DEFAULT_SKILL_FORM);

  const [isImportOpen, setIsImportOpen] = useState(false);
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
      preferredModel: skill.execution_config?.preferred_model ?? "",
    });
    setIsDialogOpen(true);
  };

  const handleToggle = (skill: UarSkill, enabled: boolean) => {
    void admin.toggle(skill, enabled).catch(() => undefined);
  };

  const handleDeleteConfirm = async () => {
    if (!deleteTarget) return;
    try {
      await admin.remove(deleteTarget);
      setDeleteTarget(null);
    } catch {
      /* error surfaced via store */
    }
  };

  const handleSave = async () => {
    const title = form.title.trim();
    if (!title) {
      setValidationError("Skill title is required.");
      return;
    }

    setValidationError(null);
    try {
      await admin.save(
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
    <div className="flex flex-1 flex-col overflow-hidden font-mono text-[13px] text-[var(--color-terminal-fg)]">
      <div className="flex items-center justify-between border-b border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] px-6 py-4">
        <div>
          <h2 className="text-[20px] font-medium tracking-tight">skills</h2>
          <p className="text-xs text-[var(--color-terminal-fg-dim)]">
            {skills.length} skills
            {admin.loading && <LoadingCursor className="ml-2" />}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => void load().catch(() => undefined)}
            className="gap-1.5 border border-[var(--color-terminal-line-strong)] bg-transparent text-[var(--color-terminal-fg)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_8%,transparent)] focus-visible:outline focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-[var(--color-ember)]"
            aria-label="Refresh skills"
          >
            <RefreshCw size={13} className={cn(admin.loading && "animate-spin")} />refresh
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsImportOpen(true)}
            className="gap-1.5 border border-[var(--color-terminal-line-strong)] bg-transparent text-[var(--color-terminal-fg)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_8%,transparent)]"
            aria-label="Import skills"
          >
            <FolderOpen size={13} />import
          </Button>
          <Button
            size="sm"
            onClick={openCreate}
            className="gap-1.5 border border-[var(--color-phosphor)] bg-[color-mix(in_srgb,var(--color-phosphor)_12%,transparent)] text-[var(--color-phosphor)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_18%,transparent)]"
          >
            <Plus size={13} />new skill
          </Button>
        </div>
      </div>

      {error && <ErrorBar code="SKILLS" message={error} onDismiss={() => { setValidationError(null); admin.clearError(); }} />}

      <div className="flex-1 overflow-y-auto p-6">
        {loading && skills.length === 0 && <LoadingCursor label="loading skills" />}
        {!loading && sortedSkills.length === 0 && !error && (
          <EmptyFrame
            title="no skills configured"
            hint="skills give agents specialised capabilities; create one to define custom behaviours"
            action={
              <Button
                size="sm"
                onClick={() => { resetDialogState(); setIsDialogOpen(true); }}
                className="gap-1.5 border border-[var(--color-phosphor)] bg-[color-mix(in_srgb,var(--color-phosphor)_12%,transparent)] text-[var(--color-phosphor)] hover:bg-[color-mix(in_srgb,var(--color-phosphor)_18%,transparent)]"
              >
                <Plus size={13} />new skill
              </Button>
            }
          />
        )}

        <div className="flex flex-col gap-2.5">
          {sortedSkills.map((skill) => {
            const isEnabled = skill.enabled !== false;
            const isBusy = admin.actionSkillId === skill.skill_id;
            const isBuiltin = (skill as { origin?: string }).origin === "builtin";
            return (
              <div
                key={skill.skill_id}
                className={cn(
                  "flex items-center gap-3 border border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-surface)] px-4 py-3 transition-colors duration-[160ms] hover:border-[color-mix(in_srgb,var(--color-phosphor)_40%,transparent)]",
                  !isEnabled && "opacity-60",
                )}
              >
                <div
                  className={cn(
                    "flex size-8 items-center justify-center border",
                    isEnabled
                      ? "border-[var(--color-phosphor)] bg-[color-mix(in_srgb,var(--color-phosphor)_12%,transparent)]"
                      : "border-[var(--color-terminal-line-strong)] bg-[var(--color-terminal-bg)]",
                  )}
                >
                  <Zap size={14} className={isEnabled ? "text-[var(--color-phosphor)]" : "text-[var(--color-terminal-fg-dim)]"} />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="flex items-center gap-2 text-sm font-medium text-[var(--color-terminal-fg)]">
                    {skill.title || skill.skill_id}
                    {isBuiltin && (
                      <span className="inline-flex items-center gap-1 rounded border border-[color-mix(in_srgb,var(--color-amber)_50%,transparent)] bg-transparent px-2 py-0.5 font-mono text-[9px] uppercase tracking-wide text-[var(--color-amber)]">
                        <Shield size={9} />built-in
                      </span>
                    )}
                  </p>
                  {skill.description && (
                    <p className="line-clamp-1 text-xs text-[var(--color-terminal-fg-dim)]">{skill.description}</p>
                  )}
                  <p className="text-[10px] text-[var(--color-terminal-fg-dim)]">
                    {skill.provider_id ? `provider: ${skill.provider_id}` : `id: ${skill.skill_id}`}
                  </p>
                </div>
                <div className="flex items-center gap-1.5">
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 cursor-pointer px-2 text-xs"
                    onClick={() => openEdit(skill)}
                    aria-label={`Edit ${skill.title || skill.skill_id}`}
                    disabled={isBuiltin}
                    title={isBuiltin ? "System skill — cannot be edited" : undefined}
                  >
                    <Pencil size={12} className="mr-1" />
                    Edit
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-7 cursor-pointer border-destructive/50 px-2 text-xs text-destructive hover:bg-destructive/10"
                    onClick={() => setDeleteTarget(skill)}
                    aria-label={`Delete ${skill.title || skill.skill_id}`}
                    disabled={isBuiltin}
                    title={isBuiltin ? "System skill — cannot be removed" : undefined}
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
                    aria-label={`${isEnabled ? "Disable" : "Enable"} ${skill.title || skill.skill_id}`}
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

            <div className="space-y-1.5">
              <Label className="font-mono text-xs text-muted-foreground">
                Model override (optional)
              </Label>
              <ModelSelector
                value={form.preferredModel}
                onChange={(value) => setForm((prev) => ({ ...prev, preferredModel: value }))}
                defaultLabel="Use agent / global default"
                placeholder="Search models…"
              />
              <p className="font-mono text-[10px] text-muted-foreground">
                When set, this skill overrides the agent's model for its duration.
              </p>
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
            <Button onClick={() => void handleSave()} disabled={admin.saving || !form.title.trim()}>
              {admin.saving && <Loader2 size={14} className="mr-1 animate-spin" />}
              {dialogCta}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <SkillImportDialog
        open={isImportOpen}
        onOpenChange={setIsImportOpen}
        onImported={() => undefined}
        parseImport={admin.parseImport}
        importParsed={admin.importParsed}
      />

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
            <AlertDialogCancel disabled={admin.deleting}>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={(event) => {
                event.preventDefault();
                void handleDeleteConfirm();
              }}
              disabled={admin.deleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {admin.deleting ? <Loader2 size={12} className="animate-spin" /> : "Delete"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};
