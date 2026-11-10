<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

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

// Placeholder shell: proves the Rust bridge and the frame transfer work. The real UI is designed separately (see RK-010).
const path = ref("");
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

async function open() {
  error.value = "";
  info.value = null;
  frameMs.value = null;
  try {
    info.value = await invoke<RecordingInfo>("open_recording", { path: path.value });
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
    <form @submit.prevent="open">
      <input v-model="path" placeholder="Path to a .ser or .avi file" />
      <button type="submit">Open</button>
    </form>
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
