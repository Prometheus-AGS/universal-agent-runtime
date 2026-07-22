import type { ComponentType, SVGProps } from "react";
import { Info, MessageSquare, Settings2 } from "lucide-react";

export interface Destination {
  path: string;
  label: string;
  icon: ComponentType<SVGProps<SVGSVGElement>>;
}

/** One product inventory shared by the wide rail and the narrow bottom bar. */
export const DESTINATIONS: readonly Destination[] = [
  { path: "/threads", label: "Chat", icon: MessageSquare },
  { path: "/admin", label: "Admin", icon: Settings2 },
  { path: "/about", label: "About", icon: Info },
] as const;

/** `/about` matches exactly; the other destinations own sub-paths. */
export function isDestinationActive(path: string, pathname: string): boolean {
  return path === "/about" ? pathname === path : pathname.startsWith(path);
}
