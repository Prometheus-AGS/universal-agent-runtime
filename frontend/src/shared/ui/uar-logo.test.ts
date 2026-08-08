import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, test } from "vitest";

function filesBelow(root: string, prefix = ""): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) return filesBelow(resolve(root, entry.name), relative);
    return [relative];
  });
}

describe("UAR brand contract", () => {
  const projectRoot = resolve(import.meta.dirname, "../../..");

  test("ships every delivered asset except operating-system metadata", () => {
    const delivered = filesBelow(resolve(projectRoot, "../docs/ui/logo"))
      .filter((file) => file.split("/").at(-1) !== ".DS_Store")
      .sort();
    const publicAssets = filesBelow(resolve(projectRoot, "public/brand")).sort();

    expect(publicAssets).toEqual(delivered);
    expect(publicAssets).not.toContain(".DS_Store");
  });

  test("selects delivered light and dark favicons", () => {
    const html = readFileSync(resolve(projectRoot, "index.html"), "utf8");

    expect(html.indexOf('href="/brand/uar-favicon-dark.svg" />')).toBeLessThan(
      html.indexOf('/brand/uar-favicon-light.svg" media="(prefers-color-scheme: light)"'),
    );
    expect(html).toContain('/brand/uar-favicon-light.svg" media="(prefers-color-scheme: light)"');
    expect(html).toContain('/brand/uar-favicon-dark.svg" media="(prefers-color-scheme: dark)"');
    expect(html).not.toContain('href="/favicon.svg"');
  });

  test("declares install icons at two delivered sizes without claiming maskable artwork", () => {
    const manifest = JSON.parse(
      readFileSync(resolve(projectRoot, "public/manifest.json"), "utf8"),
    ) as { icons: Array<{ sizes: string; purpose: string }> };

    expect(manifest.icons.map(({ sizes }) => sizes)).toEqual(["256x256", "512x512"]);
    expect(manifest.icons.every(({ purpose }) => purpose === "any")).toBe(true);
  });
});
