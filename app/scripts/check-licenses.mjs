// `pnpm check:licenses`: fails when a production npm dependency has a license that is not GPL-3.0-compatible (PLANETS-7).
// The Rust side is `cargo deny check` (deny.toml).

import { fileURLToPath } from "node:url";
import { ALLOWED, isAllowed, listNpmLicenses } from "./licenses.mjs";

const root = fileURLToPath(new URL("..", import.meta.url));
const packages = listNpmLicenses(root);
const rejected = packages.filter((p) => !isAllowed(p.license));

if (rejected.length > 0) {
  console.error("npm dependencies with a license that is not allowed:");
  for (const p of rejected) console.error(`  ${p.name}@${p.version}: ${p.license || "(no license)"}`);
  console.error(`Allowed: ${ALLOWED.join(", ")}. Check GPL-3.0 compatibility before adding one (app/scripts/licenses.mjs).`);
  process.exit(1);
}
console.log(`npm licenses ok: ${packages.length} production packages, all GPL-3.0-compatible.`);
