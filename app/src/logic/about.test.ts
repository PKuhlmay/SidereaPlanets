import { describe, expect, it } from "vitest";
import { LICENSE_ID, NOTICE_MARKER, SOURCE_URL, themedNotice } from "./about";

const notice = `<!doctype html>\n${NOTICE_MARKER}\n<html lang="en"><head></head><body>MIT</body></html>`;

describe("themedNotice", () => {
  it("puts the theme of the app on the page", () => {
    expect(themedNotice(notice, "light")).toContain('<html data-theme="light" lang="en">');
    expect(themedNotice(notice, "dark")).toContain('<html data-theme="dark" lang="en">');
  });

  it("refuses a page that is not the generated notice", () => {
    // A dev server answers a missing file with its index page.
    expect(themedNotice("<!doctype html><html><body><div id=app></div></body></html>", "dark")).toBeNull();
  });
});

describe("constants", () => {
  it("name the license and a source address", () => {
    expect(LICENSE_ID).toBe("GPL-3.0-or-later");
    expect(SOURCE_URL).toMatch(/^https:\/\/github\.com\//);
  });
});
