import { describe, expect, it } from "vitest";
import {
  againDiscards,
  againLabel,
  atLeast,
  formatDuration,
  imageCaption,
  imageKind,
  nextOpenStep,
  pickImage,
  stepAfter,
  stepBefore,
  stepInfo,
  stepNumber,
  stepResult,
  stepStates,
  timeLeft,
  viewSteps,
  windowTitle,
  type Progress,
  type Step,
} from "./workflow";

const at = (step: Step, waveletsDone = false, saved = false): Progress => ({ step, waveletsDone, saved });

/** The pills of the step bar as in the table of docs/design.md, "Zustände": ✓ done, ● next open, ○ possible, – locked. */
function pills(p: Progress): string {
  const states = stepStates(p);
  const next = nextOpenStep(p);
  return viewSteps
    .map((s) => {
      const state = states[s.id].status;
      if (state === "done") return "✓";
      if (state === "locked") return "–";
      return s.id === next ? "●" : "○";
    })
    .join(" ");
}

describe("the seven steps", () => {
  it("come in the order of the step bar with the names for newcomers", () => {
    expect(viewSteps.map((s) => s.label)).toEqual(["Open", "Alignpoints", "Align", "Best Frames", "Stack", "Wavelets", "Save"]);
    expect(viewSteps.map((s) => s.action)).toEqual([
      "Select Recording…",
      "Set Alignpoints",
      "Align",
      "Keep Best Frames",
      "Stack",
      "Do All",
      "Save Image",
    ]);
  });

  it("name the button of a done step in Title Case, Open with Select Another Recording…", () => {
    expect(viewSteps.map((s) => againLabel(s.id))).toEqual([
      "Select Another Recording…",
      "Set Alignpoints Again",
      "Align Again",
      "Keep Best Frames Again",
      "Stack Again",
      "Do All Again",
      "Save Image Again",
    ]);
  });

  it("explain each step and its action in a tooltip", () => {
    expect(stepInfo("limit").explanation).toBe("Best Frames: choose which frames go into the stack, the sharpest ones.");
    expect(stepInfo("alignpoints").actionExplanation).toBe("Places alignpoints automatically on the frame that is shown.");
    expect(viewSteps.every((s) => s.explanation.startsWith(`${s.label}: `))).toBe(true);
    expect(viewSteps.every((s) => s.actionExplanation.endsWith("."))).toBe(true);
  });

  it("tell newcomers in the panel header how Open, Alignpoints and Wavelets are used", () => {
    expect(stepInfo("open").sentence).toBe("Choose a recording: a SER or AVI video or a series of images. You can also drop it onto the window.");
    expect(stepInfo("alignpoints").sentence).toBe(
      "Places marks on surface details that every frame is lined up on. Move the slider left for more points, right for fewer.",
    );
    expect(stepInfo("wavelets").sentence).toBe(
      "Sharpens the stack. Raise the first layers for fine detail, then press Do All to apply them to the whole image.",
    );
  });

  it("are numbered from 1 to 7 and know their neighbours", () => {
    expect(stepNumber("open")).toBe(1);
    expect(stepNumber("save")).toBe(7);
    expect(stepAfter("limit")).toBe("stack");
    expect(stepAfter("save")).toBeNull();
    expect(stepBefore("alignpoints")).toBe("open");
    expect(stepBefore("open")).toBeNull();
  });
});

describe("step states (docs/design.md, Zustände)", () => {
  it("follow the table row by row", () => {
    expect(pills(at("empty"))).toBe("● – – – – – –");
    expect(pills(at("loaded"))).toBe("✓ ● – – – – –");
    expect(pills(at("points"))).toBe("✓ ✓ ● – – – –");
    expect(pills(at("aligned"))).toBe("✓ ✓ ✓ ● – – –");
    expect(pills(at("limited"))).toBe("✓ ✓ ✓ ✓ ● – –");
    expect(pills(at("stacked"))).toBe("✓ ✓ ✓ ✓ ✓ ● ○");
    expect(pills(at("stacked", true))).toBe("✓ ✓ ✓ ✓ ✓ ✓ ●");
    expect(pills(at("stacked", true, true))).toBe("✓ ✓ ✓ ✓ ✓ ✓ ✓");
  });

  it("open the next step after each run, and Save stays open once everything is done", () => {
    expect(nextOpenStep(at("empty"))).toBe("open");
    expect(nextOpenStep(at("loaded"))).toBe("alignpoints");
    expect(nextOpenStep(at("points"))).toBe("align");
    expect(nextOpenStep(at("aligned"))).toBe("limit");
    expect(nextOpenStep(at("limited"))).toBe("stack");
    expect(nextOpenStep(at("stacked"))).toBe("wavelets");
    expect(nextOpenStep(at("stacked", true))).toBe("save");
    expect(nextOpenStep(at("stacked", true, true))).toBe("save");
  });

  it("name the missing step in the tooltip of a locked pill", () => {
    expect(stepStates(at("empty")).alignpoints.hint).toBe("Open a recording first");
    expect(stepStates(at("loaded")).align.hint).toBe("Run Set Alignpoints first");
    expect(stepStates(at("points")).limit.hint).toBe("Run Align first");
    expect(stepStates(at("aligned")).stack.hint).toBe("Run Keep Best Frames first");
    expect(stepStates(at("points")).stack.hint).toBe("Run Align first");
    expect(stepStates(at("limited")).wavelets.hint).toBe("Run Stack first");
    expect(stepStates(at("limited")).save.hint).toBe("Run Stack first");
  });

  it("give no hint to steps that can be opened", () => {
    const states = stepStates(at("stacked"));
    expect(Object.values(states).filter((s) => s.status !== "locked").every((s) => s.hint === "")).toBe(true);
  });

  it("make Save possible right after Stack, before Do All", () => {
    expect(stepStates(at("stacked")).save.status).toBe("possible");
  });

  it("take the check mark from Wavelets and Save when the wavelet values change after Do All", () => {
    // waveletsDone turns false when a layer moves; Save loses its check mark with it, even if it was saved.
    const states = stepStates(at("stacked", false, true));
    expect(states.wavelets.status).toBe("possible");
    expect(states.save.status).toBe("possible");
    expect(nextOpenStep(at("stacked", false, true))).toBe("wavelets");
  });

  it("ignore a stale Do All or save when there is no stack", () => {
    const states = stepStates(at("limited", true, true));
    expect(states.wavelets.status).toBe("locked");
    expect(states.save.status).toBe("locked");
    expect(nextOpenStep(at("limited", true, true))).toBe("stack");
  });

  it("fall back to the step after the points when Set Alignpoints or Align runs again", () => {
    // A new run drops Limit, Stack and the sharpened image: the state goes back to "points" or "aligned".
    expect(pills(at("points", true, true))).toBe("✓ ✓ ● – – – –");
    expect(nextOpenStep(at("aligned", true, true))).toBe("limit");
  });

  it("keep Open possible without a recording and Set Alignpoints and Align usable after they ran", () => {
    expect(stepStates(at("empty")).open.status).toBe("possible");
    const states = stepStates(at("stacked", true));
    expect(states.alignpoints.status).toBe("done");
    expect(states.align.status).toBe("done");
  });
});

describe("againDiscards", () => {
  it("warns when a later step has a result", () => {
    expect(againDiscards("alignpoints", at("stacked"))).toBe(true);
    expect(againDiscards("limit", at("stacked"))).toBe(true);
    expect(againDiscards("stack", at("stacked", true))).toBe(true);
    expect(againDiscards("open", at("loaded"))).toBe(false);
    expect(againDiscards("alignpoints", at("points"))).toBe(false);
  });

  it("never warns for Do All and Save image", () => {
    expect(againDiscards("wavelets", at("stacked", true, true))).toBe(false);
    expect(againDiscards("save", at("stacked", true, true))).toBe(false);
  });
});

describe("step results (docs/design.md, Schrittpanel)", () => {
  const data = {
    fileName: "JUPITER.AVI",
    frameCount: 10,
    alignpoints: 82,
    rated: 10,
    kept: 8,
    stackSize: 8,
    stackWidth: 1024,
    stackHeight: 680,
    savedName: "JUPITER.tif",
  };

  it("use the texts of the table", () => {
    expect(viewSteps.map((s) => stepResult(s.id, data))).toEqual([
      "JUPITER.AVI · 10 frames",
      "82 points",
      "10 frames rated",
      "8 of 10 frames kept",
      "8 frames · 1024×680",
      "Sharpening applied",
      "JUPITER.tif",
    ]);
  });
});

describe("atLeast", () => {
  it("compares steps by their order", () => {
    expect(atLeast("aligned", "points")).toBe(true);
    expect(atLeast("points", "aligned")).toBe(false);
    expect(atLeast("empty", "loaded")).toBe(false);
  });
});

describe("windowTitle", () => {
  it("shows the file name after loading", () => {
    expect(windowTitle("JUPITER.AVI", null)).toBe("Siderea Planets — JUPITER.AVI");
    expect(windowTitle("", null)).toBe("Siderea Planets");
  });

  it("shows step and percent during a calculation", () => {
    expect(windowTitle("JUPITER.AVI", { name: "Align", progress: 0.4 })).toBe("Siderea Planets · Align 40%");
  });
});

describe("time display", () => {
  it("formats as hh:mm:ss", () => {
    expect(formatDuration(0)).toBe("00:00:00");
    expect(formatDuration(65)).toBe("00:01:05");
    expect(formatDuration(3725.4)).toBe("01:02:05");
  });

  it("estimates the time left from the progress", () => {
    expect(timeLeft(4, 0.4)).toBeCloseTo(6);
    expect(timeLeft(4, 0)).toBe(0);
    expect(timeLeft(10, 1)).toBe(0);
  });
});

describe("imageKind", () => {
  it("shows frames up to Limit, the stack in Stack and the sharpened stack in Wavelets and Save", () => {
    expect(viewSteps.map((s) => imageKind(s.id))).toEqual(["frame", "frame", "frame", "frame", "stack", "wavelet", "wavelet"]);
  });
});

describe("pickImage", () => {
  const images = { frame: "frame", stack: "stack", processed: "processed" };

  it("shows the current frame in the steps up to Limit", () => {
    expect(pickImage("align", "aligned", false, images)).toBe("frame");
    expect(pickImage("limit", "stacked", false, images)).toBe("frame");
  });

  it("shows the frame in Stack until the stack exists, then the stack", () => {
    expect(pickImage("stack", "limited", false, images)).toBe("frame");
    expect(pickImage("stack", "stacked", false, images)).toBe("stack");
    expect(pickImage("stack", "stacked", false, { ...images, stack: null })).toBe("frame");
  });

  it("shows the processed image in Wavelets and Save, and the stack with Before", () => {
    expect(pickImage("wavelets", "stacked", false, images)).toBe("processed");
    expect(pickImage("save", "stacked", false, images)).toBe("processed");
    expect(pickImage("wavelets", "stacked", true, images)).toBe("stack");
  });

  it("shows the stack in Wavelets while nothing is processed", () => {
    expect(pickImage("wavelets", "stacked", false, { ...images, processed: null })).toBe("stack");
  });
});

describe("imageCaption", () => {
  const data = { frame: 3, frameCount: 10, stackSize: 8, processed: true };

  it("names the frame, the stack or the sharpened stack", () => {
    expect(imageCaption("align", "stacked", false, data)).toBe("Frame 4 of 10");
    expect(imageCaption("stack", "stacked", false, data)).toBe("Stack of 8 frames");
    expect(imageCaption("wavelets", "stacked", false, data)).toBe("Sharpened stack");
    expect(imageCaption("wavelets", "stacked", true, data)).toBe("Stack of 8 frames");
    expect(imageCaption("save", "stacked", false, { ...data, processed: false })).toBe("Stack of 8 frames");
  });

  it("names the frame in Stack before there is a stack", () => {
    expect(imageCaption("stack", "limited", false, { ...data, stackSize: null })).toBe("Frame 4 of 10");
  });
});
