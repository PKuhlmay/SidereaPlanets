// The commands of the Mac menu bar and of the keyboard shortcuts (logic/commands.ts, logic/keys.ts): what each does and
// whether it can run now. The toolbar buttons keep calling their functions directly; these call the same ones.

import { computed, reactive } from "vue";
import { cancelJob, togglePause } from "./jobs";
import { commandEnabled, type CommandId, type CommandState } from "./logic/commands";
import { atLeast, viewSteps, type ViewStep } from "./logic/workflow";
import { openHelp } from "./helpWindow";
import { chooseFiles, recent } from "./recording";
import { openProject, saveImage, saveProject } from "./saving";
import { about, session, status } from "./state";
import { states } from "./steps";
import { viewerControl } from "./viewerControl";

/** `settingsRequest` counts the requests for the Settings menu of the toolbar (Cmd+, and Siderea Planets ▸ Settings…). */
export const ui = reactive({ settingsRequest: 0 });

export const commandState = computed<CommandState>(() => ({
  loaded: session.step !== "empty",
  busy: status.busy !== null,
  stacked: atLeast(session.step, "stacked"),
  hasImage: session.frameImage !== null || session.stackImage !== null || session.processedImage !== null,
  paused: status.busy?.paused ?? false,
  stepOpenable: Object.fromEntries(viewSteps.map((step) => [step.id, states.value[step.id].status !== "locked"])) as Record<ViewStep, boolean>,
  recent: recent.list,
}));

/** Opens a step like a click on the step bar: not a locked step, and not during a calculation. */
export function goToStep(step: ViewStep): void {
  if (status.busy === null && states.value[step].status !== "locked") session.view = step;
}

/** Runs a command if it is possible now. */
export async function runCommand(id: CommandId): Promise<void> {
  if (!commandEnabled(id, commandState.value)) return;
  switch (id) {
    case "selectRecording":
      return chooseFiles();
    case "openProject":
      return openProject();
    case "saveImage":
      return saveImage();
    case "saveProject":
      return saveProject();
    case "cancel":
      return cancelJob();
    case "togglePause":
      return togglePause();
    case "settings":
      ui.settingsRequest++;
      return;
    case "help":
      return openHelp();
    case "about":
      about.open = true;
      return;
    case "zoomIn":
      return viewerControl.zoom?.(1.25);
    case "zoomOut":
      return viewerControl.zoom?.(1 / 1.25);
    case "zoomFit":
      return viewerControl.fit?.();
    case "zoomActual":
      return viewerControl.actualSize?.();
  }
}
