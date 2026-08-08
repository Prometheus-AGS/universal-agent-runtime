import { useEffect, useRef } from "react";
import { defaultRangeExtractor, useVirtualizer } from "@tanstack/react-virtual";
import {
  Bot,
  Brain,
  ChevronDown,
  ChevronRight,
  CircleDot,
  Code2,
  Database,
  MessageSquare,
  Wrench,
} from "lucide-react";

import type { PersistedRunEventKind } from "@/platform/pglite/run-event-repository";
import type {
  RunTraceFilters,
  RunTraceProjection,
  VisibleTraceRow,
} from "@/features/chat/model/run-trace-types";
import { cn } from "@/lib/utils";

const FILTERS: ReadonlyArray<{ kind: PersistedRunEventKind; label: string }> = [
  { kind: "lifecycle", label: "Lifecycle" },
  { kind: "message", label: "Messages" },
  { kind: "reasoning", label: "Reasoning" },
  { kind: "tool", label: "Tools" },
  { kind: "state", label: "State" },
  { kind: "custom", label: "Custom" },
  { kind: "raw", label: "Raw" },
];

function KindIcon({ row }: { row: VisibleTraceRow }) {
  if (row.node.type === "run") return <Bot aria-hidden="true" />;
  if (row.node.type === "phase") return <CircleDot aria-hidden="true" />;
  switch (row.node.event.kind) {
    case "message": return <MessageSquare aria-hidden="true" />;
    case "reasoning": return <Brain aria-hidden="true" />;
    case "tool": return <Wrench aria-hidden="true" />;
    case "state": return <Database aria-hidden="true" />;
    case "raw": return <Code2 aria-hidden="true" />;
    default: return <CircleDot aria-hidden="true" />;
  }
}

function TraceRow({
  row,
  index,
  selected,
  onSelect,
  onToggle,
  onKeyDown,
  measureRef,
}: {
  row: VisibleTraceRow;
  index: number;
  selected: boolean;
  onSelect: () => void;
  onToggle: () => void;
  onKeyDown: (event: React.KeyboardEvent<HTMLDivElement>) => void;
  measureRef?: (element: HTMLDivElement | null) => void;
}) {
  return (
    <div
      ref={measureRef}
      data-index={index}
      id={`trace-row-${row.id.replaceAll(":", "-")}`}
      role="treeitem"
      aria-level={row.depth}
      aria-posinset={row.positionInSet}
      aria-setsize={row.setSize}
      aria-expanded={row.expandable ? row.expanded : undefined}
      aria-selected={selected}
      tabIndex={selected ? 0 : -1}
      className={cn(
        "flex min-h-11 cursor-default items-center gap-2 rounded-lg px-2 py-2 text-sm text-fg-sub transition-colors focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember",
        selected ? "bg-card-hov text-foreground" : "hover:bg-surface hover:text-foreground",
      )}
      style={{ paddingInlineStart: `${8 + (row.depth - 1) * 16}px` }}
      onClick={onSelect}
      onDoubleClick={() => { if (row.expandable) onToggle(); }}
      onKeyDown={onKeyDown}
    >
      <span className="flex size-5 shrink-0 items-center justify-center text-fg-faint">
        {row.expandable
          ? row.expanded ? <ChevronDown aria-hidden="true" /> : <ChevronRight aria-hidden="true" />
          : <KindIcon row={row} />}
      </span>
      <span className="min-w-0 flex-1 truncate font-mono text-xs">{row.node.label}</span>
      {row.node.type === "event" && (
        <span className="shrink-0 font-mono text-[10px] text-fg-faint">#{row.node.event.seq}</span>
      )}
    </div>
  );
}

export function RunTraceTimeline({
  projection,
  filters,
  onToggleFilter,
  onToggleExpanded,
  onSelectNode,
}: {
  projection: RunTraceProjection;
  filters: RunTraceFilters;
  onToggleFilter: (kind: PersistedRunEventKind) => void;
  onToggleExpanded: (nodeId: string) => void;
  onSelectNode: (nodeId: string) => void;
}) {
  const scrollElement = useRef<HTMLDivElement>(null);
  const rowElements = useRef(new Map<string, HTMLDivElement>());
  const pendingFocusNodeId = useRef<string | null>(null);
  const lastScrolledNodeId = useRef<string | null>(null);
  const rows = projection.visibleRows;
  const shouldVirtualize = rows.length > 200;
  const selectedIndex = rows.findIndex((row) => row.id === projection.selectedNodeId);
  // TanStack Virtual intentionally returns imperative functions that React Compiler cannot memoize.
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollElement.current,
    estimateSize: () => 44,
    getItemKey: (index) => rows[index]?.id ?? index,
    measureElement: (element) => element.getBoundingClientRect().height,
    overscan: 8,
    enabled: shouldVirtualize,
    initialRect: { width: 800, height: 440 },
    rangeExtractor: (range) => {
      const indexes = defaultRangeExtractor(range);
      if (selectedIndex >= 0 && !indexes.includes(selectedIndex)) indexes.push(selectedIndex);
      return indexes.sort((left, right) => left - right);
    },
  });
  const measuredItems = virtualizer.getVirtualItems();
  const fallbackIndexes = [
    ...Array.from({ length: Math.min(10, rows.length) }, (_, index) => index),
    ...(selectedIndex >= 10 ? [selectedIndex] : []),
  ];
  const virtualItems = measuredItems.length > 0
    ? measuredItems
    : fallbackIndexes.map((index) => ({
        index,
        key: rows[index]?.id ?? index,
        start: index * 44,
      }));
  const virtualRangeKey = virtualItems.map((item) => item.index).join(",");

  useEffect(() => {
    const selectedIndex = rows.findIndex((row) => row.id === projection.selectedNodeId);
    if (selectedIndex < 0) return;
    const shouldMoveFocus = pendingFocusNodeId.current === projection.selectedNodeId;
    if (lastScrolledNodeId.current !== projection.selectedNodeId || shouldMoveFocus) {
      if (shouldVirtualize) virtualizer.scrollToIndex(selectedIndex, { align: "auto" });
      else rowElements.current.get(rows[selectedIndex]!.id)?.scrollIntoView({ block: "nearest" });
      lastScrolledNodeId.current = projection.selectedNodeId;
    }
    if (!shouldMoveFocus) return;
    const frame = window.requestAnimationFrame(() => {
      const target = rowElements.current.get(projection.selectedNodeId!);
      if (!target) return;
      target.focus({ preventScroll: true });
      if (pendingFocusNodeId.current === projection.selectedNodeId) pendingFocusNodeId.current = null;
    });
    return () => window.cancelAnimationFrame(frame);
  }, [projection.selectedNodeId, rows, shouldVirtualize, virtualizer, virtualRangeKey]);

  const handleKey = (event: React.KeyboardEvent<HTMLDivElement>, index: number): void => {
    const row = rows[index];
    if (!row) return;
    const selectIndex = (next: number): void => {
      const target = rows[Math.max(0, Math.min(next, rows.length - 1))];
      if (target) {
        pendingFocusNodeId.current = target.id;
        onSelectNode(target.id);
      }
    };
    if (event.key === "ArrowDown") {
      event.preventDefault();
      selectIndex(index + 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selectIndex(index - 1);
    } else if (event.key === "Home") {
      event.preventDefault();
      selectIndex(0);
    } else if (event.key === "End") {
      event.preventDefault();
      selectIndex(rows.length - 1);
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      if (row.expandable && !row.expanded) onToggleExpanded(row.id);
      else if (row.expandable) selectIndex(index + 1);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (row.expandable && row.expanded) onToggleExpanded(row.id);
      else if (row.parentId) {
        pendingFocusNodeId.current = row.parentId;
        onSelectNode(row.parentId);
      }
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onSelectNode(row.id);
      if (row.expandable) onToggleExpanded(row.id);
    }
  };

  const renderRow = (row: VisibleTraceRow, index: number, measure = false) => (
    <TraceRow
      key={row.id}
      row={row}
      index={index}
      selected={row.id === projection.selectedNodeId}
      onSelect={() => onSelectNode(row.id)}
      onToggle={() => onToggleExpanded(row.id)}
      onKeyDown={(event) => handleKey(event, index)}
      measureRef={measure
        ? (element) => {
            if (element) {
              rowElements.current.set(row.id, element);
              virtualizer.measureElement(element);
            } else rowElements.current.delete(row.id);
          }
        : (element) => {
            if (element) rowElements.current.set(row.id, element);
            else rowElements.current.delete(row.id);
          }}
    />
  );

  return (
    <section aria-labelledby="run-trace-timeline-heading" className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="flex items-end justify-between gap-3">
        <div>
          <p className="font-mono text-[10px] uppercase tracking-widest text-ember">Sequence</p>
          <h2 id="run-trace-timeline-heading" className="font-display text-base font-semibold text-foreground">
            Event timeline
          </h2>
        </div>
        <span className="font-mono text-xs text-fg-faint">{projection.eventsById.size} events</span>
      </div>

      <div aria-label="Event filters" className="flex flex-wrap gap-1" role="group">
        {FILTERS.map(({ kind, label }) => (
          <button
            key={kind}
            type="button"
            aria-pressed={filters[kind]}
            className={cn(
              "min-h-11 rounded-lg px-3 font-mono text-[10px] font-semibold uppercase tracking-wide transition-colors focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ember",
              filters[kind] ? "bg-card-hov text-foreground" : "bg-surface text-fg-faint",
            )}
            onClick={() => onToggleFilter(kind)}
          >
            {label} {projection.countsByKind[kind]}
          </button>
        ))}
      </div>

      <div
        ref={scrollElement}
        role="tree"
        aria-label="Run event tree"
        data-virtualized={shouldVirtualize ? "true" : "false"}
        data-mounted-count={shouldVirtualize ? virtualItems.length : rows.length}
        className="h-64 min-h-0 flex-1 overflow-auto rounded-xl bg-card p-1"
        style={{ height: 256, minHeight: 0 }}
      >
        {rows.length === 0 ? (
          <p className="px-4 py-10 text-center text-sm text-fg-sub">No persisted events match the current filters.</p>
        ) : shouldVirtualize ? (
          <div className="relative w-full" style={{ height: `${virtualizer.getTotalSize()}px` }}>
            {virtualItems.map((item) => {
              const row = rows[item.index];
              if (!row) return null;
              return (
                <div
                  key={item.key}
                  className="absolute left-0 top-0 w-full"
                  style={{ transform: `translateY(${item.start}px)` }}
                >
                  {renderRow(row, item.index, true)}
                </div>
              );
            })}
          </div>
        ) : rows.map((row, index) => renderRow(row, index))}
      </div>
    </section>
  );
}
