/**
 * The classes of the button that opens a menu (MenuButton.vue): borderless in the toolbar, `plain` as a standard button
 * and `filled` as the action of a step panel. All of them carry the arrow, so that a button that opens a list of
 * choices looks like one (Save image in the Stack and Save steps).
 */
export function openerClass(kind: { plain?: boolean; filled?: boolean }): string {
  if (kind.filled) return "next caret";
  return kind.plain ? "caret" : "ghost caret";
}
