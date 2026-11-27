// The native context menu of the web view in release builds (PLANETS-26): it offers Reload and, depending on the engine, Back or Inspect,
// none of which belongs in the app. It stays where the system menu does something useful: text fields (Copy, Paste, Select All).

export interface MenuTarget {
  /** A release build; a dev build keeps the menu (Inspect Element, Reload). */
  release: boolean;
  /** `Element.tagName` of the element under the pointer. */
  tag: string;
  /** `HTMLInputElement.type`, "" for other elements. */
  type: string;
  /** `HTMLElement.isContentEditable`. */
  editable: boolean;
}

/** The `type` of an input that holds text; check boxes, radio buttons, sliders and buttons do not. */
const TEXT_INPUTS = ["", "text", "search", "number", "password", "email", "url", "tel"];

/** Whether the default context menu is suppressed for a right-click on this element. */
export function suppressContextMenu(target: MenuTarget): boolean {
  if (!target.release) return false;
  const tag = target.tag.toUpperCase();
  if (target.editable || tag === "TEXTAREA") return false;
  return !(tag === "INPUT" && TEXT_INPUTS.includes(target.type.toLowerCase()));
}
