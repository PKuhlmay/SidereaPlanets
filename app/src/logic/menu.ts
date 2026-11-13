/**
 * The classes of the button that opens a menu (MenuButton.vue): borderless in the toolbar, `plain` as a standard button
 * and `filled` as the action of a step panel. All of them carry the arrow, so that a button that opens a list of
 * choices looks like one (Save image in the Stack and Save steps).
 */
export function openerClass(kind: { plain?: boolean; filled?: boolean }): string {
  if (kind.filled) return "next caret";
  return kind.plain ? "caret" : "ghost caret";
}

/** One check mark of the Show menu; `available` is false for the ones that change nothing on screen yet. */
export interface ShowItem {
  title: string;
  /** The setting behind the check mark, in `settings.align`, `settings.stack` or `settings.wavelet`. */
  key: string;
  available: boolean;
}

const item = (title: string, key: string, available = true): ShowItem => ({ title, key, available });

/**
 * The entries of the Show menu at the top right of the image, per kind of image, in menu order (MAC-014). The ones
 * without an effect are dimmed under "Not available yet" at the end of the menu.
 */
export function showItems(kind: "frame" | "stack" | "wavelet"): ShowItem[] {
  if (kind === "stack") {
    return [
      item("Show Full Image", "showFullImage", false),
      item("Show Alignpoints", "showAlignSections"),
      item("Show Stack Graph", "showStackgraph", false),
      item("Show Stack List", "showStacklist", false),
    ];
  }
  if (kind === "wavelet") {
    return [
      item("Show Full Image", "showFullImage", false),
      item("Show Alignpoints", "showAlignpoints"),
      item("Show Preview Area", "showProcessingArea"),
    ];
  }
  return [
    item("Show Full Image", "showFullImage", false),
    item("Show Region", "showRoi"),
    item("Show Alignpoints", "showAlignpoints"),
    item("Show Align Data", "showAligndata", false),
    item("Show Frame List", "showFramelist"),
    item("Show Quality Graph", "showRegistrationgraph"),
    item("Show Prefilter", "showPrefilter", false),
  ];
}
