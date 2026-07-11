import { useSkillsAdminStore } from "@/stores/skills-admin-store";

export function useSkillsAdmin() {
  const loading = useSkillsAdminStore((state) => state.loading);
  const error = useSkillsAdminStore((state) => state.error);
  const saving = useSkillsAdminStore((state) => state.saving);
  const deleting = useSkillsAdminStore((state) => state.deleting);
  const actionSkillId = useSkillsAdminStore((state) => state.actionSkillId);
  const load = useSkillsAdminStore((state) => state.load);
  const toggle = useSkillsAdminStore((state) => state.toggle);
  const remove = useSkillsAdminStore((state) => state.remove);
  const save = useSkillsAdminStore((state) => state.save);
  const parseImport = useSkillsAdminStore((state) => state.parseImport);
  const importParsed = useSkillsAdminStore((state) => state.importParsed);
  const clearError = useSkillsAdminStore((state) => state.clearError);
  return { loading, error, saving, deleting, actionSkillId, load, toggle, remove, save, parseImport, importParsed, clearError };
}
