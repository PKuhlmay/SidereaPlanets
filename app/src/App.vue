<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface RecordingInfo {
  width: number;
  height: number;
  bitDepth: number;
  frameCount: number;
  color: string;
  observer: string;
  instrument: string;
  telescope: string;
}

// Placeholder shell: proves the Rust bridge works. The real UI is designed separately (see RK-010).
const path = ref("");
const info = ref<RecordingInfo | null>(null);
const error = ref("");

async function open() {
  error.value = "";
  info.value = null;
  try {
    info.value = await invoke<RecordingInfo>("open_recording", { path: path.value });
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <main>
    <h1>SidereaPlanets</h1>
    <form @submit.prevent="open">
      <input v-model="path" placeholder="Path to a .ser file" />
      <button type="submit">Open</button>
    </form>
    <p v-if="error">{{ error }}</p>
    <pre v-if="info">{{ info }}</pre>
  </main>
</template>
