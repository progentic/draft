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

describe("explicit page-break presentation", () => {
  it("renders explicit page breaks as full page-surface gaps", () => {
    expect(styles).toMatch(
      /\.draft-editor \[data-draft-page-break\] \{[^}]*display: block;[^}]*height: 34px;[^}]*background: var\(--workspace\);[^}]*border-top: 1px solid[^}]*border-bottom: 1px solid[^}]*box-shadow:/su,
    );
    expect(styles).not.toMatch(
      /\.draft-editor \[data-draft-page-break\] \{[^}]*border-top: 1px dashed/su,
    );
  });

  it("keeps the page gap full width at normal and narrow editor padding", () => {
    expect(styles).toContain("--page-edge-offset: 73px;");
    expect(styles).toContain("--page-edge-offset: 53px;");
    expect(styles).toContain("--page-edge-offset: 24px;");
  });
});
