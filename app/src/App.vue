<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

interface RecordingInfo {
  format: string;
  width: number;
  height: number;
  bitDepth: number;
  frameCount: number;
  color: string;
  fps: number | null;
  observer: string | null;
  instrument: string | null;
  telescope: string | null;
}

// Same filters and order as the "Open file(s)" dialog of the reference program (docs/formats.md).
const fileFilters = [
  { name: "Video (*.avi *.ser)", extensions: ["avi", "ser"] },
  { name: "Still frames (*.bmp *.jpg *.tif *.png)", extensions: ["bmp", "jpg", "jpeg", "tif", "tiff", "png"] },
  { name: "FITS frames (*.fit *.fits *.fts)", extensions: ["fit", "fits", "fts"] },
  { name: "All files", extensions: ["*"] },
];

// Placeholder shell: proves the Rust bridge and the frame transfer work. The real UI is designed separately (see RK-010).
const paths = ref<string[]>([]);
const info = ref<RecordingInfo | null>(null);
const error = ref("");
const frame = ref(0);
const frameMs = ref<number | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);

// Drops answers that arrive after a newer frame was requested.
let latestRequest = 0;

async function loadFrame() {
  const request = ++latestRequest;
  const started = performance.now();
  try {
    // Binary layout: width (u32 LE), height (u32 LE), then RGBA bytes.
    const buffer = await invoke<ArrayBuffer>("read_frame", { index: frame.value });
    if (request !== latestRequest || !canvas.value) return;
    const view = new DataView(buffer);
    const width = view.getUint32(0, true);
    const height = view.getUint32(4, true);
    const pixels = new Uint8ClampedArray(buffer, 8, width * height * 4);
    canvas.value.width = width;
    canvas.value.height = height;
    canvas.value.getContext("2d")?.putImageData(new ImageData(pixels, width, height), 0, 0);
    frameMs.value = Math.round(performance.now() - started);
  } catch (e) {
    error.value = String(e);
  }
}

async function chooseFiles() {
  // Several still frames together form one sequence, so multiple selection is allowed.
  const picked = await openDialog({ multiple: true, directory: false, filters: fileFilters });
  if (!picked || picked.length === 0) return;
  paths.value = picked;
  await open();
}

async function open() {
  error.value = "";
  info.value = null;
  frameMs.value = null;
  try {
    info.value = await invoke<RecordingInfo>("open_recording", { paths: paths.value });
    frame.value = 0;
    await loadFrame();
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <main>
    <h1>SidereaPlanets</h1>
    <button type="button" @click="chooseFiles">Select</button>
    <p v-if="paths.length > 1">{{ paths.length }} files selected</p>
    <p v-else-if="paths.length === 1">{{ paths[0] }}</p>
    <p v-if="error">{{ error }}</p>
    <template v-if="info">
      <p>
        {{ info.format }} · {{ info.width }}×{{ info.height }} · {{ info.bitDepth }} bit · {{ info.color }} ·
        {{ info.frameCount }} frames<template v-if="info.fps"> · {{ info.fps.toFixed(2) }} fps</template>
      </p>
      <label>
        Frame {{ frame + 1 }} / {{ info.frameCount }}
        <input v-model.number="frame" type="range" min="0" :max="info.frameCount - 1" @input="loadFrame" />
      </label>
      <p v-if="frameMs !== null">Loaded in {{ frameMs }} ms</p>
    </template>
    <canvas ref="canvas" style="max-width: 100%" />
  </main>
</template>
