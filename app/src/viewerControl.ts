// The zoom of the image area for the menu bar and the keys: the image area (ImageViewer.vue) registers its zoom functions
// here while it is on screen.

export const viewerControl: {
  zoom: ((factor: number) => void) | null;
  fit: (() => void) | null;
  actualSize: (() => void) | null;
} = { zoom: null, fit: null, actualSize: null };
