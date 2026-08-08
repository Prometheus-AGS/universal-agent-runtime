import { AlertTriangleIcon, ShieldXIcon } from "lucide-react";
import { useState } from "react";
import type { ToolApprovalChunk, ToolDeniedChunk } from "@/features/chat/model/chunk";
import { Button } from "@/components/ui/button";
import { ToolApprovalDialog } from "@/features/chat/components/ToolApprovalDialog";
import { ChunkSurface, JsonSource } from "./chunk-surface";

export function ToolApprovalChunkView({ chunk }: { chunk: ToolApprovalChunk }) {
  const [open, setOpen] = useState(false);
  const resolved = chunk.decision !== undefined;
  return (
    <ChunkSurface className="bg-warning/10" label="Tool approval required" live={!resolved}>
      <div className="flex flex-wrap items-center gap-2">
        <AlertTriangleIcon size={16} className="text-warning" aria-hidden="true" />
        <span className="font-mono text-xs font-medium">Approval required · {chunk.toolName}</span>
        <span className="ml-auto text-xs">{chunk.decision ?? "pending"}</span>
      </div>
      {chunk.reason ? <p className="mt-2 text-sm text-fg-sub">{chunk.reason}</p> : null}
      <div className="mt-2"><JsonSource value={chunk.args} label="Proposed tool arguments" /></div>
      {!resolved && chunk.runId ? (
        <Button type="button" size="sm" className="mt-3 min-h-11" onClick={() => setOpen(true)}>Review approval</Button>
      ) : null}
      {chunk.runId ? <ToolApprovalDialog open={open} onOpenChange={setOpen} runId={chunk.runId} toolName={chunk.toolName} args={chunk.args} riskReason={chunk.reason} /> : null}
    </ChunkSurface>
  );
}

export function ToolDeniedChunkView({ chunk }: { chunk: ToolDeniedChunk }) {
  return (
    <ChunkSurface className="bg-destructive/10" label="Tool denied">
      <div className="flex items-center gap-2 text-destructive">
        <ShieldXIcon size={16} aria-hidden="true" />
        <span className="font-mono text-xs font-medium">Denied · {chunk.toolName}</span>
      </div>
      <p className="mt-2 text-sm">{chunk.reason}</p>
      {chunk.policy ? <p className="mt-1 font-mono text-[10px] text-fg-faint">Policy: {chunk.policy}</p> : null}
    </ChunkSurface>
  );
}
