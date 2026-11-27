import { describe, expect, it } from "vitest";
// @ts-expect-error plain JavaScript module without type declarations (it runs under node, see package.json)
import { isAllowed } from "./licenses.mjs";
// @ts-expect-error plain JavaScript module without type declarations
import { MARKER, renderNotice } from "./notice.mjs";

const allowed = ["MIT", "Apache-2.0", "BSD-3-Clause"];
const check = (expression: string): boolean => isAllowed(expression, allowed);

// The npm license policy of `pnpm check:licenses` (PLANETS-7).
describe("isAllowed", () => {
  it("accepts an allowed identifier and rejects others", () => {
    expect(check("MIT")).toBe(true);
    expect(check("GPL-2.0-only")).toBe(false);
    expect(check("")).toBe(false);
  });

  it("OR needs one allowed side, AND needs both", () => {
    expect(check("(MIT OR GPL-2.0-only)")).toBe(true);
    expect(check("GPL-2.0-only OR AGPL-3.0-only")).toBe(false);
    expect(check("MIT AND Apache-2.0")).toBe(true);
    expect(check("MIT AND GPL-2.0-only")).toBe(false);
  });

  it("lets AND bind tighter than OR and honours parentheses", () => {
    expect(check("GPL-2.0-only AND MIT OR Apache-2.0")).toBe(true);
    expect(check("GPL-2.0-only AND (MIT OR Apache-2.0)")).toBe(false);
  });

  it("judges an exception by its base license", () => {
    expect(check("Apache-2.0 WITH LLVM-exception")).toBe(true);
    expect(check("GPL-2.0-only WITH Classpath-exception-2.0")).toBe(false);
  });
});

describe("renderNotice", () => {
  const mit = { name: "a", version: "1.0.0", license: "MIT", homepage: "", text: "MIT text <b>" };

  it("prints a license text once for all packages that share it, escaped", () => {
    const html = renderNotice([mit, { ...mit, name: "b" }]);
    expect(html).toContain(MARKER);
    expect(html.match(/<section>/g)).toHaveLength(1);
    expect(html).toContain("<li>a 1.0.0</li><li>b 1.0.0</li>");
    expect(html).toContain("MIT text &lt;b&gt;");
  });

  it("writes a placeholder page that says why the list is missing", () => {
    const html = renderNotice([], "cargo-about missing");
    expect(html).toContain(MARKER);
    expect(html).toContain("cargo-about missing");
    expect(html).not.toContain("<section>");
  });
});
