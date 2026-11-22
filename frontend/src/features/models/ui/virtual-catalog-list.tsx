import { type ReactNode, useCallback, useLayoutEffect, useRef, useState } from "react";
import { defaultRangeExtractor, useVirtualizer } from "@tanstack/react-virtual";

/** Height guess for a row before it is measured; real rows are measured and may wrap. */
const ROW_ESTIMATE_PX = 62;
/** Space between rows (the former `gap-1.5`). */
const ROW_GAP_PX = 6;
/** Rows kept mounted beyond each edge of the viewport. */
const OVERSCAN_ROWS = 8;
/** Viewport assumed until the scroll container is measured, so the first render is bounded. */
const INITIAL_VIEWPORT = { width: 960, height: 800 };

interface VirtualCatalogListProps<T extends { key: string }> {
  rows: ReadonlyArray<T>;
  /** The page's one scroll container; the list adds no scrollbar of its own. */
  scrollElement: HTMLElement | null;
  /** Accessible name of the list. */
  label: string;
  /** Rows that stay mounted wherever the scroll is, so they stay in the tab order. */
  pinnedKeys?: ReadonlyArray<string>;
  renderRow: (row: T) => ReactNode;
}

/**
 * Mounts only the catalog rows near the viewport. The virtualizer lives here, so a
 * scroll re-renders this list (tens of rows) and nothing else on the page.
 */
export function VirtualCatalogList<T extends { key: string }>({
  rows,
  scrollElement,
  label,
  pinnedKeys,
  renderRow,
}: VirtualCatalogListProps<T>) {
  const listRef = useRef<HTMLDivElement>(null);
  // Distance from the top of the scroll content to the top of this list. The
  // configured section and the filters above it change height, so it is observed.
  const [scrollMargin, setScrollMargin] = useState(0);
  // The last focused row stays mounted with its neighbours and is never cleared on blur:
  // focusout fires before the next row takes focus, so clearing would unmount the Tab
  // target, and returning from another window or a dialog needs the same row.
  const [focusedKey, setFocusedKey] = useState<string | null>(null);
  const focusedIndex = focusedKey === null ? -1 : rows.findIndex((row) => row.key === focusedKey);

  useLayoutEffect(() => {
    const list = listRef.current;
    if (!scrollElement || !list) return;
    const measure = () => {
      const next = Math.round(
        list.getBoundingClientRect().top -
          scrollElement.getBoundingClientRect().top +
          scrollElement.scrollTop,
      );
      setScrollMargin((prev) => (prev === next ? prev : next));
    };
    measure();
    // A size change in the scroll container, any section in it, or the catalog
    // section (filter wrap, loading and empty notices) can move the list.
    const observer = new ResizeObserver(measure);
    observer.observe(scrollElement);
    for (const child of Array.from(scrollElement.children)) observer.observe(child);
    if (list.parentElement) observer.observe(list.parentElement);
    return () => observer.disconnect();
  }, [scrollElement]);

  // A stable identity: a new one makes the virtualizer rebuild every row's offset.
  const getItemKey = useCallback((index: number) => rows[index]?.key ?? index, [rows]);

  // TanStack Virtual returns imperative functions the React Compiler cannot memoize.
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollElement,
    estimateSize: () => ROW_ESTIMATE_PX,
    getItemKey,
    gap: ROW_GAP_PX,
    scrollMargin,
    overscan: OVERSCAN_ROWS,
    initialRect: INITIAL_VIEWPORT,
    rangeExtractor: (range) => {
      const indexes = defaultRangeExtractor(range);
      const wanted = new Set(indexes);
      const keep = (index: number) => {
        if (index >= 0 && index < rows.length) wanted.add(index);
      };
      if (focusedIndex >= 0) {
        keep(focusedIndex - 1);
        keep(focusedIndex);
        keep(focusedIndex + 1);
      }
      for (const key of pinnedKeys ?? []) keep(rows.findIndex((row) => row.key === key));
      return wanted.size === indexes.length ? indexes : [...wanted].sort((a, b) => a - b);
    },
  });

  const hasRows = rows.length > 0;
  return (
    <>
      {/* Visible to screen-reader browse mode, which skips descriptions on a list. */}
      {hasRows && (
        <p className="sr-only">
          Only nearby rows are loaded. Use the catalog search to reach any other row.
        </p>
      )}
      <div
        ref={listRef}
        role={hasRows ? "list" : undefined}
        aria-label={hasRows ? label : undefined}
        className="relative w-full"
        style={{ height: virtualizer.getTotalSize() }}
      >
        {virtualizer.getVirtualItems().map((item) => {
          const row = rows[item.index];
          if (!row) return null;
          return (
            <div
              key={item.key}
              ref={virtualizer.measureElement}
              data-index={item.index}
              role="listitem"
              aria-setsize={rows.length}
              aria-posinset={item.index + 1}
              className="absolute left-0 top-0 w-full"
              style={{ transform: `translateY(${item.start - scrollMargin}px)` }}
              onFocus={() => setFocusedKey(row.key)}
            >
              {renderRow(row)}
            </div>
          );
        })}
      </div>
    </>
  );
}
