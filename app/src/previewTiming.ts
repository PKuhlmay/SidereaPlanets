// Where the time of a preview goes, for the measurement in previewBench.ts. Off in a normal run: the three hooks below then
// cost one boolean check.

/** One finished preview, in milliseconds. */
export interface PreviewSample {
  /** `invoke` of `wavelet_preview`: the request until the bytes are in the page (the Rust side takes under a millisecond on a Mac). */
  invoke: number;
  /** Copying the area into the image and making the ImageData. */
  write: number;
  /** `putImageData` in the viewer, as far as the call itself blocks. */
  draw: number;
}

export const previewTiming = {
  on: false,
  samples: [] as PreviewSample[],
};

export function recordPreview(invoke: number, write: number): void {
  if (!previewTiming.on) return;
  previewTiming.samples.push({ invoke, write, draw: 0 });
}

/** The draw of a write is added to the sample of that write. */
export function recordDraw(ms: number): void {
  if (!previewTiming.on) return;
  const last = previewTiming.samples[previewTiming.samples.length - 1];
  if (last) last.draw += ms;
}
