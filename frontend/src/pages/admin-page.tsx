import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { AdminShell, type AdminSection } from "@/admin/admin-shell";
import { AdminWelcome } from "@/admin/components/admin-welcome";
import { ProvidersPage } from "@/admin/pages/providers-page";
import { ModelsPage } from "@/admin/pages/models-page";
import { SkillsPage } from "@/admin/pages/skills-page";
import { AgentsPage } from "@/admin/pages/agents-page";
import { ToolsPage } from "@/admin/pages/tools-page";
import { AuthPage } from "@/admin/pages/auth-page";
import { KnowledgePage } from "@/admin/pages/knowledge-page";
import { MemoryPage } from "@/admin/pages/memory-page";
import { CompilerPage } from "@/admin/pages/compiler-page";
import { SettingsPage } from "@/admin/pages/settings-page";
import { A2uiTestingPage } from "@/admin/A2uiTestingPage";
import { McpHealthPage } from "@/admin/McpHealthPage";
import {
  RuntimeApprovalsPage,
  RuntimeCockpitPage,
  RuntimeProtocolsPage,
  RuntimeRunsPage,
} from "@/admin/pages/runtime-console-page";

const PAGE_MAP: Record<AdminSection, () => ReactNode> = {
  runtime: () => <RuntimeCockpitPage />,
  runs: () => <RuntimeRunsPage />,
  approvals: () => <RuntimeApprovalsPage />,
  protocols: () => <RuntimeProtocolsPage />,
  providers: () => <ProvidersPage />,
  models: () => <ModelsPage />,
  skills: () => <SkillsPage />,
  agents: () => <AgentsPage />,
  tools: () => <ToolsPage />,
  auth: () => <AuthPage />,
  knowledge: () => <KnowledgePage />,
  memory: () => <MemoryPage />,
  compiler: () => <CompilerPage />,
  settings: () => <SettingsPage />,
  "a2ui-testing": () => <A2uiTestingPage />,
  "mcp-health": () => <McpHealthPage />,
};

export function AdminPage() {
  const navigate = useNavigate();

  function renderAdminContent(section: AdminSection) {
    return (
      <div
        className="flex flex-1 flex-col overflow-hidden"
        data-testid={`admin-section-${section}`}
      >
        {/* Welcome banner — keep provider onboarding on that page only. */}
        {section === "providers" && (
          <AdminWelcome onNavigate={(path) => navigate(path)} />
        )}
        <div className="flex flex-1 overflow-hidden">
          {PAGE_MAP[section]?.() ?? null}
        </div>
      </div>
    );
  }

  return <AdminShell renderContent={renderAdminContent} />;
}
