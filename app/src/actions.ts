// The one place where later tickets plug in the computing functions. Every button whose function does not exist
// yet reports "Not available yet" in the status bar; a ticket that builds a function registers its handler here.
//
//   RK-005  setAlignpoints, align, limit, noAlign, regionOfInterest, scanFrames, emptyList
//   RK-006  stack, saveAligned, saveIntegrated
//   RK-007  process, doAll, loadScheme, saveScheme (registered below, wavelet.ts); realign and stackAgain wait for
//           RK-005/RK-006; resetWavelets is done in the panel
//   RK-008  functions panels, copyTo, loadTo, difference
//   RK-009  saveImage, darkframe/flatfield/reference menu entries
//
// A handler normally calls runJob (jobs.ts) so that progress, Cancel and Pause work.

import { stack } from "./stack";
import { status } from "./state";
import { doAll, loadScheme, process, saveScheme } from "./wavelet";

export const actionLabels = {
  setAlignpoints: "Set Alignpoints",
  align: "Align",
  limit: "Limit",
  noAlign: "No Align",
  regionOfInterest: "R.o.Interest",
  scanFrames: "Scan Frames",
  emptyList: "Empty List",
  stack: "Stack",
  saveImage: "Save image",
  saveAligned: "Save Aligned",
  saveIntegrated: "Save Integrated",
  process: "Process",
  doAll: "Do All",
  realign: "Realign with Processed",
  loadScheme: "Load Scheme",
  saveScheme: "Save Scheme",
  copyTo: "Copy To",
  loadTo: "Load to",
  difference: "Difference",
  previewLayer: "Preview",
  loadDarkframe: "Load Darkframe",
  loadFlatfield: "Load Flatfield",
  loadReferenceframe: "Load Referenceframe",
  macroBatch: "Show MACRO/Batch window",
  about: "About",
} as const;

export type ActionId = keyof typeof actionLabels;

export const handlers: Partial<Record<ActionId, () => void | Promise<void>>> = {
  stack,
  process,
  doAll,
  loadScheme,
  saveScheme,
};

/** The message shown in the status bar for a button without a function. */
export const unavailableMessage = (id: ActionId): string => `${actionLabels[id]}: not available yet`;

/** Runs the handler of a button, or tells that the function does not exist yet. */
export async function runAction(id: ActionId): Promise<void> {
  const handler = handlers[id];
  if (handler) await handler();
  else status.result = unavailableMessage(id);
}
