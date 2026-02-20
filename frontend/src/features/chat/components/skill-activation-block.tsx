import { CheckCircle2Icon, Loader2Icon, ZapIcon } from "lucide-react";
import type { FC } from "react";
import { cn } from "@/lib/utils";

const METHOD_LABELS: Record<string, string> = {
  "skill_service.keyword": "keyword",
  "skill_service.embedding": "embedding",
  "skill_service.local_embedding": "local embedding",
  "skill_service.llm": "LLM",
  "skill_service.hybrid": "hybrid",
  "legacy_classifier.rules": "rules",
  "legacy_classifier.tfidf": "TF-IDF",
  "legacy_classifier.wasm": "WASM",
  "legacy_classifier.local_embedding": "local embedding",
  "legacy_classifier.llm": "LLM",
  "legacy_classifier.hybrid": "hybrid",
  "legacy_fallback.tag_vector_hybrid": "tag+vector",
};

interface SkillActivationBlockProps {
  skillId: string;
  skillName: string;
  selectionMethod?: string;
  status: "active" | "complete";
}

export const SkillActivationBlock: FC<SkillActivationBlockProps> = ({ skillName, selectionMethod, status }) => {
  const isActive = status === "active";
  const methodLabel = selectionMethod ? (METHOD_LABELS[selectionMethod] ?? selectionMethod) : null;
  return (
    <div className="my-1.5 flex items-center gap-2 rounded-lg border border-primary/20 bg-primary/5 px-3 py-2">
      <ZapIcon size={12} className={cn("shrink-0 text-primary", isActive && "animate-pulse")} />
      <span className="font-mono text-[11px] font-medium text-primary">{skillName}</span>
      {methodLabel && <span className="rounded-full bg-primary/10 px-1.5 py-0.5 font-mono text-[9px] text-primary/70">{methodLabel}</span>}
      <span className="ml-auto">
        {isActive ? <Loader2Icon size={10} className="animate-spin text-primary/60" /> : <CheckCircle2Icon size={10} className="text-primary/60" />}
      </span>
    </div>
  );
};
