import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

const styles = readFileSync(resolve("src/styles.css"), "utf8");

describe("workspace motion policy", () => {
  it("keeps the normal layout transition timings", () => {
    expect(styles).toContain("transition: grid-template-columns 160ms ease;");
    expect(styles).toContain("transition: visibility 160ms ease;");
  });

  it("removes both workspace transitions when reduced motion is requested", () => {
    expect(styles).toMatch(
      /@media \(prefers-reduced-motion: reduce\) \{\s*\.workspace-body,\s*\.outline-panel \{\s*transition: none;\s*\}\s*\}/u,
    );
  });
});
