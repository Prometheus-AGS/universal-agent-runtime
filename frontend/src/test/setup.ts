import "@testing-library/jest-dom/vitest";
import { afterEach, beforeEach } from "vitest";
import { cleanup } from "@testing-library/react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

// Reset the entity graph between every test so state from one test doesn't
// leak into the next. We merge a fresh `entities` slice — passing `true`
// to replace the entire state would wipe out the store's methods too.
beforeEach(() => {
  useGraphStore.setState({
    entities: {},
  } as unknown as Parameters<typeof useGraphStore.setState>[0]);
});

afterEach(() => {
  cleanup();
});
