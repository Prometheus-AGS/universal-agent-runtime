import { useEffect, useState } from "react";

// Accessibility mechanics only: no agent records or persisted business state.
// The lifetime is one document; restored notices never request an announcement.
const consumedSelections = new Set<string>();

export function useSelectionAnnouncement(id: string, requested: boolean, agentName: string): string {
  const [announcement, setAnnouncement] = useState({ id: "", text: "" });
  useEffect(() => {
    if (!requested || consumedSelections.has(id)) return;
    consumedSelections.add(id);
    setAnnouncement({ id, text: `Selected agent: ${agentName}` });
  }, [id, requested, agentName]);
  return requested && announcement.id === id ? announcement.text : "";
}
