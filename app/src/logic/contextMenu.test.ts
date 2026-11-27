import { describe, expect, it } from "vitest";
import { suppressContextMenu, type MenuTarget } from "./contextMenu";

const target = (more: Partial<MenuTarget> = {}): MenuTarget => ({ release: true, tag: "DIV", type: "", editable: false, ...more });

describe("suppressContextMenu (PLANETS-26)", () => {
  it("suppresses the menu on the image, the panels and the documentation in a release build", () => {
    expect(suppressContextMenu(target())).toBe(true);
    expect(suppressContextMenu(target({ tag: "CANVAS" }))).toBe(true);
    expect(suppressContextMenu(target({ tag: "A" }))).toBe(true);
    expect(suppressContextMenu(target({ tag: "BUTTON" }))).toBe(true);
  });

  it("keeps the native menu (copy, paste) in text fields, text areas and editable content", () => {
    expect(suppressContextMenu(target({ tag: "INPUT", type: "text" }))).toBe(false);
    expect(suppressContextMenu(target({ tag: "INPUT", type: "search" }))).toBe(false);
    expect(suppressContextMenu(target({ tag: "INPUT", type: "number" }))).toBe(false);
    expect(suppressContextMenu(target({ tag: "INPUT", type: "" }))).toBe(false);
    expect(suppressContextMenu(target({ tag: "TEXTAREA" }))).toBe(false);
    expect(suppressContextMenu(target({ tag: "DIV", editable: true }))).toBe(false);
  });

  it("suppresses it on inputs that hold no text, whose menu would only offer Reload", () => {
    expect(suppressContextMenu(target({ tag: "INPUT", type: "checkbox" }))).toBe(true);
    expect(suppressContextMenu(target({ tag: "INPUT", type: "radio" }))).toBe(true);
    expect(suppressContextMenu(target({ tag: "INPUT", type: "range" }))).toBe(true);
    expect(suppressContextMenu(target({ tag: "INPUT", type: "button" }))).toBe(true);
  });

  it("keeps the menu everywhere in a dev build, with Inspect Element and Reload", () => {
    expect(suppressContextMenu(target({ release: false }))).toBe(false);
    expect(suppressContextMenu(target({ release: false, tag: "CANVAS" }))).toBe(false);
  });
});
