<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import ActionRow from "./components/ActionRow.vue";
import AlignPanel from "./components/AlignPanel.vue";
import FloatingCards from "./components/FloatingCards.vue";
import FunctionCards from "./components/FunctionCards.vue";
import FrameBar from "./components/FrameBar.vue";
import FunctionsPanel from "./components/FunctionsPanel.vue";
import ImageOverlays from "./components/ImageOverlays.vue";
import ImageViewer from "./components/ImageViewer.vue";
import StackPanel from "./components/StackPanel.vue";
import StatusBar from "./components/StatusBar.vue";
import TabBar from "./components/TabBar.vue";
import Toolbar from "./components/Toolbar.vue";
import WaveletPanel from "./components/WaveletPanel.vue";
import DocsView from "./docs/DocsView.vue";
import type { Rect } from "./logic/align";
import { startColour } from "./colour";
import { pickToneImage } from "./logic/colour";
import { atLeast, pickImage, tabLocked, windowTitle } from "./logic/workflow";
import { session, settings, status } from "./state";

// The work view stays mounted (v-show) while the documentation is open, so the recording and settings survive.
const showDocs = ref(false);
// "Before" in the zoom bar of the Wavelet tab: the unprocessed stack instead of the sharpened image.
const before = ref(false);

const image = computed(() =>
  pickImage(session.tab, session.step, before.value, {
    frame: session.frameImage,
    stack: session.stackImage,
    processed: session.processedImage,
  }),
);
const wavelet = computed(() => session.tab === "wavelet");
startColour();

// The Wavelet tab shows the colour and tone cards applied to the image, or the Copy To image or the difference to it;
// Before shows the plain stack.
const shown = computed(() => {
  if (!wavelet.value || before.value) return image.value;
  return pickToneImage(session.toneView, {
    current: session.adjustedImage ?? image.value,
    clipboard: session.clipboardImage,
    difference: session.differenceImage,
  });
});
const withFunctions = computed(() => wavelet.value && settings.wavelet.functionsOpen);
const emptyText = computed(() => (session.step === "empty" ? "Press Select to open a recording" : "No image yet"));

// After Limit the window moves on to the Stack tab by itself; a tab that is no longer open falls back to Align.
watch(
  () => session.step,
  (step, was) => {
    if (step === "limited" && !atLeast(was, "limited")) session.tab = "stack";
    else if (tabLocked(step, session.tab)) session.tab = "align";
  },
);
watch(
  () => session.tab,
  () => {
    before.value = false;
    status.position = "";
  },
);

// A region drawn with R.o.Interest ends the drawing and shows the rectangle.
function onSelect(rect: Rect) {
  session.roi = rect;
  session.roiDrawing = false;
  settings.align.showRoi = true;
}

// Position under the pointer, shown in the status bar of the Wavelet tab.
function onHover(pixel: { x: number; y: number } | null) {
  if (!wavelet.value || !pixel) {
    status.position = "";
    return;
  }
  const stack = session.stackSize === null ? "" : ` Stack=${session.stackSize}`;
  status.position = `X=${pixel.x} Y=${pixel.y}${stack}`;
}

// The title of the window shows the file and, during a calculation, step and percent (also in the taskbar).
watch(
  () => windowTitle(session.fileName, status.busy ? { name: status.busy.name, progress: status.progress } : null),
  (title) => {
    document.title = title;
    try {
      getCurrentWindow()
        .setTitle(title)
        .catch(() => undefined);
    } catch {
      // Not running inside Tauri (browser preview of a dev build): the document title is enough.
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="shell">
    <Toolbar :docs-open="showDocs" @toggle-docs="showDocs = !showDocs" />
    <DocsView v-if="showDocs" class="docs-slot" @close="showDocs = false" />
    <div v-show="!showDocs" class="work-view">
      <TabBar />
      <ActionRow />
      <div class="work" :class="{ 'with-functions': withFunctions }">
        <AlignPanel v-if="session.tab === 'align'" />
        <StackPanel v-else-if="session.tab === 'stack'" />
        <WaveletPanel v-else />
        <ImageViewer
          v-model:before="before"
          :image="shown"
          :comparable="wavelet"
          :empty="emptyText"
          :selecting="session.roiDrawing && session.tab === 'align'"
          @hover="onHover"
          @select="onSelect"
        >
          <template #overlay>
            <ImageOverlays v-if="shown" :width="shown.width" :height="shown.height" />
          </template>
          <template #floats>
            <FloatingCards />
            <FunctionCards />
          </template>
        </ImageViewer>
        <FunctionsPanel v-if="withFunctions" />
      </div>
      <FrameBar v-if="session.tab === 'align'" />
      <StatusBar />
    </div>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  min-width: 1120px;
  min-height: 600px;
}
.docs-slot {
  flex: 1;
  min-height: 0;
}
.work-view {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
}
.work {
  display: grid;
  flex: 1;
  grid-template-columns: var(--side) minmax(0, 1fr);
  min-height: 0;
}
.work.with-functions {
  grid-template-columns: var(--side) minmax(0, 1fr) var(--side-right);
}
</style>
