<script setup lang="ts">
import { settings } from "../state";
import NumberField from "./NumberField.vue";
import StepPanel from "./StepPanel.vue";

// Step 3 Align: the two sizes; under Advanced the locked alignment modes and the rest of Alignment Setup.
const align = settings.align;
</script>

<template>
  <StepPanel id="align" :more="5">
    <label class="row" title="Edge length of the square around each alignpoint that is searched for in the other frames, in pixels.">
      Box size per point (px) <NumberField v-model="align.boxSize" />
    </label>
    <label class="row" title="How far an alignpoint may move from one frame to the next, in pixels.">
      Max. point shift (px) <NumberField v-model="align.maxMovement" />
    </label>

    <template #advanced>
      <div class="grp">
        <div class="gh">Alignment Setup</div>
        <div class="sub first">Alignment mode</div>
        <div class="seg" title="Not available yet">
          <button type="button" disabled>No Align</button>
          <button type="button" disabled>Scan Frames</button>
        </div>
        <span class="factor">Not available yet</span>
        <!-- Locked: the window does not draw the alignment yet; Align by center of gravity and Estimate rotation are locked
             as observed in the original, what unlocks them is not known (docs/ui.md) -->
        <div class="inl" title="Not available yet">
          <label class="chk"><input type="checkbox" disabled />Show alignment</label>
          <span class="factor">Not available yet</span>
        </div>
        <div class="inl" title="Not available yet">
          <label class="chk"><input type="checkbox" disabled />Align by center of gravity</label>
          <span class="factor">Not available yet</span>
        </div>
        <label class="row indent" title="Not available yet">Brightness threshold <NumberField :model-value="20" disabled label="Brightness threshold" /></label>
        <div class="inl" title="Not available yet">
          <label class="chk"><input type="checkbox" disabled />Estimate rotation</label>
          <span class="factor">Not available yet</span>
        </div>
      </div>
    </template>
  </StepPanel>
</template>

<style scoped>
.factor {
  display: block;
  margin-top: 2px;
  font-size: var(--fs-s);
  color: var(--muted);
}
.first {
  margin-top: 0;
}
.indent {
  padding-left: 20px;
}
.inl {
  display: flex;
  gap: 0 6px;
  align-items: center;
  min-height: 29px;
}
.inl .factor {
  margin: 0 0 0 auto;
}
</style>
