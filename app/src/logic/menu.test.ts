import { describe, expect, it } from "vitest";
import { openerClass } from "./menu";

describe("openerClass", () => {
  it("gives every opener of a menu the arrow, whatever kind of button it is", () => {
    expect(openerClass({})).toBe("ghost caret");
    expect(openerClass({ plain: true })).toBe("caret");
    expect(openerClass({ plain: true, filled: true })).toBe("next caret");
  });
});
