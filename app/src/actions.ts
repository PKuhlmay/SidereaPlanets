// The one place where later tickets plug in the computing functions. A button whose function does not exist yet is
// locked and says "Not available yet" in its tooltip (MAC-014); should an action without a handler run anyway, it
// reports that in the status bar. A ticket that builds a function registers its handler here.
//
//   SP-005  setAlignpoints, align, limit, regionOfInterest
//   SP-006  stack
//   SP-007  process, doAll, loadScheme, saveScheme (registered below, wavelet.ts); resetWavelets is done in the panel
//   SP-008  functions panels, copyTo, loadTo, difference
//   SP-009  saveImage, saveAligned, saveIntegrated (saving.ts); the
//           darkframe/flatfield/reference menu entries are still open
//
// Buttons whose function does not exist (No Align, Scan Frames, Realign with Processed, the Preview button of a layer,
// Clear List, the darkframe, flatfield and reference entries, Show MACRO/Batch Window, About) are locked and marked
// "Not available yet"; they have no action here.
//
// A handler normally calls runJob (jobs.ts) so that progress, Cancel and Pause work.

import { saveAligned, saveImage, saveIntegrated } from "./saving";
import { stack } from "./stack";
import { status } from "./state";
import { advance } from "./steps";
import { loadScheme, process, runDoAll, saveScheme } from "./wavelet";

export const actionLabels = {
  setAlignpoints: "Set Alignpoints",
  align: "Align",
  limit: "Keep Best Frames",
  regionOfInterest: "Draw Region",
  stack: "Stack",
  saveImage: "Save Image",
  saveAligned: "Save Aligned Video",
  saveIntegrated: "Save Averaged Video",
  process: "Update Preview",
  doAll: "Do All",
  loadScheme: "Load Scheme",
  saveScheme: "Save Scheme",
  copyTo: "Remember",
  loadTo: "Restore",
  difference: "Difference",
} as const;

export type ActionId = keyof typeof actionLabels;

export const handlers: Partial<Record<ActionId, () => void | Promise<void>>> = {
  saveImage,
  saveAligned,
  saveIntegrated,
  stack,
  process,
  // A finished Do All opens the Save step.
  doAll: async () => {
    if (await runDoAll()) advance();
  },
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
