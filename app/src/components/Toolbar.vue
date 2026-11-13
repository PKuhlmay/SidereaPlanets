<script setup lang="ts">
import { computed } from "vue";
import { cpu } from "../compute";
import { cancelJob, togglePause } from "../jobs";
import type { ProcessingArea } from "../logic/settings";
import { recentLabel } from "../logic/recent";
import { applyTheme } from "../theme";
import { chooseFiles, openRecording, recent } from "../recording";
import { session, settings, status, theme } from "../state";
import { openHelp } from "../helpWindow";
import { resourceSettings } from "../resource";
import { openProject, saveProject } from "../saving";
import MenuButton from "./MenuButton.vue";
import NumberField from "./NumberField.vue";

const loaded = computed(() => session.step !== "empty");

// Help moves the focus into another window. After a mouse click the button gives its focus up, otherwise the main
// window shows the focus ring on it when the help window closes; a keyboard press (detail 0) keeps the focus.
function onHelp(event: MouseEvent) {
  if (event.detail > 0) (event.currentTarget as HTMLElement).blur();
  void openHelp();
}
const busy = computed(() => status.busy !== null);

const processingAreas: ProcessingArea[] = [2048, 1024, 512, 256, 128];
const areaLabel = (size: number) => `${size} × ${size} px`;

// The entries of the Settings menu that no step reads yet: their values are only stored, so they are locked (MAC-014). The
// menus Flat/Dark/Reference and Tools, where nothing worked, are gone from the toolbar; their values stay in `settings.menu`.
const lockedSettings = [
  "Automatic Processing",
  "Autodetect Color/BW",
  "Luminance Setting",
  "Stretch FIT/TIFF",
  "Play Sounds",
  "Swap Red/Blue Channel",
  "Wait When Loading a Single Image",
  "Maximize During Messages",
  "Save to Subdir",
  "Bias Subtract",
];
</script>

<template>
  <header class="toolbar">
    <div class="zone">
      <button type="button" class="btn ghost select" :disabled="busy" title="Select a recording: a video or a series of images" @click="chooseFiles">
        Select Recording…
      </button>

      <MenuButton v-slot="{ close }" label="Recent Recordings" title="Recently opened recordings">
        <div v-if="recent.list.length === 0" class="empty">No recent recordings</div>
        <button
          v-for="paths in recent.list"
          :key="paths.join('|')"
          type="button"
          class="mi"
          :title="paths.join(', ')"
          @click="close(), openRecording(paths)"
        >
          {{ recentLabel(paths) }}
        </button>
      </MenuButton>

      <MenuButton label="Settings" wide>
        <div class="sw">
          <button
            type="button"
            class="mi"
            aria-haspopup="menu"
            title="Size of the square in the middle of the image that the Wavelets preview calculates. Smaller is faster."
          >
            Preview Area <span class="v arrow">{{ areaLabel(settings.menu.processingArea) }}</span>
          </button>
          <div class="menu fly">
            <label v-for="size in processingAreas" :key="size" class="mi">
              <input v-model="settings.menu.processingArea" type="radio" name="processing-area" :value="size" />{{ areaLabel(size) }}
            </label>
          </div>
        </div>
        <!-- Takes the place of "Set Skin" in the original: the light theme is part of the design (docs/design.md). -->
        <div class="sw">
          <button type="button" class="mi" aria-haspopup="menu">Theme <span class="v arrow">{{ theme.current === "dark" ? "Dark" : "Light" }}</span></button>
          <div class="menu fly">
            <label class="mi"><input type="radio" name="theme" :checked="theme.current === 'dark'" @change="applyTheme('dark')" />Dark</label>
            <label class="mi"><input type="radio" name="theme" :checked="theme.current === 'light'" @change="applyTheme('light')" />Light</label>
          </div>
        </div>
        <label class="mi" title="Opens the Advanced section of every step">
          <input v-model="settings.menu.advancedControls" type="checkbox" />Show Advanced Controls
        </label>
        <label class="mi" title="CPU load of this app as a share of all cores and the memory it holds, at the left end of the status bar">
          <input v-model="resourceSettings.shown" type="checkbox" />Show CPU and Memory Usage
        </label>
        <hr />
        <div class="cap">Not available yet</div>
        <div v-for="name in lockedSettings" :key="name" class="mi off" title="Not available yet">{{ name }}</div>
      </MenuButton>

      <!-- Not in the original: the project file keeps the recording and the settings of a session (docs/formats.md). -->
      <button
        type="button"
        class="btn ghost"
        title="Open a project file: the recording and the settings of an earlier session"
        :disabled="busy"
        @click="openProject"
      >
        <svg class="ico" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path d="M1.5 3.5h4l1.5 1.5h7.5v7.5h-13z" />
        </svg>
        Open Project…
      </button>
      <button
        type="button"
        class="btn ghost"
        title="Save the recording and the settings of this session as a project file"
        :disabled="!loaded || busy"
        @click="saveProject"
      >
        <svg class="ico" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path d="M2.5 2.5h8l3 3v8h-11zM5 2.5v3.5h5V2.5M5 13.5v-4h6v4" />
        </svg>
        Save Project…
      </button>
    </div>

    <div class="zone mid">
      <button type="button" class="btn" :class="{ next: busy }" :disabled="!busy" title="Stop the calculation" @click="cancelJob">Cancel</button>
      <button type="button" class="btn" :disabled="!busy" title="Pause the calculation; press again to resume" @click="togglePause">
        {{ status.busy?.paused ? "Resume" : "Pause" }}
      </button>
    </div>

    <div class="zone right">
      <button type="button" class="btn ghost" @click="onHelp">
        <svg class="ico" viewBox="0 0 16 16" width="16" height="16" aria-hidden="true">
          <path d="M8 3.5C6.5 2.5 4.5 2.3 1.5 2.5v10c3-.2 5 0 6.5 1zm0 0c1.5-1 3.5-1.2 6.5-1v10c-3-.2-5 0-6.5 1" />
        </svg>
        Help
      </button>
      <button type="button" class="btn ghost" disabled title="Not available yet">About</button>
      <label class="cpus" title="CPU cores to use for all calculations">
        CPUs <NumberField v-model="settings.cpus" :min="1" :max="cpu.max" :width="52" label="CPU cores to use" />
      </label>
    </div>
  </header>
</template>

<style scoped>
.toolbar {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 12px;
  align-items: center;
  padding: 8px 12px;
  background: var(--panel);
  border-bottom: 1px solid var(--line);
}
.zone {
  display: flex;
  gap: 4px;
  align-items: center;
}
.zone.mid {
  justify-content: center;
}
.zone.right {
  justify-content: flex-end;
  gap: 10px;
}
.select {
  margin-right: 8px;
  padding: 0 18px;
}
.cpus {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* menu entries (the list itself is styled in MenuButton) */
.mi {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  min-height: 28px;
  padding: 0 8px;
  text-align: left;
  background: none;
  border: 0;
  border-radius: var(--r-s);
  cursor: pointer;
}
.mi:hover:not(:disabled),
.sw:focus-within > .mi {
  background: var(--accent-soft);
}
.mi:disabled,
.mi.off,
.mi:has(input:disabled) {
  color: var(--off);
  cursor: default;
}
.mi:disabled:hover,
.mi.off:hover,
.mi:has(input:disabled):hover {
  background: none;
}
.mi .v {
  margin-left: auto;
  padding-left: 16px;
  color: var(--muted);
}
.mi .arrow::after {
  content: " ›";
}
hr {
  margin: 4px 0;
  border: 0;
  border-top: 1px solid var(--line);
}
.empty {
  padding: 8px;
  color: var(--muted);
}
.cap {
  padding: 4px 8px 2px;
  font-size: var(--fs-s);
  color: var(--muted);
}

/* submenus open to the right of their entry on hover or focus */
.sw {
  position: relative;
}
.fly {
  position: absolute;
  top: -5px;
  left: calc(100% - 2px);
  z-index: 31;
  display: none;
  min-width: 220px;
  padding: 4px;
  background: var(--raised);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-l);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
}
.sw:hover > .fly,
.sw:focus-within > .fly {
  display: block;
}
</style>
