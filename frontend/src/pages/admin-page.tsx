import { AdminShell, type AdminSection } from "@/admin/admin-shell";
import { ProvidersPage } from "@/admin/pages/providers-page";
import { ModelsPage } from "@/admin/pages/models-page";
import { SkillsPage } from "@/admin/pages/skills-page";
import { AgentsPage } from "@/admin/pages/agents-page";
import { ToolsPage } from "@/admin/pages/tools-page";
import { AuthPage } from "@/admin/pages/auth-page";
import { KnowledgePage } from "@/admin/pages/knowledge-page";
import { CompilerPage } from "@/admin/pages/compiler-page";

function renderAdminContent(section: AdminSection) {
  switch (section) {
    case "providers": return <ProvidersPage />;
    case "models": return <ModelsPage />;
    case "skills": return <SkillsPage />;
    case "agents": return <AgentsPage />;
    case "tools": return <ToolsPage />;
    case "auth": return <AuthPage />;
    case "knowledge": return <KnowledgePage />;
    case "compiler": return <CompilerPage />;
  }
}

export function AdminPage() {
  return <AdminShell renderContent={renderAdminContent} />;
}
