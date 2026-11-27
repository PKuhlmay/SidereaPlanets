// Builds the project site into site/dist: the landing page, the user docs and the assets.
//
// The docs are rendered from the same Markdown files the app shows (app/src/docs/pages/), with the same parser and
// the same markdown library (`marked`, resolved from app/node_modules). Run after `cd app && pnpm install`:
//   node site/build.ts
// Needs Node 22.18 or newer, which runs TypeScript files directly.
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { groupPages, parsePage, renderBody, type DocPage } from "../app/src/docs/docs.ts";
import { SITE_NAME } from "./config.ts";
import { escapeHtml, layout } from "./layout.ts";
import { landingBody } from "./landing.ts";

const siteDir = dirname(fileURLToPath(import.meta.url));
const repoDir = join(siteDir, "..");
const outDir = join(siteDir, "dist");
const pagesDir = join(repoDir, "app", "src", "docs", "pages");

function write(path: string, content: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

// Real pictures (jpg, png, webp) win over the SVG placeholder of the same name.
function imageSrc(name: string): string {
  for (const ext of ["jpg", "png", "webp"]) {
    if (existsSync(join(siteDir, "assets", `${name}.${ext}`))) return `assets/${name}.${ext}`;
  }
  return `assets/${name}.svg`;
}

function loadDocs(): DocPage[] {
  return readdirSync(pagesDir)
    .filter((file) => file.endsWith(".md"))
    .map((file) => parsePage(file.replace(/\.md$/, ""), readFileSync(join(pagesDir, file), "utf8")));
}

function docsNav(pages: DocPage[], base: string, activeSlug: string | null): string {
  const groups = groupPages(pages)
    .map((group) => {
      const items = group.pages
        .map((p) => {
          const current = p.slug === activeSlug ? ' aria-current="page"' : "";
          return `<li><a href="${base}docs/${p.slug}/"${current}>${escapeHtml(p.title)}</a></li>`;
        })
        .join("");
      return `<h2>${escapeHtml(group.name)}</h2><ul>${items}</ul>`;
    })
    .join("");
  return `<nav class="docs-nav" aria-label="Documentation">${groups}</nav>`;
}

function renderDocPage(page: DocPage, pages: DocPage[]): string {
  const base = "../../";
  // The app links pages as "#slug"; on the site they are real pages.
  const html = renderBody(page.body).replace(/<a href="#([a-z0-9-]+)" data-page="[a-z0-9-]+">/g, `<a href="${base}docs/$1/">`);
  const ordered = groupPages(pages).flatMap((g) => g.pages);
  const index = ordered.findIndex((p) => p.slug === page.slug);
  const prev = ordered[index - 1];
  const next = ordered[index + 1];
  const pager = `<nav class="pager" aria-label="Previous and next page">${
    prev ? `<a class="prev" href="${base}docs/${prev.slug}/"><span>Previous</span>${escapeHtml(prev.title)}</a>` : "<span></span>"
  }${next ? `<a class="next" href="${base}docs/${next.slug}/"><span>Next</span>${escapeHtml(next.title)}</a>` : ""}</nav>`;
  const body = `<div class="wrap docs">
${docsNav(pages, base, page.slug)}
<article class="doc">
${html}
${pager}
</article>
</div>`;
  return layout({ title: `${page.title} – ${SITE_NAME} documentation`, description: `${page.title}: ${SITE_NAME} user documentation.`, base, section: "docs", body });
}

function renderDocsIndex(pages: DocPage[]): string {
  const base = "../";
  const groups = groupPages(pages)
    .map(
      (group) => `<section><h2>${escapeHtml(group.name)}</h2><ul>${group.pages
        .map((p) => `<li><a href="${p.slug}/">${escapeHtml(p.title)}</a></li>`)
        .join("")}</ul></section>`,
    )
    .join("\n");
  const body = `<div class="wrap docs-index">
<h1>Documentation</h1>
<p class="section-lead">How to use ${SITE_NAME}, from opening a recording to saving the sharpened image. The same pages are available in the app under Help.</p>
<div class="cards">
${groups}
</div>
</div>`;
  return layout({ title: `Documentation – ${SITE_NAME}`, description: `User documentation of ${SITE_NAME}.`, base, section: "docs", body });
}

rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });

// Assets: everything in site/assets, plus the design tokens and the app icon straight from their single sources.
cpSync(join(siteDir, "assets"), join(outDir, "assets"), { recursive: true });
// The comments of tokens.css are internal design notes, so they are not published.
write(
  join(outDir, "assets", "tokens.css"),
  readFileSync(join(repoDir, "docs", "design", "tokens.css"), "utf8").replace(/\/\*[\s\S]*?\*\//g, "").replace(/[ \t]+$/gm, "").replace(/\n{2,}/g, "\n"),
);
cpSync(join(repoDir, "app", "src-tauri", "icons", "128x128@2x.png"), join(outDir, "assets", "icon.png"));

write(
  join(outDir, "index.html"),
  layout({
    title: `${SITE_NAME} – stack and sharpen planets and the Moon`,
    description: "Free, open-source planetary and lunar astrophotography: stack the best frames of a video and sharpen the result with wavelets. For macOS, Windows and Linux.",
    base: "",
    section: "home",
    body: landingBody(imageSrc),
  }),
);

const pages = loadDocs();
write(join(outDir, "docs", "index.html"), renderDocsIndex(pages));
for (const page of pages) write(join(outDir, "docs", page.slug, "index.html"), renderDocPage(page, pages));

console.log(`Built ${pages.length} doc pages and the landing page into ${outDir}`);
