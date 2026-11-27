<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { LICENSE_ID, NOTICE_URL, SOURCE_URL, themedNotice } from "../logic/about";
import { appBuildLabel } from "../logic/buildInfo";
import { about, theme } from "../state";

// About: version, license and the way to the source code (PLANETS-7), and the licenses of the components the app contains. The
// notice is app/public/third-party-licenses.html, written by `pnpm gen:licenses`; it is shown in a frame without scripts.
const build = appBuildLabel();
const view = ref<"about" | "licenses">("about");
// null: loading; "": there is no generated notice in this build.
const notice = ref<string | null>(null);
const dialog = ref<HTMLElement | null>(null);

function close() {
  about.open = false;
}

async function showLicenses() {
  view.value = "licenses";
  notice.value = null;
  try {
    const response = await fetch(NOTICE_URL);
    notice.value = (response.ok ? themedNotice(await response.text(), theme.current) : null) ?? "";
  } catch {
    notice.value = "";
  }
}

async function openSource() {
  try {
    await openUrl(SOURCE_URL);
  } catch {
    // Not running inside Tauri (browser preview of a dev build): a new tab does the same job.
    window.open(SOURCE_URL, "_blank", "noopener");
  }
}

function onKey(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  event.stopPropagation();
  if (view.value === "licenses") view.value = "about";
  else close();
}

// Opening starts at the About page and puts the focus into the dialog; the keys are read only while it is open.
watch(
  () => about.open,
  async (open) => {
    if (open) {
      view.value = "about";
      window.addEventListener("keydown", onKey, true);
      await nextTick();
      dialog.value?.focus();
    } else {
      window.removeEventListener("keydown", onKey, true);
    }
  },
);
onBeforeUnmount(() => window.removeEventListener("keydown", onKey, true));
</script>

<template>
  <div v-if="about.open" class="scrim" @pointerdown.self="close">
    <section ref="dialog" class="about" :class="{ wide: view === 'licenses' }" role="dialog" aria-modal="true" aria-labelledby="about-title" tabindex="-1">
      <template v-if="view === 'about'">
        <h2 id="about-title">Siderea Planets</h2>
        <p class="build">{{ build }}</p>
        <p>Aligns, stacks and sharpens planetary and lunar recordings.</p>
        <dl>
          <dt>License</dt>
          <dd>{{ LICENSE_ID }}</dd>
          <dt>Source code</dt>
          <dd>
            <a :href="SOURCE_URL" title="Open the source code in your browser" @click.prevent="openSource">{{ SOURCE_URL }}</a>
          </dd>
        </dl>
        <p class="free">
          Siderea Planets is free software: you can redistribute it and change it under the terms of the GNU General Public License,
          version 3 or any later version. It comes without any warranty.
        </p>
        <div class="acts">
          <button type="button" class="btn" title="The licenses of the components Siderea Planets contains" @click="showLicenses">
            Third-party licenses
          </button>
          <button type="button" class="btn next" @click="close">Close</button>
        </div>
      </template>
      <template v-else>
        <h2 id="about-title" class="sr">Third-party licenses</h2>
        <p v-if="notice === null" class="state">Loading…</p>
        <p v-else-if="notice === ''" class="state">The list of third-party licenses is not part of this build. Run “pnpm gen:licenses” in the folder app and build again.</p>
        <iframe v-else class="notice" title="Third-party licenses" sandbox="" :srcdoc="notice" />
        <div class="acts">
          <button type="button" class="btn" @click="view = 'about'">Back</button>
          <button type="button" class="btn next" @click="close">Close</button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.scrim {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.5);
}
.about {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: min(460px, calc(100vw - 48px));
  max-height: calc(100vh - 48px);
  padding: 20px 22px 16px;
  background: var(--panel);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-l);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.45);
  outline: none;
}
.about.wide {
  width: min(820px, calc(100vw - 48px));
  height: calc(100vh - 48px);
}
.sr {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip-path: inset(50%);
}
h2 {
  margin: 0;
  font-size: 18px;
}
p {
  margin: 0;
}
.build,
.free,
.state {
  color: var(--muted);
}
dl {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 4px 14px;
  margin: 4px 0;
}
dt {
  color: var(--muted);
}
dd {
  margin: 0;
  overflow-wrap: anywhere;
}
a {
  color: var(--accent-ink);
}
.notice {
  flex: 1;
  min-height: 0;
  width: 100%;
  border: 1px solid var(--line);
  border-radius: var(--r);
  background: var(--panel);
}
.acts {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 6px;
}
</style>
