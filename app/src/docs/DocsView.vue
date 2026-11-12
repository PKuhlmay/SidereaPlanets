<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { groupPages, parsePage, renderBody, searchPages } from "./docs";

const emit = defineEmits<{ close: [] }>();

// Every Markdown file in ./pages is one page; the file name is its slug.
const files = import.meta.glob("./pages/*.md", { query: "?raw", import: "default", eager: true }) as Record<string, string>;
const pages = Object.entries(files).map(([path, raw]) => parsePage(path.replace(/^.*\/(.+)\.md$/, "$1"), raw));
const groups = groupPages(pages);
const ordered = groups.flatMap((g) => g.pages);

const query = ref("");
const results = computed(() => searchPages(ordered, query.value));
const searching = computed(() => query.value.trim() !== "");

const slug = ref(ordered[0]?.slug ?? "");
const index = computed(() => ordered.findIndex((p) => p.slug === slug.value));
const page = computed(() => ordered[index.value]);
const previous = computed(() => ordered[index.value - 1]);
const next = computed(() => ordered[index.value + 1]);
const html = computed(() => (page.value ? renderBody(page.value.body) : ""));
const breadcrumb = computed(() => page.value?.group ?? "");

const content = ref<HTMLElement | null>(null);

function open(target: string) {
  slug.value = target;
  query.value = "";
}

// New page: back to the top of the text.
watch(slug, async () => {
  await nextTick();
  content.value?.scrollTo({ top: 0 });
});

function onContentClick(event: MouseEvent) {
  const link = (event.target as HTMLElement).closest<HTMLAnchorElement>("a[data-page]");
  if (!link) return;
  event.preventDefault();
  open(link.dataset.page ?? "");
}

function onKey(event: KeyboardEvent) {
  if (event.key === "Escape") emit("close");
}

onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <div class="docs">
    <nav class="list" aria-label="Documentation pages">
      <input v-model="query" class="search" type="search" placeholder="Search the documentation" aria-label="Search the documentation" />
      <template v-if="searching">
        <p v-if="results.length === 0" class="none">No page matches.</p>
        <button
          v-for="p in results"
          :key="p.slug"
          type="button"
          class="entry"
          :aria-current="p.slug === slug ? 'page' : undefined"
          @click="open(p.slug)"
        >
          {{ p.title }}
        </button>
      </template>
      <template v-else>
        <div v-for="g in groups" :key="g.name" class="group">
          <h2>{{ g.name }}</h2>
          <button
            v-for="p in g.pages"
            :key="p.slug"
            type="button"
            class="entry"
            :aria-current="p.slug === slug ? 'page' : undefined"
            @click="open(p.slug)"
          >
            {{ p.title }}
          </button>
        </div>
      </template>
    </nav>

    <div ref="content" class="content">
      <article v-if="page" class="page">
        <p class="crumb">{{ breadcrumb }}</p>
        <div class="text" @click="onContentClick" v-html="html" />
        <footer class="pager">
          <button v-if="previous" type="button" @click="open(previous.slug)">‹ {{ previous.title }}</button>
          <span v-else />
          <button v-if="next" type="button" @click="open(next.slug)">{{ next.title }} ›</button>
        </footer>
      </article>
    </div>
  </div>
</template>

<style scoped>
.docs {
  display: grid;
  grid-template-columns: var(--side) minmax(0, 1fr);
  min-height: 0;
  height: 100%;
  background: var(--bg);
  color: var(--text);
  font-family: var(--font);
  font-size: var(--fs);
}

/* page list */
.list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 12px;
  overflow-y: auto;
  background: var(--panel);
  border-right: 1px solid var(--line);
}
.search {
  height: 28px;
  padding: 0 8px;
  margin-bottom: 12px;
  font: inherit;
  color: var(--text);
  background: var(--field);
  border: 1px solid var(--line-strong);
  border-radius: var(--r-s);
}
.search:focus {
  outline: none;
  border-color: var(--accent);
}
.search::placeholder {
  color: var(--muted);
}
.group {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: 10px;
}
.group h2 {
  margin: 0;
  padding: 4px 8px;
  font-size: var(--fs-s);
  font-weight: 400;
  color: var(--muted);
}
.entry {
  height: 30px;
  padding: 0 8px;
  font: inherit;
  text-align: left;
  color: var(--text);
  background: none;
  border: 0;
  border-radius: var(--r);
  cursor: pointer;
}
.entry:hover {
  background: var(--raised);
}
.entry[aria-current="page"] {
  font-weight: 600;
  color: var(--accent-ink);
  background: var(--accent-soft);
}
.none {
  margin: 4px 8px;
  color: var(--muted);
}

/* text */
.content {
  overflow-y: auto;
}
/* 720 px of text in a wide window; the column and its side margin shrink with a narrow one */
.page {
  box-sizing: border-box;
  width: 100%;
  max-width: 816px;
  margin: 0 auto;
  padding: 36px clamp(16px, 5vw, 48px) 48px;
}
.crumb {
  margin: 0 0 14px;
  font-size: var(--fs-s);
  color: var(--muted);
}
.text :deep(h1) {
  margin: 0 0 14px;
  font-size: 28px;
  font-weight: 600;
  line-height: 36px;
}
.text :deep(h1 + p) {
  font-size: 15px;
  line-height: 22px;
  color: var(--muted);
}
.text :deep(h2) {
  margin: 28px 0 14px;
  font-size: 18px;
  font-weight: 600;
  line-height: 26px;
}
.text :deep(p),
.text :deep(li) {
  font-size: 14px;
  line-height: 20px;
}
.text :deep(p) {
  margin: 0 0 14px;
}
.text :deep(ol),
.text :deep(ul) {
  margin: 0 0 14px;
  padding-left: 22px;
}
.text :deep(li) {
  margin-bottom: 8px;
}
.text :deep(a) {
  color: var(--accent-ink);
}
.text :deep(table) {
  width: 100%;
  margin: 0 0 14px;
  border-collapse: separate;
  border-spacing: 0;
  overflow: hidden;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: var(--r-l);
}
.text :deep(th) {
  height: 32px;
  padding: 0 12px;
  font-size: var(--fs-s);
  font-weight: 600;
  text-align: left;
  color: var(--muted);
  background: var(--raised);
}
.text :deep(td) {
  min-height: 38px;
  padding: 9px 12px;
  line-height: 20px;
  border-top: 1px solid var(--line);
}
.text :deep(td:first-child) {
  font-weight: 600;
}
.text :deep(tr.off td) {
  color: var(--off);
}
.text :deep(blockquote) {
  margin: 0 0 14px;
  padding: 12px 14px;
  background: var(--accent-soft);
  border-left: 3px solid var(--accent);
  border-radius: var(--r);
}
.text :deep(blockquote p) {
  margin: 0;
}
.text :deep(code) {
  font-size: 12px;
}

.pager {
  display: flex;
  justify-content: space-between;
  padding-top: 16px;
}
.pager button {
  padding: 0;
  font: inherit;
  color: var(--accent-ink);
  background: none;
  border: 0;
  cursor: pointer;
}
.pager button:hover {
  text-decoration: underline;
}
</style>
