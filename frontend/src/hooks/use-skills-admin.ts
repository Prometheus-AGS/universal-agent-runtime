import { useEffect } from "react";

import { useSkillsAdminStore } from "@/stores/skills-admin-store";

export function useSkillsAdmin() {
  const skills = useSkillsAdminStore((s) => s.skills);
  const loading = useSkillsAdminStore((s) => s.loading);
  const error = useSkillsAdminStore((s) => s.error);
  const actionSkillId = useSkillsAdminStore((s) => s.actionSkillId);
  const saving = useSkillsAdminStore((s) => s.saving);
  const deleting = useSkillsAdminStore((s) => s.deleting);
  const load = useSkillsAdminStore((s) => s.load);
  const setError = useSkillsAdminStore((s) => s.setError);
  const toggle = useSkillsAdminStore((s) => s.toggle);
  const remove = useSkillsAdminStore((s) => s.remove);
  const save = useSkillsAdminStore((s) => s.save);

  useEffect(() => {
    void load();
  }, [load]);

  return {
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
  };
}
