// The step bar in the window: the state of the seven steps from `session` (logic/workflow.ts), moving on to the next
// open step after a run, and the remembered open/closed state of "Advanced" in every step panel.

import { computed, reactive, watch } from "vue";
import { nextOpenStep, stepResult, stepStates, viewSteps, type Progress, type ResultData, type ViewStep } from "./logic/workflow";
import { session, settings } from "./state";
import { doAllIsUpToDate } from "./wavelet";

export const progress = computed<Progress>(() => ({
  step: session.step,
  waveletsDone: session.step === "stacked" && doAllIsUpToDate(),
  saved: session.saved,
}));

export const states = computed(() => stepStates(progress.value));
export const nextOpen = computed(() => nextOpenStep(progress.value));

/** What the result texts of the steps are made of. */
export const resultData = computed<ResultData>(() => {
  return {
    fileName: session.fileName,
    frameCount: session.info?.frameCount ?? session.frames.length,
    alignpoints: session.alignpoints.length,
    rated: session.frames.filter((f) => f.quality !== null).length,
    kept: session.frames.filter((f) => f.checked).length,
    stackSize: session.stackSize,
    stackWidth: session.stackImage?.width ?? 0,
    stackHeight: session.stackImage?.height ?? 0,
    savedName: session.savedName,
  };
});

/** The result of a step, as the result line, the tooltip of a done pill and the foot of a re-opened step show it. */
export const resultOf = (id: ViewStep): string => stepResult(id, resultData.value);

/** After a successful run the panel opens the next open step (after Select Alignpoints, after Do All Save). */
export function advance(): void {
  session.view = nextOpen.value;
}

// Save keeps its check mark only while the saved image is the current Do All result: changing a wavelet value, a new
// stack or running an earlier step again takes it away for good.
watch(
  () => progress.value.waveletsDone,
  (done) => {
    if (!done) session.saved = false;
  },
);

// A step that became locked (Set Alignpoints or Align ran again and dropped what came after) cannot stay open.
watch(
  () => states.value[session.view].status,
  (status) => {
    if (status === "locked") advance();
  },
);

const ADVANCED_KEY = "sp-advanced";

function loadAdvanced(): Partial<Record<ViewStep, boolean>> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(ADVANCED_KEY) ?? "{}");
    if (typeof parsed !== "object" || parsed === null) return {};
    return Object.fromEntries(Object.entries(parsed).filter(([, open]) => typeof open === "boolean"));
  } catch {
    return {};
  }
}

/** Which step panels show their Advanced settings; kept until closed again, also after a restart. */
export const advancedOpen = reactive<Partial<Record<ViewStep, boolean>>>(loadAdvanced());

function saveAdvanced(): void {
  try {
    localStorage.setItem(ADVANCED_KEY, JSON.stringify(advancedOpen));
  } catch {
    // Storage can be blocked; the panels then start closed next time.
  }
}

export function toggleAdvanced(id: ViewStep): void {
  advancedOpen[id] = !advancedOpen[id];
  saveAdvanced();
}

// Settings > Show Advanced Controls opens the Advanced section of every step when it is ticked. Each section can be
// closed and opened on its own afterwards; unticking leaves them as they are.
watch(
  () => settings.menu.advancedControls,
  (on) => {
    if (!on) return;
    viewSteps.forEach((step) => (advancedOpen[step.id] = true));
    saveAdvanced();
  },
);
