import { describe, expect, it } from "vitest";
import { fileName, MAX_RECENT, pushRecent, recentLabel } from "./recent";

describe("pushRecent", () => {
  it("puts the newest selection first", () => {
    expect(pushRecent([["a.ser"]], ["b.ser"])).toEqual([["b.ser"], ["a.ser"]]);
  });

  it("does not list the same selection twice", () => {
    expect(pushRecent([["a.ser"], ["b.ser"]], ["b.ser"])).toEqual([["b.ser"], ["a.ser"]]);
    expect(pushRecent([["a.png", "b.png"]], ["a.png", "b.png"])).toEqual([["a.png", "b.png"]]);
  });

  it("keeps a different order of the same files apart", () => {
    expect(pushRecent([["a.png", "b.png"]], ["b.png", "a.png"])).toHaveLength(2);
  });

  it("keeps at most ten entries", () => {
    const list = Array.from({ length: MAX_RECENT }, (_, i) => [`${i}.ser`]);
    const next = pushRecent(list, ["new.ser"]);
    expect(next).toHaveLength(MAX_RECENT);
    expect(next[0]).toEqual(["new.ser"]);
    expect(next[next.length - 1]).toEqual([`${MAX_RECENT - 2}.ser`]);
  });
});

describe("recentLabel", () => {
  it("shows the file name without the folder", () => {
    expect(fileName("C:\\Data\\JUPITER.AVI")).toBe("JUPITER.AVI");
    expect(fileName("/home/me/moon.ser")).toBe("moon.ser");
  });

  it("adds the number of further files of a sequence", () => {
    expect(recentLabel(["C:\\x\\a.png"])).toBe("a.png");
    expect(recentLabel(["C:\\x\\a.png", "C:\\x\\b.png", "C:\\x\\c.png"])).toBe("a.png (+2)");
  });
});
