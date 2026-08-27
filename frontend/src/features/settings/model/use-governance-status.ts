import { useCallback, useEffect, useRef, useState } from "react";
import { useGraphEntity } from "@/entities/hooks/use-graph-entities";
import { fetchGovernanceStatus } from "../api/settings-api";
import {
  onSettingsChanged,
  onSettingsRealtimeConnected,
} from "../api/settings-change-bus";
import {
  GOVERNANCE_STATUS_ENTITY_ID,
  GOVERNANCE_STATUS_ENTITY_TYPE,
  ingestGovernanceStatus,
  invalidateGovernanceStatus,
  nextGovernanceRequestSequence,
  type GovernanceStatusEntity,
} from "./governance-status";

const GOVERNANCE_STATUS_REVALIDATION_MS = 60_000;

export interface UseGovernanceStatusReturn {
  status: GovernanceStatusEntity | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useGovernanceStatus(): UseGovernanceStatusReturn {
  const status =
    useGraphEntity<GovernanceStatusEntity>(
      GOVERNANCE_STATUS_ENTITY_TYPE,
      GOVERNANCE_STATUS_ENTITY_ID,
    ) ?? null;
  const latestRequest = useRef(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const requestSequence = nextGovernanceRequestSequence();
    let terminalSequence = requestSequence;
    latestRequest.current = requestSequence;
    setLoading(true);
    try {
      const next = await fetchGovernanceStatus();
      const ingested = ingestGovernanceStatus(next, requestSequence);
      if (ingested.restarted) {
        const confirmationSequence = nextGovernanceRequestSequence();
        terminalSequence = confirmationSequence;
        latestRequest.current = confirmationSequence;
        const confirmed = await fetchGovernanceStatus();
        const confirmationResult = ingestGovernanceStatus(
          confirmed,
          confirmationSequence,
        );
        if (
          !confirmationResult.accepted ||
          confirmationResult.restarted ||
          confirmed.boot_instance_id !== next.boot_instance_id
        ) {
          throw new Error(
            "Governance restart confirmation did not match the adopted runtime",
          );
        }
      }
      if (latestRequest.current === terminalSequence) setError(null);
    } catch (cause) {
      if (latestRequest.current === terminalSequence) {
        const invalidated = invalidateGovernanceStatus(terminalSequence);
        setError(invalidated ? (cause as Error).message : null);
      }
    } finally {
      if (latestRequest.current === terminalSequence) setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const removeSettingsListener = onSettingsChanged((detail) => {
      if (detail.namespace === "governance" || detail.namespace === "*") {
        void refresh();
      }
    });
    const removeReconnectListener = onSettingsRealtimeConnected(() => {
      void refresh();
    });
    const handleFocus = () => void refresh();
    window.addEventListener("focus", handleFocus);
    const interval = window.setInterval(
      () => void refresh(),
      GOVERNANCE_STATUS_REVALIDATION_MS,
    );
    return () => {
      removeSettingsListener();
      removeReconnectListener();
      window.removeEventListener("focus", handleFocus);
      window.clearInterval(interval);
    };
  }, [refresh]);

  return { status, loading, error, refresh };
}
