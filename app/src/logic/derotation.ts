// De-rotate images in the Stack step: what Stack and the command `derotation_info`
// are sent, the texts of the panel, the overlay of the measured disc and the section `derotation` of the project file.

import type { createSettings, Planet } from "./settings";

type Stack = Pick<ReturnType<typeof createSettings>["stack"], "derotate" | "planet" | "cameraAngle" | "mirrored" | "startTime">;

/** Matches `DerotationSettings` of crates/session/src/derotate.rs. */
export interface DerotationRequest {
  planet: Planet;
  cameraAngleDeg: number;
  mirrored: boolean;
  startTimeUtc: number | null;
  useFileTimestamps: boolean;
}

/** The measured disc as the command `derotation_info` returns it (`DiscView` in app/src-tauri/src/stack_commands.rs), in image pixels. */
export interface DiscShape {
  x: number;
  y: number;
  /** Along the equator. */
  semiMajor: number;
  /** Along the axis of the planet. */
  semiMinor: number;
  /** Direction of the major axis, degrees clockwise on the screen from pointing right. */
  angle: number;
  /** Outer edge of Saturn's ring, same direction. */
  ring: [number, number] | null;
  /** Unit vector towards the north pole, y down. */
  north: [number, number];
}

/** `DerotationView` of stack_commands.rs. */
export interface DerotationInfo {
  spanSeconds: number;
  rotationDegrees: number;
  disc: DiscShape | null;
  problem: string | null;
}

export const PLANETS: { value: Planet; title: string }[] = [
  { value: "jupiter", title: "Jupiter" },
  { value: "saturn", title: "Saturn" },
  { value: "mars", title: "Mars" },
];

export const CAMERA_ANGLE = { min: -360, max: 360 };

/** Below this rotation in degrees de-rotating changes next to nothing. */
export const NEGLIGIBLE_DEGREES = 1;

/** What Stack sends as `derotation`; null while De-rotate images is off. The time stamps of the file win over the start time. */
export function derotationRequest(stack: Stack): DerotationRequest | null {
  if (!stack.derotate) return null;
  return {
    planet: stack.planet,
    cameraAngleDeg: stack.cameraAngle,
    mirrored: stack.mirrored,
    startTimeUtc: stack.startTime,
    useFileTimestamps: true,
  };
}

/** The planet a file name names ("JUPITER.AVI", "2023-sat_ir.ser"), as `Planet::from_name` of the core. */
export function planetFromName(name: string): Planet | null {
  const lower = name.toLowerCase();
  if (lower.includes("jup")) return "jupiter";
  if (lower.includes("sat")) return "saturn";
  if (lower.includes("mar")) return "mars";
  return null;
}

/** A length of time as the panel shows it: "0.4 s", "45 s", "3 min 12 s", "1 h 05 min". */
export function spanText(seconds: number): string {
  if (seconds < 10) return `${Math.max(seconds, 0).toFixed(1)} s`;
  const total = Math.max(Math.round(seconds), 0);
  if (total < 60) return `${total} s`;
  if (total < 3600) return `${Math.floor(total / 60)} min ${total % 60} s`;
  return `${Math.floor(total / 3600)} h ${String(Math.floor((total % 3600) / 60)).padStart(2, "0")} min`;
}

/** The line under the settings: span and rotation with a hint when that is too little to matter; or what is missing. */
export function derotationSummary(info: DerotationInfo): { text: string; hint: string | null; isProblem: boolean } {
  if (info.problem) return { text: info.problem, hint: null, isProblem: true };
  return {
    text: `Span ${spanText(info.spanSeconds)} · rotation ${info.rotationDegrees.toFixed(1)}°`,
    hint: info.rotationDegrees < NEGLIGIBLE_DEGREES ? "Too short to matter" : null,
    isProblem: false,
  };
}

const two = (n: number): string => String(n).padStart(2, "0");

/** The start time in UTC as "2026-11-12 08:38 UTC", for the line under the field. */
export function utcText(seconds: number): string {
  const d = new Date(seconds * 1000);
  return `${d.getUTCFullYear()}-${two(d.getUTCMonth() + 1)}-${two(d.getUTCDate())} ${two(d.getUTCHours())}:${two(d.getUTCMinutes())} UTC`;
}

/** The value of the date and time field (local time of the computer, to the minute); empty without a time. */
export function localInputValue(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds)) return "";
  const d = new Date(seconds * 1000);
  return `${d.getFullYear()}-${two(d.getMonth() + 1)}-${two(d.getDate())}T${two(d.getHours())}:${two(d.getMinutes())}`;
}

/** The field back to seconds since 1970 UTC; null for an empty or broken value. */
export function parseLocalInput(value: string): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(value);
  if (!m) return null;
  const [year, month, day, hour, minute] = m.slice(1).map(Number);
  const ms = new Date(year, month - 1, day, hour, minute).getTime();
  return Number.isFinite(ms) ? ms / 1000 : null;
}

interface Ellipse {
  cx: number;
  cy: number;
  rx: number;
  ry: number;
  transform: string;
}

/** The overlay of the Stack step in image pixels: the globe, Saturn's ring edge, and a stroke with N over the north pole. */
export function discOverlay(disc: DiscShape): {
  globe: Ellipse;
  ring: Ellipse | null;
  axis: { x1: number; y1: number; x2: number; y2: number };
  label: { x: number; y: number };
} {
  const transform = `rotate(${disc.angle} ${disc.x} ${disc.y})`;
  const ellipse = (rx: number, ry: number): Ellipse => ({ cx: disc.x, cy: disc.y, rx, ry, transform });
  const [nx, ny] = disc.north;
  const at = (distance: number) => ({ x: disc.x + nx * distance, y: disc.y + ny * distance });
  const from = at(disc.semiMinor);
  const to = at(disc.semiMinor + 14);
  return {
    globe: ellipse(disc.semiMajor, disc.semiMinor),
    // A ring seen edge-on is only a line; half a pixel keeps it visible.
    ring: disc.ring ? ellipse(disc.ring[0], Math.max(disc.ring[1], 0.5)) : null,
    axis: { x1: from.x, y1: from.y, x2: to.x, y2: to.y },
    label: at(disc.semiMinor + 24),
  };
}

// --- Project file ---------------------------------------------------------------------------------------------------

/** The keys of `settings.stack` that belong to the section `derotation`, not to the section `stack`. */
export const DEROTATION_KEYS = ["derotate", "planet", "cameraAngle", "mirrored", "startTime"] as const;

/** The section `derotation` of the project file; `startTimeUtc` only when there is a time. */
export function derotationSection(stack: Stack): Record<string, unknown> {
  const section: Record<string, unknown> = {
    enabled: stack.derotate,
    planet: stack.planet,
    cameraAngle: stack.cameraAngle,
    mirrored: stack.mirrored,
  };
  if (stack.startTime !== null) section.startTimeUtc = stack.startTime;
  return section;
}

const finite = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);

/** Reads the section back; a missing section or a key of the wrong type keeps the value. */
export function applyDerotationSection(section: unknown, stack: Stack): void {
  if (typeof section !== "object" || section === null || Array.isArray(section)) return;
  const s = section as Record<string, unknown>;
  if (typeof s.enabled === "boolean") stack.derotate = s.enabled;
  const planet = PLANETS.find((p) => p.value === s.planet);
  if (planet) stack.planet = planet.value;
  if (finite(s.cameraAngle)) stack.cameraAngle = Math.min(CAMERA_ANGLE.max, Math.max(CAMERA_ANGLE.min, s.cameraAngle));
  if (typeof s.mirrored === "boolean") stack.mirrored = s.mirrored;
  if (finite(s.startTimeUtc)) stack.startTime = s.startTimeUtc;
}
