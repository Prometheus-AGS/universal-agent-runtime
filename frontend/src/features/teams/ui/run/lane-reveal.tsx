import { useEffect, type MutableRefObject } from "react";
import { useAttemptField, useRoutingProgressField, useRoutingTestTaskIds } from "@/platform/entities";
import type { RunContext } from "./run-hooks";

/** How long the target lane keeps its mark. */
const MARK_MS = 2_400;

function prefersReducedMotion(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/**
 * Brings one member's lane into view, moves focus to its heading and marks it with the ring token for a moment.
 * The lane is found by the `data-member-lane` attribute the lane renders. Scrolling is instant under reduced
 * motion. The mark is a data attribute that React does not render, so a re-render of the lane leaves it alone.
 * Returns the cleanup that cancels the pending work and clears the mark, or null when the lane is not on the page.
 */
function revealLane(teamId: string, memberId: string): (() => void) | null {
  const key = `${teamId}:${memberId}`;
  const lane = Array.from(document.querySelectorAll<HTMLElement>("[data-member-lane]")).find((element) => element.dataset.memberLane === key);
  if (!lane) return null;
  const article = lane.querySelector<HTMLElement>("article");
  const heading = lane.querySelector<HTMLElement>("[data-lane-heading]");
  lane.scrollIntoView({ block: "nearest", behavior: prefersReducedMotion() ? "auto" : "smooth" });
  heading?.focus({ preventScroll: true });
  article?.setAttribute("data-reveal", "true");
  const timer = window.setTimeout(() => article?.removeAttribute("data-reveal"), MARK_MS);
  return () => {
    window.clearTimeout(timer);
    article?.removeAttribute("data-reveal");
  };
}

interface RevealProps extends RunContext {
  taskId: string;
  armedRef: MutableRefObject<boolean>;
  onRevealed: () => void;
}

/**
 * Watches one routing test. Once it is admitted, its attempt is in the graph and a member holds it, the member's lane
 * is revealed, but only for a test the operator sent or watched owe a step in this view: opening a page that already
 * shows an admitted test moves nothing.
 */
function RevealTask({ workspace, teamId, taskId, armedRef, onRevealed }: RevealProps) {
  const step = useRoutingProgressField(workspace, teamId, taskId, "step");
  const memberId = useRoutingProgressField(workspace, teamId, taskId, "memberId");
  const attemptId = useRoutingProgressField(workspace, teamId, taskId, "attemptId");
  const attemptStatus = useAttemptField(workspace, teamId, attemptId ?? "", "status");
  const ready = step === "admitted" && memberId !== undefined && memberId !== null && attemptStatus !== undefined;

  useEffect(() => {
    if (step !== undefined && step !== "admitted") armedRef.current = true;
  }, [step, armedRef]);

  useEffect(() => {
    if (!ready || memberId === undefined || memberId === null || !armedRef.current) return;
    armedRef.current = false;
    let cleanup: (() => void) | null = null;
    // After paint, so the lane has taken its new place in the attention order.
    const frame = window.requestAnimationFrame(() => {
      cleanup = revealLane(teamId, memberId);
      if (cleanup !== null) onRevealed();
    });
    return () => {
      window.cancelAnimationFrame(frame);
      cleanup?.();
    };
  }, [ready, memberId, teamId, armedRef, onRevealed]);

  return null;
}

/** Follows the latest routing test of the team and reveals its member's lane when it is admitted. Renders nothing. */
export function AdmittedLaneReveal({
  workspace,
  teamId,
  armedRef,
  onRevealed,
}: RunContext & { armedRef: MutableRefObject<boolean>; onRevealed: () => void }) {
  const taskIds = useRoutingTestTaskIds(workspace, teamId);
  if (taskIds.length === 0) return null;
  const taskId = taskIds[taskIds.length - 1];
  return <RevealTask key={taskId} workspace={workspace} teamId={teamId} taskId={taskId} armedRef={armedRef} onRevealed={onRevealed} />;
}
