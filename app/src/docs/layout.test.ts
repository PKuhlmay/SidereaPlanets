import { describe, expect, it } from "vitest";

// The look of the windows is plain CSS; these tests pin the few rules whose absence was found in the real window
// (RK-010): a text column wider than the window, the default Windows scrollbar and the dimming of locked fields.
// @ts-expect-error type error without @types/node package
import { readFileSync } from "node:fs";

const read = (path: string): string => (readFileSync as (file: URL, encoding: "utf8") => string)(new URL(path, import.meta.url), "utf8");
const docsView = read("./DocsView.vue");
const styles = read("../styles.css");
const numberField = read("../components/NumberField.vue");

/** Body of the first rule whose selector is exactly `selector`. */
function rule(css: string, selector: string): string {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\:]/g, "\\$&");
  const match = new RegExp(`(?:^|\\})\\s*${escaped}\\s*\\{([^}]*)\\}`, "m").exec(css);
  if (!match) throw new Error(`No rule for ${selector}`);
  return match[1];
}

describe("documentation text column", () => {
  const page = rule(docsView, ".page");

  it("shrinks with the window instead of adding its padding to a fixed width", () => {
    expect(page).toContain("box-sizing: border-box");
    expect(page).toContain("width: 100%");
    expect(page).not.toMatch(/width:\s*720px/);
  });

  it("stays at 720 px of text (plus padding) in a wide window", () => {
    expect(page).toMatch(/max-width:\s*816px/);
  });
});

describe("scrollbars", () => {
  it("are narrow, without arrow buttons, in colours of the tokens only", () => {
    expect(rule(styles, "*::-webkit-scrollbar")).toMatch(/width:\s*\d+px/);
    expect(rule(styles, "*::-webkit-scrollbar-button")).toContain("display: none");
    const colours = rule(styles, "*::-webkit-scrollbar-thumb") + rule(styles, "*::-webkit-scrollbar-thumb:hover");
    expect(colours).toContain("var(--");
    expect(colours).not.toMatch(/#[0-9a-f]{3,6}|rgb/i);
  });
});

describe("number field in a locked panel", () => {
  // Outside a label row nothing else greys the text, so the field has to do it itself (the Limit Setup values).
  it("is greyed out with its own text colour too, also inside a disabled fieldset", () => {
    expect(numberField).toMatch(/fieldset:disabled\s+\.num\s*\{[^}]*color:\s*var\(--off\)[^}]*opacity:\s*0\.45/);
    expect(numberField).toMatch(/\.num:has\(input:disabled\)/);
  });
});
