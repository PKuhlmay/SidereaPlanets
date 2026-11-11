// Saving through the Tauri commands of the backend (export_commands.rs, sequence_commands.rs): Save image, Save Aligned,
// Save Integrated and the project file. The file dialogs are opened by the backend.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { reactive } from "vue";
import { runJob } from "./jobs";
import { colourPayload, isNeutral } from "./logic/colour";
import { directoryOf, stem } from "./logic/exporting";
import { applyProject, buildProject, type ProjectFile } from "./logic/projectfile";
import { fileName } from "./logic/recent";
import { buildSequenceRequest } from "./logic/stack";
import { openRecording } from "./recording";
import { doAllIsUpToDate, runDoAll } from "./wavelet";
import { session, settings, status } from "./state";

export type ImageSource = "stack" | "wavelet";

export interface ExportFormatInfo {
  /** Name the backend knows the format by. */
  id: string;
  label: string;
  extension: string;
}

/** The format Save image uses; the menu next to the button sets it before the dialog opens. */
export const saveChoice = reactive({ format: "tiff16" });

/** The file types of Save image in a tab (the Stack tab offers more than the Wavelet tab). */
export const exportFormats = (source: ImageSource) => invoke<ExportFormatInfo[]>("export_formats", { source });

const folder = () => directoryOf(session.paths[0]);

const message = (action: string, error: unknown) => `${action}: ${String(error)}`;

/**
 * Save image: the stacked image in the Stack tab, the processed one in the Wavelet tab. In the Wavelet tab the file is
 * written the way the window shows it, with the values of the colour and tone cards applied; the Stack tab shows the
 * stack as it is. If the layer values, Colour or LRGB changed since the last Do All, or none ran yet, Do All runs
 * first with the current values (as a job, so it shows progress and can be cancelled); a cancelled or failed Do All
 * saves nothing.
 */
export async function saveImage(): Promise<void> {
  if (status.busy) return;
  const source: ImageSource = session.tab === "wavelet" ? "wavelet" : "stack";
  if (source === "wavelet" && !doAllIsUpToDate() && !(await runDoAll())) return;
  const values = colourPayload(settings.wavelet);
  try {
    const path = await invoke<string | null>("save_image_as", {
      source,
      format: saveChoice.format,
      suggestedName: stem(session.fileName),
      directory: folder(),
      adjustments: source === "wavelet" && !isNeutral(values) ? values : null,
    });
    status.result = path ? `Saved ${fileName(path)}` : "Save cancelled";
  } catch (e) {
    status.result = message("Save image", e);
  }
}

interface SavedVideo {
  path: string;
  frames: number;
}

interface SequenceProgress {
  done: number;
  total: number;
}

type VideoKind = { job: "Save Aligned" | "Save Integrated"; command: "save_aligned" | "save_integrated"; suffix: string };

/** The file dialog comes first, then the video is written as a job with progress, Cancel and Pause. */
async function saveVideo({ job: name, command, suffix }: VideoKind): Promise<void> {
  if (status.busy) return;
  try {
    const request = buildSequenceRequest(session, settings);
    const path = await invoke<string | null>("choose_video_path", {
      suggestedName: `${stem(session.fileName)}${suffix}`,
      directory: folder(),
    });
    if (!path) {
      status.result = "Save cancelled";
      return;
    }
    await runJob(name, async (job) => {
      const total = request.stack.frames.length;
      job.progress(0, total, "Writing frame: 0");
      const unlisten = await listen<SequenceProgress>("sequence-progress", ({ payload }) =>
        job.progress(payload.done, payload.total, `Writing frame: ${payload.done}`),
      );
      let saved: SavedVideo;
      try {
        saved = await invoke<SavedVideo>(command, { path, settings: request });
      } finally {
        unlisten();
      }
      return `Saved ${fileName(saved.path)} (${saved.frames} frames)`;
    });
  } catch (e) {
    status.result = message(name, e);
  }
}

/** Save Aligned: the frames that stay after Limit, moved onto the reference frame, as an AVI. */
export const saveAligned = () => saveVideo({ job: "Save Aligned", command: "save_aligned", suffix: "_aligned" });

/** Save Integrated: the moving average over Average of n Frames aligned frames, as an AVI. */
export const saveIntegrated = () =>
  saveVideo({ job: "Save Integrated", command: "save_integrated", suffix: "_integrated" });

/** Save project in the toolbar. */
export async function saveProject(): Promise<void> {
  if (session.paths.length === 0) {
    status.result = "Save project: no recording is open";
    return;
  }
  try {
    const path = await invoke<string | null>("project_save_as", {
      project: buildProject(session.paths, settings),
      suggestedName: stem(session.fileName),
      directory: folder(),
    });
    status.result = path ? `Saved ${fileName(path)}` : "Save cancelled";
  } catch (e) {
    status.result = message("Save project", e);
  }
}

interface LoadedProject {
  path: string;
  project: ProjectFile;
  missingFiles: string[];
}

/** Open project in the toolbar: opens the recording of the project and puts its settings back. */
export async function openProject(): Promise<void> {
  try {
    const loaded = await invoke<LoadedProject | null>("project_open", { directory: folder() });
    if (!loaded) return;
    const { project, missingFiles } = loaded;
    if (missingFiles.length > 0) {
      status.result = `Open project: file not found: ${missingFiles.map(fileName).join(", ")}`;
      return;
    }
    if (project.recording.length === 0) {
      status.result = "Open project: the project has no recording";
      return;
    }
    await openRecording(project.recording);
    // openRecording reports its own error; the settings only apply if the recording is the one that is open now.
    if (session.paths.join("|") !== project.recording.join("|")) return;
    applyProject(settings, project);
    status.result = `Opened ${fileName(loaded.path)}`;
  } catch (e) {
    status.result = message("Open project", e);
  }
}
