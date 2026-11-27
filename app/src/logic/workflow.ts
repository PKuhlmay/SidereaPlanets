// Step logic of the window: the seven steps of the step bar, which one is done, possible or locked, which one opens
// next, and the texts that go with them (docs/design.md, "Schrittleiste", "Schrittpanel" and "Zustände").

/** How far the calculation has come: the results that exist. */
export type Step = "empty" | "loaded" | "points" | "aligned" | "limited" | "stacked";

/** The seven steps of the step bar; the left panel shows one of them. */
export type ViewStep = "open" | "alignpoints" | "align" | "limit" | "stack" | "wavelets" | "save";

const rank: Record<Step, number> = { empty: -1, loaded: 0, points: 1, aligned: 2, limited: 3, stacked: 4 };

export const atLeast = (step: Step, min: Step): boolean => rank[step] >= rank[min];

export interface StepInfo {
  id: ViewStep;
  /** Name in the step bar and in the panel header. */
  label: string;
  /** Label of the action button. */
  action: string;
  /** The sentence under the name in the panel header. */
  sentence: string;
  /** What the step is, for the tooltip of its pill (docs/design.md, "Texte und Schreibweise"). */
  explanation: string;
  /** What the action button does, for its tooltip. */
  actionExplanation: string;
}

export const viewSteps: StepInfo[] = [
  {
    id: "open",
    label: "Open",
    action: "Select Recording…",
    sentence: "Choose a recording: a SER or AVI video or a series of images. You can also drop it onto the window.",
    explanation: "Open: choose the video or the images to work on.",
    actionExplanation: "Choose the video or the images to work on.",
  },
  {
    id: "alignpoints",
    label: "Alignpoints",
    action: "Set Alignpoints",
    sentence: "Places marks on surface details that every frame is lined up on. Move the slider left for more points, right for fewer.",
    explanation: "Alignpoints: reference marks on surface details that every frame is lined up on.",
    actionExplanation: "Places alignpoints automatically on the frame that is shown.",
  },
  {
    id: "align",
    label: "Align",
    action: "Align",
    sentence: "Lines up every frame on the alignpoints and rates how sharp each one is.",
    explanation: "Align: lines up every frame on the alignpoints and rates how sharp each frame is.",
    actionExplanation: "Lines up every frame on the alignpoints and rates how sharp each frame is.",
  },
  {
    id: "limit",
    label: "Best Frames",
    action: "Keep Best Frames",
    sentence: "Keeps the sharpest frames and drops the rest. Only the kept frames go into the stack.",
    explanation: "Best Frames: choose which frames go into the stack, the sharpest ones.",
    actionExplanation: "Keeps the frames chosen in the settings above and leaves out the others.",
  },
  {
    id: "stack",
    label: "Stack",
    action: "Stack",
    sentence: "Adds the kept frames into one image with far less noise.",
    explanation: "Stack: averages the kept frames into one image with far less noise.",
    actionExplanation: "Averages the kept frames into one image with far less noise.",
  },
  {
    id: "wavelets",
    label: "Wavelets",
    action: "Do All",
    sentence: "Sharpens the stack. Raise the first layers for fine detail, then press Do All to apply them to the whole image.",
    explanation: "Wavelets: sharpens the stack in layers from fine to coarse detail.",
    actionExplanation: "Applies the sharpening to the whole image (the preview shows only the preview area).",
  },
  {
    id: "save",
    label: "Save",
    action: "Save Image",
    sentence: "Writes the finished image to a file.",
    explanation: "Save: writes the finished image to a file.",
    actionExplanation: "Writes the finished image to a file.",
  },
];

export const stepInfo = (id: ViewStep): StepInfo => viewSteps.find((s) => s.id === id) ?? viewSteps[0];
export const stepNumber = (id: ViewStep): number => viewSteps.findIndex((s) => s.id === id) + 1;

/** The label of the button of a done step: running it again. */
export const againLabel = (id: ViewStep): string => (id === "open" ? "Select Another Recording…" : `${stepInfo(id).action} Again`);

/** The step after `id` in the bar, or null after Save. */
export function stepAfter(id: ViewStep): ViewStep | null {
  return viewSteps[stepNumber(id)]?.id ?? null;
}

/** The step before `id` in the bar, or null before Open. */
export function stepBefore(id: ViewStep): ViewStep | null {
  const index = stepNumber(id) - 2;
  return index >= 0 ? viewSteps[index].id : null;
}

/**
 * Everything the step states depend on. `waveletsDone`: the result of Do All matches the stack and the current
 * wavelet values. `saved`: Save image wrote the sharpened image since then.
 */
export interface Progress {
  step: Step;
  waveletsDone: boolean;
  saved: boolean;
}

export type StepStatus = "done" | "possible" | "locked";

export interface StepState {
  status: StepStatus;
  /** Tooltip of a locked pill: the step that is missing. Empty otherwise. */
  hint: string;
}

/** Whether each step's result exists (Wavelets and Save only count with a stack). */
function doneMap(p: Progress): Record<ViewStep, boolean> {
  const stacked = atLeast(p.step, "stacked");
  const wavelets = stacked && p.waveletsDone;
  return {
    open: atLeast(p.step, "loaded"),
    alignpoints: atLeast(p.step, "points"),
    align: atLeast(p.step, "aligned"),
    limit: atLeast(p.step, "limited"),
    stack: stacked,
    wavelets,
    save: wavelets && p.saved,
  };
}

/** The step a step needs before it can run. Save needs only Stack: Save image runs a pending Do All itself (SP-009). */
const requires: Record<ViewStep, Step> = {
  open: "empty",
  alignpoints: "loaded",
  align: "points",
  limit: "aligned",
  stack: "limited",
  wavelets: "stacked",
  save: "stacked",
};

/** The step that opens next: the first one that is not done; Save once everything is done. */
export function nextOpenStep(p: Progress): ViewStep {
  const done = doneMap(p);
  return viewSteps.find((s) => !done[s.id])?.id ?? "save";
}

/** The tooltip of a locked pill names the next open step's action ("Run Align first"). */
function lockHint(p: Progress): string {
  const next = nextOpenStep(p);
  return next === "open" ? "Open a recording first" : `Run ${stepInfo(next).action} first`;
}

export function stepState(id: ViewStep, p: Progress): StepState {
  if (doneMap(p)[id]) return { status: "done", hint: "" };
  if (atLeast(p.step, requires[id])) return { status: "possible", hint: "" };
  return { status: "locked", hint: lockHint(p) };
}

export function stepStates(p: Progress): Record<ViewStep, StepState> {
  return Object.fromEntries(viewSteps.map((s) => [s.id, stepState(s.id, p)])) as Record<ViewStep, StepState>;
}

/**
 * Whether running a done step again throws away results of later steps (the hint under "‹Action› again"). Do All
 * and Save image build nothing that a later step depends on.
 */
export function againDiscards(id: ViewStep, p: Progress): boolean {
  if (id === "wavelets" || id === "save") return false;
  const done = doneMap(p);
  return viewSteps.slice(stepNumber(id)).some((s) => done[s.id]);
}

/** What the image area shows in a step: the current frame, the stack, or the sharpened stack. */
export type ImageKind = "frame" | "stack" | "wavelet";

export function imageKind(view: ViewStep): ImageKind {
  if (view === "stack") return "stack";
  if (view === "wavelets" || view === "save") return "wavelet";
  return "frame";
}

/** The image the image area shows. The stack and the processed image exist once their steps are built. */
export function pickImage<T>(
  view: ViewStep,
  step: Step,
  before: boolean,
  images: { frame: T | null; stack: T | null; processed: T | null },
): T | null {
  const kind = imageKind(view);
  if (kind === "frame") return images.frame;
  // Until the stack is made, the Stack step shows the current frame.
  if (kind === "stack") return step === "stacked" ? (images.stack ?? images.frame) : images.frame;
  return before ? images.stack : (images.processed ?? images.stack);
}

/** The chip at the top left of the image area: what the image shows. */
export function imageCaption(
  view: ViewStep,
  step: Step,
  before: boolean,
  data: { frame: number; frameCount: number; stackSize: number | null; processed: boolean },
): string {
  const kind = imageKind(view);
  const stacked = step === "stacked" && data.stackSize !== null;
  const frame = `Frame ${data.frame + 1} of ${data.frameCount}`;
  if (kind === "frame" || !stacked) return frame;
  if (kind === "wavelet" && data.processed && !before) return "Sharpened stack";
  return `Stack of ${data.stackSize} frames`;
}

/** What the results of the steps are made of. */
export interface ResultData {
  fileName: string;
  frameCount: number;
  alignpoints: number;
  /** Number of frames Align rated. */
  rated: number;
  kept: number;
  stackSize: number | null;
  stackWidth: number;
  stackHeight: number;
  savedName: string;
}

/** The result of a step (result line of the next step, tooltip of a done pill, foot of a re-opened step). */
export function stepResult(id: ViewStep, d: ResultData): string {
  switch (id) {
    case "open":
      return `${d.fileName} · ${d.frameCount} frames`;
    case "alignpoints":
      return `${d.alignpoints} points`;
    case "align":
      return `${d.rated} frames rated`;
    case "limit":
      return `${d.kept} of ${d.frameCount} frames kept`;
    case "stack":
      return `${d.stackSize ?? 0} frames · ${d.stackWidth}×${d.stackHeight}`;
    case "wavelets":
      return "Sharpening applied";
    case "save":
      return d.savedName;
  }
}

export function windowTitle(fileName: string, busy: { name: string; progress: number } | null): string {
  if (busy) return `Siderea Planets · ${busy.name} ${Math.round(busy.progress * 100)}%`;
  return fileName ? `Siderea Planets — ${fileName}` : "Siderea Planets";
}

/** "00:01:05" for 65 seconds. */
export function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds));
  const part = (n: number) => String(n).padStart(2, "0");
  return `${part(Math.floor(total / 3600))}:${part(Math.floor((total % 3600) / 60))}:${part(total % 60)}`;
}

/** Remaining time from the elapsed time and the progress (0..1); unknown at the very start. */
export function timeLeft(elapsedSeconds: number, progress: number): number {
  if (progress <= 0) return 0;
  return (elapsedSeconds * (1 - progress)) / progress;
}