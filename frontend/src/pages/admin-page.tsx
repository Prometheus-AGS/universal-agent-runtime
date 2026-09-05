import { lazy, Suspense, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router";

import { AgentsPage } from "@/features/agents";
import { AuthPage } from "@/features/auth";
import { CompilerPage } from "@/features/compiler";
import { CostDashboardPage } from "@/features/cost";
import { CredentialsPage } from "@/features/credentials";
import { KnowledgePage } from "@/features/knowledge";
import { MemoryPage } from "@/features/memory";
import { ModelsPage } from "@/features/models";
import { PresentationsPage } from "@/features/presentations";
import { AdminWelcome, ProvidersPage } from "@/features/providers";
import {
  RuntimeApprovalsPage,
  RuntimeCockpitPage,
  RuntimeProtocolsPage,
  RuntimeRunsPage,
} from "@/features/runtime";
import { useRuntimeConsoleFeeds } from "@/features/runtime/model/runtime-console-feeds";
import { SettingsPage } from "@/features/settings";
import { SkillsPage } from "@/features/skills";
import { McpHealthPage, ToolsPage } from "@/features/tools";

const LazyA2uiTestingPage = import.meta.env.DEV
  ? lazy(() => import("@/features/a2ui/testing").then((module) => ({
      default: module.A2uiTestingPage,
    })))
  : null;

const PAGE_MAP = {
  runtime: () => <RuntimeCockpitPage />,
  runs: () => <RuntimeRunsPage />,
  approvals: () => <RuntimeApprovalsPage />,
  protocols: () => <RuntimeProtocolsPage />,
  providers: () => <ProvidersPage />,
  credentials: () => <CredentialsPage />,
  models: () => <ModelsPage />,
  skills: () => <SkillsPage />,
  presentations: () => <PresentationsPage />,
  agents: () => <AgentsPage />,
  tools: () => <ToolsPage />,
  auth: () => <AuthPage />,
  knowledge: () => <KnowledgePage />,
  memory: () => <MemoryPage />,
  compiler: () => <CompilerPage />,
  settings: () => <SettingsPage />,
  "mcp-health": () => <McpHealthPage />,
  cost: () => <CostDashboardPage />,
} satisfies Record<string, () => ReactNode>;

type AdminSection = keyof typeof PAGE_MAP | "a2ui-testing";

function sectionFromPath(pathname: string): AdminSection {
  const segment = pathname.replace(/^\/admin\/?/, "").split("/")[0];
  if (segment in PAGE_MAP) return segment as keyof typeof PAGE_MAP;
  if (segment === "a2ui-testing" && LazyA2uiTestingPage) return segment;
  return "runtime";
}

export function AdminPage() {
  const { pathname } = useLocation();
  const navigate = useNavigate();
  const section = sectionFromPath(pathname);

  useRuntimeConsoleFeeds();

  const content = section === "a2ui-testing" && LazyA2uiTestingPage
    ? (
        <Suspense fallback={<div className="flex flex-1 items-center justify-center" role="status">Loading A2UI testing…</div>}>
          <LazyA2uiTestingPage />
        </Suspense>
      )
    : PAGE_MAP[section as keyof typeof PAGE_MAP]();

  return (
    <div
      className="flex min-w-0 flex-1 flex-col overflow-hidden"
      data-testid={`admin-section-${section}`}
    >
      {section === "providers" && (
        <AdminWelcome onNavigate={(path) => navigate(path)} />
      )}
      <div className="flex min-h-0 flex-1 overflow-hidden">{content}</div>
    </div>
  );
}
