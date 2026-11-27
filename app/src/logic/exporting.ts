// Names for the save dialogs: the dialogs propose the name of the recording and open in its folder.

/** "JUPITER.AVI" -> "JUPITER"; a name without extension stays as it is. */
export function stem(fileName: string): string {
  const dot = fileName.lastIndexOf(".");
  return dot > 0 ? fileName.slice(0, dot) : fileName;
}

/** The folder of a file path (either kind of separator), or undefined for a bare name. */
export function directoryOf(path: string | undefined): string | undefined {
  if (!path) return undefined;
  const cut = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  return cut > 0 ? path.slice(0, cut) : undefined;
}
