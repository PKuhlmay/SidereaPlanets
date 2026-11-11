// Saving through the Tauri commands of the backend (export_commands.rs): Save image and the project file. The
// file dialogs are opened by the backend.

import { invoke } from "@tauri-apps/api/core";
import { reactive } from "vue";
import { directoryOf, stem } from "./logic/exporting";
import { applyProject, buildProject, type ProjectFile } from "./logic/projectfile";
import { fileName } from "./logic/recent";
import { openRecording } from "./recording";
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

/** Save image: the stacked image in the Stack tab, the processed one in the Wavelet tab. */
export async function saveImage(): Promise<void> {
  const source: ImageSource = session.tab === "wavelet" ? "wavelet" : "stack";
  try {
    const path = await invoke<string | null>("save_image_as", {
      source,
      format: saveChoice.format,
      suggestedName: stem(session.fileName),
      directory: folder(),
    });
    status.result = path ? `Saved ${fileName(path)}` : "Save cancelled";
  } catch (e) {
    status.result = message("Save image", e);
  }
}

/** Tools > Save project. */
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

/** Tools > Open project: opens the recording of the project and puts its settings back. */
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
