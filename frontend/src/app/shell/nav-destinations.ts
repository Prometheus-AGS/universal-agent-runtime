import type { LucideIcon } from "lucide-react";
import {
  Bot,
  BookOpen,
  Boxes,
  FileClock,
  Info,
  MessageSquare,
  Server,
  Settings2,
  SlidersHorizontal,
  Sparkles,
} from "lucide-react";

export type NavigationGroup = "work" | "configure" | "system";

export type NavigationDestinationId =
  | "chat"
  | "knowledge"
  | "agents"
  | "runs"
  | "providers"
  | "mcp-tools"
  | "skills"
  | "a2ui"
  | "runtime-settings"
  | "about";

export interface NavigationDestination {
  id: NavigationDestinationId;
  path: string;
  label: string;
  description: string;
  group: NavigationGroup;
  icon: LucideIcon;
  compactTab?: boolean;
  matchPaths?: readonly string[];
  exactMatchPaths?: readonly string[];
}

const BASE_NAV_DESTINATIONS: readonly NavigationDestination[] = [
  {
    id: "chat",
    path: "/threads",
    label: "Chat",
    description: "Conversations and agent work",
    group: "work",
    icon: MessageSquare,
    compactTab: true,
  },
  {
    id: "knowledge",
    path: "/admin/knowledge",
    label: "Knowledge",
    description: "Documents, retrieval, and memory",
    group: "work",
    icon: BookOpen,
    compactTab: true,
  },
  {
    id: "agents",
    path: "/admin/agents",
    label: "Agents",
    description: "Agent definitions and capabilities",
    group: "work",
    icon: Bot,
    compactTab: true,
  },
  {
    id: "runs",
    path: "/admin/runs",
    label: "Runs",
    description: "Runtime execution history",
    group: "work",
    icon: FileClock,
  },
  {
    id: "providers",
    path: "/admin/providers",
    label: "Providers",
    description: "Models, credentials, and routes",
    group: "configure",
    icon: Server,
    matchPaths: ["/admin/providers", "/admin/credentials", "/admin/models"],
  },
  {
    id: "mcp-tools",
    path: "/admin/tools",
    label: "MCP & tools",
    description: "Tool discovery and server health",
    group: "configure",
    icon: Boxes,
    matchPaths: ["/admin/tools", "/admin/mcp-health"],
  },
  {
    id: "skills",
    path: "/admin/skills",
    label: "Skills",
    description: "Skill registry and compiler",
    group: "configure",
    icon: Sparkles,
    matchPaths: ["/admin/skills", "/admin/compiler"],
  },
  {
    id: "runtime-settings",
    path: "/admin/settings",
    label: "Runtime settings",
    description: "Runtime, access, and application settings",
    group: "configure",
    icon: Settings2,
    matchPaths: [
      "/admin/runtime",
      "/admin/approvals",
      "/admin/protocols",
      "/admin/settings",
      "/admin/auth",
      "/admin/cost",
      "/admin/memory",
    ],
    exactMatchPaths: ["/admin"],
  },
  {
    id: "about",
    path: "/about",
    label: "About",
    description: "Runtime version and capabilities",
    group: "system",
    icon: Info,
    matchPaths: [],
    exactMatchPaths: ["/about"],
  },
] as const;

const DEVELOPMENT_NAV_DESTINATIONS: readonly NavigationDestination[] = [{
  id: "a2ui",
  path: "/admin/a2ui-testing",
  label: "A2UI",
  description: "Generated interface surface testing",
  group: "configure",
  icon: SlidersHorizontal,
}] as const;

export function buildNavigationDestinations({ includeDevelopment }: { includeDevelopment: boolean }): readonly NavigationDestination[] {
  const configureIndex = BASE_NAV_DESTINATIONS.findIndex((destination) => destination.id === "runtime-settings");
  if (!includeDevelopment || configureIndex < 0) return BASE_NAV_DESTINATIONS;
  return [
    ...BASE_NAV_DESTINATIONS.slice(0, configureIndex),
    ...DEVELOPMENT_NAV_DESTINATIONS,
    ...BASE_NAV_DESTINATIONS.slice(configureIndex),
  ];
}

export const NAV_DESTINATIONS = buildNavigationDestinations({ includeDevelopment: import.meta.env.DEV });

export const WORK_DESTINATIONS = NAV_DESTINATIONS.filter(
  (destination) => destination.group === "work",
);

export const CONFIGURE_DESTINATIONS = NAV_DESTINATIONS.filter(
  (destination) => destination.group === "configure",
);

export const SYSTEM_DESTINATIONS = NAV_DESTINATIONS.filter(
  (destination) => destination.group === "system",
);

export const COMPACT_DESTINATIONS = NAV_DESTINATIONS.filter(
  (destination) => destination.compactTab,
);

function matchesPath(candidate: string, pathname: string): boolean {
  return pathname === candidate || pathname.startsWith(`${candidate}/`);
}

export function isDestinationActive(
  destination: NavigationDestination,
  pathname: string,
): boolean {
  const paths = destination.matchPaths ?? [destination.path];
  return (
    destination.exactMatchPaths?.includes(pathname) === true ||
    paths.some((path) => matchesPath(path, pathname))
  );
}

export function findDestinationForPath(
  pathname: string,
  destinations: readonly NavigationDestination[] = NAV_DESTINATIONS,
): NavigationDestination | undefined {
  return destinations.find((destination) =>
    isDestinationActive(destination, pathname),
  );
}

export function isConfigurePath(pathname: string): boolean {
  return findDestinationForPath(pathname)?.group === "configure";
}
