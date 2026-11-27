import { AFTER, BEFORE, LATEST_RELEASE_URL, RELEASES_URL, REPO_URL, SCREENSHOTS, type SiteImage } from "./config.ts";
import { escapeHtml } from "./layout.ts";

// Maps an image name to its published path (relative to the site root).
export type ImageSrc = (name: string) => string;

function img(image: SiteImage, src: ImageSrc, extra = ""): string {
  return `<img src="${src(image.name)}" alt="${escapeHtml(image.alt)}" width="${image.width}" height="${image.height}" loading="lazy" decoding="async"${extra}>`;
}

const steps: [string, string][] = [
  ["Open", "Choose a SER or AVI video or a series of images."],
  ["Alignpoints", "Mark the surface details that every frame is lined up on."],
  ["Align", "Line up every frame and rate how sharp it is."],
  ["Best Frames", "Keep the sharpest frames and drop the rest."],
  ["Stack", "Add up the kept frames into one cleaner image."],
  ["Wavelets", "Bring out fine detail layer by layer."],
  ["Save", "Write the finished image to a file."],
];

const features: [string, string][] = [
  ["Made for planets and the Moon", "Alignpoints follow surface detail across the disc, so the whole image stays sharp, not just the centre."],
  ["Rates every frame", "Each frame gets a quality value. Keep the best percentage, a fixed number, or everything above a quality limit."],
  ["Wavelet sharpening", "Layer by layer, with a preview before you apply it. Save your settings as a scheme and reuse them on the next night."],
  ["One step after the other", "A step bar leads from the recording to the saved image. The next button is always the one to press."],
  ["Fast on many frames", "Videos with thousands of frames are processed on all CPU cores, with Cancel and Pause at any time."],
  ["Free and open source", "Licensed under the GPL-3.0-or-later. Read the code, build it yourself, report problems on GitHub."],
];

export function landingBody(src: ImageSrc): string {
  return `
<section class="hero">
  <div class="wrap">
    <h1>Sharper planets and Moon from your video</h1>
    <p class="lead">Siderea Planets stacks the best frames of a planetary or lunar recording and sharpens the result with wavelets. Free and open source, for macOS, Windows and Linux.</p>
    <p class="cta">
      <a class="btn primary" href="#download" data-hero-download>Download</a>
      <a class="btn" href="docs/">Read the documentation</a>
    </p>
    <p class="hint" data-hero-hint>Pick your system below.</p>
  </div>
</section>

<section class="compare-section" aria-labelledby="compare-title">
  <div class="wrap">
    <h2 id="compare-title" class="visually-hidden">Before and after</h2>
    <div class="compare" data-compare>
      ${img(BEFORE, src, ' class="compare-before"')}
      ${img(AFTER, src, ' class="compare-after"')}
      <span class="tag tag-before" aria-hidden="true">Before</span>
      <span class="tag tag-after" aria-hidden="true">After</span>
      <span class="handle" aria-hidden="true"></span>
      <input class="compare-range" type="range" min="0" max="100" value="50" aria-label="Compare the single frame (left) with the stacked and sharpened result (right)">
    </div>
    <p class="caption">Drag the handle to compare one video frame with the final image.</p>
  </div>
</section>

<section id="how" aria-labelledby="how-title">
  <div class="wrap">
    <h2 id="how-title">From recording to finished image in seven steps</h2>
    <p class="section-lead">The window guides you from left to right. After each step the next one opens by itself.</p>
    <ol class="steps">
      ${steps.map(([name, text]) => `<li><strong>${name}</strong><span>${text}</span></li>`).join("\n      ")}
    </ol>
  </div>
</section>

<section id="features" aria-labelledby="features-title">
  <div class="wrap">
    <h2 id="features-title">What it does</h2>
    <div class="cards">
      ${features.map(([title, text]) => `<article class="card"><h3>${title}</h3><p>${text}</p></article>`).join("\n      ")}
    </div>
    <p class="formats"><strong>Reads</strong> SER and AVI videos (uncompressed, Lagarith, MJPEG, HuffYUV). More formats are on the way, see <a href="docs/supported-formats/">Supported formats</a>.</p>
  </div>
</section>

<section id="screenshots" aria-labelledby="shots-title">
  <div class="wrap">
    <h2 id="shots-title">Screenshots</h2>
    <div class="shots">
      ${SCREENSHOTS.map((s) => `<figure>${img(s, src)}<figcaption>${s.caption}</figcaption></figure>`).join("\n      ")}
    </div>
  </div>
</section>

<section id="download" aria-labelledby="download-title">
  <div class="wrap">
    <h2 id="download-title">Download</h2>
    <p class="release" data-release>The newest version is on <a href="${LATEST_RELEASE_URL}">the releases page</a>.</p>
    <div class="platforms">
      <article class="platform" data-platform="mac">
        <h3>macOS</h3>
        <p class="badge" hidden>Recommended for your system</p>
        <p>One universal app for Apple silicon and Intel Macs. Signed and notarised by Apple, so it opens without a warning.</p>
        <div class="buttons" data-kind="dmg"><a class="btn primary" href="${LATEST_RELEASE_URL}">Download the .dmg</a></div>
        <p class="note">Open the disk image and drag Siderea Planets to the Applications folder.</p>
      </article>
      <article class="platform" data-platform="windows">
        <h3>Windows</h3>
        <p class="badge" hidden>Recommended for your system</p>
        <p>A standard setup installer.</p>
        <div class="buttons" data-kind="exe"><a class="btn primary" href="${LATEST_RELEASE_URL}">Download the installer</a></div>
        <p class="note"><strong>SmartScreen warning:</strong> the installer is not code-signed yet, so Windows may show &ldquo;Windows protected your PC&rdquo;. Choose <strong>More info</strong>, then <strong>Run anyway</strong>.</p>
      </article>
      <article class="platform" data-platform="linux">
        <h3>Linux</h3>
        <p class="badge" hidden>Recommended for your system</p>
        <p>An AppImage that runs on most distributions, and a .deb package for Debian and Ubuntu.</p>
        <div class="buttons" data-kind="appimage"><a class="btn primary" href="${LATEST_RELEASE_URL}">Download the AppImage</a></div>
        <div class="buttons" data-kind="deb"><a class="btn" href="${LATEST_RELEASE_URL}">Download the .deb</a></div>
        <p class="note">Make the AppImage executable first: <code>chmod +x Siderea*.AppImage</code>, then start it. Install the .deb with <code>sudo apt install ./&lt;file&gt;.deb</code>.</p>
      </article>
    </div>
    <p class="all-releases">Older versions and release notes: <a href="${RELEASES_URL}">all releases</a>. The source code is on <a href="${REPO_URL}">GitHub</a>.</p>
  </div>
</section>
`;
}
