import { ISSUES_URL, LICENSE, LICENSE_URL, REPO, REPO_URL, SITE_NAME } from "./config.ts";

export function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

export interface PageOptions {
  title: string;
  description: string;
  // Path from the page to the site root, "" for the landing page and "../../" for docs/<slug>/.
  base: string;
  // Which header link is current.
  section: "home" | "docs";
  body: string;
}

// Runs before first paint: follows the system theme unless the visitor chose one with the toggle.
const themeScript = `(function(){var d=document.documentElement;d.classList.add("js");var t=null;try{t=localStorage.getItem("theme")}catch(e){}
if(t!=="light"&&t!=="dark"){t=window.matchMedia("(prefers-color-scheme: light)").matches?"light":"dark"}
if(t==="light"){d.setAttribute("data-theme","light")}})();`;

export function layout(page: PageOptions): string {
  const { base } = page;
  const current = (section: PageOptions["section"]) => (page.section === section ? ' aria-current="page"' : "");
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escapeHtml(page.title)}</title>
<meta name="description" content="${escapeHtml(page.description)}">
<meta property="og:title" content="${escapeHtml(page.title)}">
<meta property="og:description" content="${escapeHtml(page.description)}">
<meta name="color-scheme" content="dark light">
<link rel="icon" type="image/png" href="${base}assets/icon.png">
<link rel="stylesheet" href="${base}assets/tokens.css">
<link rel="stylesheet" href="${base}assets/style.css">
<script>${themeScript}</script>
<script src="${base}assets/site.js" defer></script>
</head>
<body data-repo="${escapeHtml(REPO)}">
<a class="skip" href="#main">Skip to content</a>
<header class="site-header">
  <div class="wrap bar">
    <a class="brand" href="${base || "./"}"><img src="${base}assets/icon.png" alt="" width="32" height="32"><span>${escapeHtml(SITE_NAME)}</span></a>
    <nav aria-label="Main">
      <a href="${base}#features">Features</a>
      <a href="${base}#download">Download</a>
      <a href="${base}docs/"${current("docs")}>Docs</a>
      <a href="${REPO_URL}">GitHub</a>
    </nav>
    <button class="theme-toggle" type="button" aria-label="Switch between light and dark theme" data-theme-toggle>
      <svg viewBox="0 0 16 16" width="18" height="18" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="8" cy="8" r="5.5"/><path d="M8 2.5v11" /><path d="M8 2.5a5.5 5.5 0 0 1 0 11z" fill="currentColor"/></svg>
    </button>
  </div>
</header>
<main id="main">
${page.body}
</main>
<footer class="site-footer">
  <div class="wrap">
    <p>${escapeHtml(SITE_NAME)} is free software, licensed under the <a href="${LICENSE_URL}">${LICENSE}</a>.</p>
    <p><a href="${REPO_URL}">Source code on GitHub</a> · <a href="${ISSUES_URL}">Report a problem</a> · <a href="${base}docs/">Documentation</a></p>
  </div>
</footer>
</body>
</html>
`;
}
