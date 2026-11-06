import "@testing-library/jest-dom/vitest";
import { afterEach, beforeEach, expect } from "vitest";
import { cleanup } from "@testing-library/react";
import { useGraphStore } from "@/platform/entities/runtime";

// Match index.html before test imports evaluate KaTeX's document-mode check.
if (!document.doctype) {
  document.insertBefore(
    document.implementation.createDocumentType("html", "", ""),
    document.documentElement,
  );
}

// HappyDOM 21.0.4 omits compatMode. Support this fixture's HTML5 document,
// deriving the value from its real doctype rather than masking KaTeX's warning.
// This is not a complete implementation of browser parsing modes.
if (!("compatMode" in document)) {
  Object.defineProperty(document, "compatMode", {
    get() {
      const doctype = document.doctype;
      return doctype?.name === "html" &&
        doctype.publicId === "" &&
        doctype.systemId === ""
        ? "CSS1Compat"
        : "BackCompat";
    },
  });
}

// Reset the entity graph between every test so state from one test doesn't
// leak into the next. We merge a fresh `entities` slice — passing `true`
// to replace the entire state would wipe out the store's methods too.
beforeEach(() => {
  expect(document.doctype?.name).toBe("html");
  expect(document.doctype?.publicId).toBe("");
  expect(document.doctype?.systemId).toBe("");
  expect(document.compatMode).toBe("CSS1Compat");
  useGraphStore.setState({
    entities: {},
  } as unknown as Parameters<typeof useGraphStore.setState>[0]);
});

afterEach(() => {
  cleanup();
});
