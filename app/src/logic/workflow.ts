// Step logic of the window: which step is next, which buttons and tabs are locked (docs/design.md, "Zustände").

export type Step = "empty" | "loaded" | "points" | "aligned" | "limited" | "stacked";
export type Tab = "align" | "stack" | "wavelet";

const rank: Record<Step, number> = { empty: -1, loaded: 0, points: 1, aligned: 2, limited: 3, stacked: 4 };

export const atLeast = (step: Step, min: Step): boolean => rank[step] >= rank[min];

// Same order and labels as the original; a tab opens once the step before it is done.
export const tabs: { id: Tab; label: string; requires: Step; hint: string }[] = [
  { id: "align", label: "Align", requires: "empty", hint: "" },
  { id: "stack", label: "Stack", requires: "limited", hint: "Run Limit first" },
  { id: "wavelet", label: "Wavelet", requires: "stacked", hint: "Run Stack first" },
];

export function tabLocked(step: Step, tab: Tab): boolean {
  const entry = tabs.find((t) => t.id === tab);
  return !entry || !atLeast(step, entry.requires);
}

export type StepId = "setAlignpoints" | "align" | "limit" | "stack" | "saveImage";

export interface StepButton {
  disabled: boolean;
  /** The one button of the sequence that is filled. */
  next: boolean;
  /** Shows the check mark. */
  done: boolean;
}

// Set Alignpoints and Align stay usable once they ran, also after Limit, Stack and Wavelet: running one again drops
// everything built on the old alignment (a deliberate difference from the original, docs/design.md, "Zustände").
export function stepButtons(step: Step, busy: boolean): Record<StepId, StepButton> {
  const r = rank[step];
  const button = (enabled: boolean, next: boolean, done: boolean): StepButton => ({
    disabled: !enabled || busy,
    next: next && enabled && !busy,
    done,
  });
  return {
    setAlignpoints: button(r >= 0, r === 0, r >= 1),
    align: button(r >= 1, r === 1, r >= 2),
    limit: button(r >= 2, r === 2, r >= 3),
    stack: button(r >= 3, r === 3, r >= 4),
    saveImage: button(r >= 4, false, false),
  };
}

export function windowTitle(fileName: string, busy: { name: string; progress: number } | null): string {
  if (busy) return `SidereaPlanets · ${busy.name} ${Math.round(busy.progress * 100)}%`;
  return fileName ? `SidereaPlanets — ${fileName}` : "SidereaPlanets";
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

/** The image the image area shows. The stack and the processed image exist once their steps are built. */
export function pickImage<T>(
  tab: Tab,
  step: Step,
  before: boolean,
  images: { frame: T | null; stack: T | null; processed: T | null },
): T | null {
  if (tab === "align") return images.frame;
  // Until the stack is made, the Stack tab shows the current frame.
  if (tab === "stack") return step === "stacked" ? (images.stack ?? images.frame) : images.frame;
  return before ? images.stack : (images.processed ?? images.stack);
}
