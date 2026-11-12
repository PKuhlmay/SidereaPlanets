<script setup lang="ts">
import { onMounted } from "vue";
import DocsView from "./docs/DocsView.vue";
import { closeHelp } from "./helpWindow";

onMounted(() => {
  document.title = "Siderea Planets Help";
});
</script>

<template>
  <DocsView class="help" @close="closeHelp" />
</template>

<style scoped>
.help {
  height: 100vh;
}
</style>
