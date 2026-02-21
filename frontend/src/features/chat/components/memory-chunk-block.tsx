import { DatabaseIcon, Edit3Icon, SearchIcon } from "lucide-react";
import type { FC } from "react";

interface MemoryRecallItem {
  key: string;
  value: string;
  source: string;
  scope?: string;
  memory_type?: string;
  importance?: number;
}

interface MemoryRecallBlockProps {
  items: MemoryRecallItem[];
  count?: number;
}

interface MemoryMutationBlockProps {
  operation: string;
  memoryId: string;
  content: string;
  scope: string;
  memoryType: string;
}

interface MemoryUpdateBlockProps {
  memoryKey: string;
  operation: string;
  value: string;
}

function truncate(value: string, max = 140): string {
  const s = value.trim();
  return s.length > max ? `${s.slice(0, max)}...` : s;
}

export const MemoryRecallBlock: FC<MemoryRecallBlockProps> = ({ items, count }) => {
  const total = count ?? items.length;

  return (
    <div className="my-2 rounded-lg border border-border/50 bg-muted/20 px-3 py-2.5">
      <div className="mb-1.5 flex items-center gap-2">
        <SearchIcon size={12} className="text-primary" />
        <span className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
          Memory Recall
        </span>
        <span className="ml-auto font-mono text-[10px] text-muted-foreground/70">
          {total} item{total === 1 ? "" : "s"}
        </span>
      </div>
      <div className="space-y-1.5">
        {items.map((item, idx) => (
          <div key={`${item.key}-${idx}`} className="rounded-md border border-border/30 bg-background/60 px-2 py-1.5">
            <div className="flex items-center gap-1.5">
              <span className="font-mono text-[10px] text-foreground/90">{item.key || "memory"}</span>
              {item.scope && <span className="font-mono text-[9px] text-muted-foreground/70">[{item.scope}]</span>}
              {item.memory_type && <span className="font-mono text-[9px] text-muted-foreground/70">{item.memory_type}</span>}
            </div>
            <p className="mt-0.5 whitespace-pre-wrap font-body text-[12px] text-muted-foreground">
              {truncate(item.value)}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
};

export const MemoryMutationBlock: FC<MemoryMutationBlockProps> = ({
  operation,
  memoryId,
  content,
  scope,
  memoryType,
}) => {
  return (
    <div className="my-2 rounded-lg border border-primary/20 bg-primary/5 px-3 py-2.5">
      <div className="mb-1.5 flex items-center gap-2">
        <Edit3Icon size={12} className="text-primary" />
        <span className="font-mono text-[10px] uppercase tracking-widest text-primary/90">
          Memory {operation}
        </span>
        <span className="ml-auto font-mono text-[9px] text-primary/70">
          {scope} · {memoryType}
        </span>
      </div>
      <p className="font-mono text-[10px] text-muted-foreground/80">
        {memoryId || "(no memory id)"}
      </p>
      {content && (
        <p className="mt-1 whitespace-pre-wrap font-body text-[12px] text-muted-foreground">
          {truncate(content)}
        </p>
      )}
    </div>
  );
};

export const MemoryUpdateBlock: FC<MemoryUpdateBlockProps> = ({ memoryKey, operation, value }) => {
  return (
    <div className="my-2 rounded-lg border border-border/50 bg-muted/20 px-3 py-2.5">
      <div className="mb-1.5 flex items-center gap-2">
        <DatabaseIcon size={12} className="text-muted-foreground" />
        <span className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
          Memory Update
        </span>
        <span className="ml-auto font-mono text-[9px] text-muted-foreground/70">
          {operation}
        </span>
      </div>
      <p className="font-mono text-[10px] text-foreground/80">{memoryKey}</p>
      <p className="mt-1 whitespace-pre-wrap font-body text-[12px] text-muted-foreground">
        {truncate(value)}
      </p>
    </div>
  );
};
