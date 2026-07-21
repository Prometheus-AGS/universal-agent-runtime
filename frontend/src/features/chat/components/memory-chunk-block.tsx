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
    <div className="my-2 rounded-xl bg-surface px-3 py-2.5">
      <div className="mb-1.5 flex items-center gap-2">
        <SearchIcon size={12} className="text-cyan" />
        <span className="eyebrow text-cyan">
          Memory Recall
        </span>
        <span className="ml-auto font-mono text-[10px] text-fg-faint">
          {total} item{total === 1 ? "" : "s"}
        </span>
      </div>
      <div className="space-y-1.5">
        {items.map((item, idx) => (
          <div key={`${item.key}-${idx}`} className="rounded-md bg-card px-2 py-1.5">
            <div className="flex items-center gap-1.5">
              <span className="font-mono text-[10px] text-foreground/90">{item.key || "memory"}</span>
              {item.scope && <span className="font-mono text-[9px] text-fg-faint">[{item.scope}]</span>}
              {item.memory_type && <span className="font-mono text-[9px] text-fg-faint">{item.memory_type}</span>}
            </div>
            <p className="mt-0.5 whitespace-pre-wrap font-body text-xs text-fg-sub">
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
    <div className="my-2 rounded-xl bg-surface px-3 py-2.5">
      <div className="mb-1.5 flex items-center gap-2">
        <Edit3Icon size={12} className="text-cyan" />
        <span className="eyebrow text-cyan">
          Memory {operation}
        </span>
        <span className="ml-auto font-mono text-[9px] text-fg-faint">
          {scope} · {memoryType}
        </span>
      </div>
      <p className="font-mono text-[10px] text-fg-sub">
        {memoryId || "(no memory id)"}
      </p>
      {content && (
        <p className="mt-1 whitespace-pre-wrap font-body text-xs text-fg-sub">
          {truncate(content)}
        </p>
      )}
    </div>
  );
};

export const MemoryUpdateBlock: FC<MemoryUpdateBlockProps> = ({ memoryKey, operation, value }) => {
  return (
    <div className="my-2 rounded-xl bg-surface px-3 py-2.5">
      <div className="mb-1.5 flex items-center gap-2">
        <DatabaseIcon size={12} className="text-cyan" />
        <span className="eyebrow text-cyan">
          Memory Update
        </span>
        <span className="ml-auto font-mono text-[9px] text-fg-faint">
          {operation}
        </span>
      </div>
      <p className="font-mono text-[10px] text-foreground/80">{memoryKey}</p>
      <p className="mt-1 whitespace-pre-wrap font-body text-xs text-fg-sub">
        {truncate(value)}
      </p>
    </div>
  );
};
