// The one place that knows where the project lives. Change REPO if the GitHub repository moves.
export const REPO = "PKuhlmay/SidereaPlanets";
export const SITE_NAME = "Siderea Planets";
export const LICENSE = "GPL-3.0-or-later";

export const REPO_URL = `https://github.com/${REPO}`;
export const RELEASES_URL = `${REPO_URL}/releases`;
export const LATEST_RELEASE_URL = `${REPO_URL}/releases/latest`;
export const LICENSE_URL = `${REPO_URL}/blob/main/LICENSE`;
export const ISSUES_URL = `${REPO_URL}/issues`;

// Images on the landing page. Each name resolves to site/assets/<name>.{jpg,png,webp} when such a file exists and
// falls back to the placeholder site/assets/<name>.svg. So real pictures only need to be dropped into site/assets/.
export interface SiteImage {
  name: string;
  alt: string;
  width: number;
  height: number;
}

export const BEFORE: SiteImage = {
  name: "before",
  alt: "Jupiter in a single video frame: blurred and noisy",
  width: 1200,
  height: 800,
};
export const AFTER: SiteImage = {
  name: "after",
  alt: "Jupiter after stacking and wavelet sharpening: cloud belts and fine detail are visible",
  width: 1200,
  height: 800,
};

export const SCREENSHOTS: (SiteImage & { caption: string })[] = [
  {
    name: "screenshot-align",
    alt: "Siderea Planets at the Align step: a planet with alignpoints, the frame list and the step bar",
    caption: "Alignpoints and Align: every frame is lined up and rated",
    width: 1440,
    height: 900,
  },
  {
    name: "screenshot-wavelets",
    alt: "Siderea Planets at the Wavelets step: the sharpened planet next to the six wavelet layers",
    caption: "Wavelets: bring out fine detail layer by layer",
    width: 1440,
    height: 900,
  },
  {
    name: "screenshot-light",
    alt: "Siderea Planets in the light theme at the Best Frames step with the quality graph",
    caption: "Best Frames in the light theme",
    width: 1440,
    height: 900,
  },
];
