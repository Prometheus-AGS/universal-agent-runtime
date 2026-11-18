import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router";

/**
 * Workspace and selection are URL state: `?workspace=<id>&team=<id>` or `&definition=<id>@<version>`.
 * The last workspace is remembered in localStorage only to prefill when the address names none.
 */

export const DEFAULT_WORKSPACE = "default";
const STORAGE_KEY = "uar.teams.workspace";

export type TeamsSelection =
  | { kind: "none" }
  | { kind: "team"; teamId: string }
  | { kind: "definition"; key: string };

/** The workspace is sent as an HTTP header value, so it must be visible ASCII without spaces. */
export function isValidWorkspace(value: string): boolean {
  return /^[\x21-\x7e]+$/.test(value);
}

export function workspaceProblem(value: string): string | null {
  const text = value.trim();
  if (!text) return "Enter a workspace id.";
  if (!isValidWorkspace(text)) return "Use visible ASCII characters without spaces; the id is sent as a request header.";
  return null;
}

function readRemembered(): string | null {
  try {
    const value = window.localStorage.getItem(STORAGE_KEY);
    return value && isValidWorkspace(value) ? value : null;
  } catch {
    return null;
  }
}

function remember(workspace: string): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, workspace);
  } catch {
    // Remembering is a convenience; private windows may refuse it.
  }
}

export function selectionFrom(team: string | null, definition: string | null): TeamsSelection {
  if (team) return { kind: "team", teamId: team };
  if (definition) return { kind: "definition", key: definition };
  return { kind: "none" };
}

export interface TeamsLocation {
  workspace: string;
  selection: TeamsSelection;
  selectTeam: (teamId: string) => void;
  selectDefinition: (key: string) => void;
  clearSelection: () => void;
  /** Switch workspace. The selected team belongs to the old workspace, so it is dropped. */
  commitWorkspace: (workspace: string) => void;
}

export function useTeamsLocation(): TeamsLocation {
  const [params, setParams] = useSearchParams();
  const [remembered] = useState(readRemembered);
  const fromUrl = params.get("workspace")?.trim() ?? "";
  const workspace = isValidWorkspace(fromUrl) ? fromUrl : (remembered ?? DEFAULT_WORKSPACE);
  const team = params.get("team");
  const definition = params.get("definition");

  const update = useCallback(
    (mutate: (next: URLSearchParams) => void, replace = false) => {
      setParams(
        (current) => {
          const next = new URLSearchParams(current);
          mutate(next);
          return next;
        },
        { replace },
      );
    },
    [setParams],
  );

  // Make the address shareable: write the effective workspace into it.
  useEffect(() => {
    if (fromUrl !== workspace) update((next) => next.set("workspace", workspace), true);
  }, [fromUrl, workspace, update]);
  useEffect(() => remember(workspace), [workspace]);

  const selection = useMemo(() => selectionFrom(team, definition), [team, definition]);

  const selectTeam = useCallback(
    (teamId: string) => update((next) => { next.set("team", teamId); next.delete("definition"); }),
    [update],
  );
  const selectDefinition = useCallback(
    (key: string) => update((next) => { next.set("definition", key); next.delete("team"); }),
    [update],
  );
  const clearSelection = useCallback(() => update((next) => { next.delete("team"); next.delete("definition"); }), [update]);
  const commitWorkspace = useCallback(
    (value: string) => update((next) => { next.set("workspace", value); next.delete("team"); }),
    [update],
  );

  return { workspace, selection, selectTeam, selectDefinition, clearSelection, commitWorkspace };
}
