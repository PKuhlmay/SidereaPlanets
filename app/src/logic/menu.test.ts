import { describe, expect, it } from "vitest";
import { openerClass, showItems } from "./menu";

describe("openerClass", () => {
  it("gives every opener of a menu the arrow, whatever kind of button it is", () => {
    expect(openerClass({})).toBe("ghost caret");
    expect(openerClass({ plain: true })).toBe("caret");
    expect(openerClass({ plain: true, filled: true })).toBe("next caret");
  });
});

describe("showItems (Show menu at the top right of the image)", () => {
  const titles = (kind: Parameters<typeof showItems>[0], available: boolean) =>
    showItems(kind)
      .filter((item) => item.available === available)
      .map((item) => item.title);

  it("names the entries of the frame steps with clear words and keeps the ones that work at the top", () => {
    expect(titles("frame", true)).toEqual(["Show Region", "Show Alignpoints", "Show Frame List", "Show Quality Graph"]);
    expect(titles("frame", false)).toEqual(["Show Full Image", "Show Align Data", "Show Prefilter"]);
  });

  it("offers the card of the alignpoints in the Stack step and locks the graph and the list", () => {
    expect(titles("stack", true)).toEqual(["Show Alignpoints"]);
    expect(titles("stack", false)).toEqual(["Show Full Image", "Show Stack Graph", "Show Stack List"]);
  });

  it("calls the processing area of the Wavelets and Save steps the preview area", () => {
    expect(titles("wavelet", true)).toEqual(["Show Alignpoints", "Show Preview Area"]);
    expect(titles("wavelet", false)).toEqual(["Show Full Image"]);
  });
});
