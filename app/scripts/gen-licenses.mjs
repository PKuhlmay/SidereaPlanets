// `pnpm gen:licenses`: writes app/public/third-party-licenses.html, the license texts of every Rust crate (cargo-about, about.toml)
// and every production npm package that the app ships. Vite copies public/ into the frontend, so the file is inside every build and
// the About dialog shows it. The file is generated and not committed (see the root ignore file).
//
//   pnpm gen:licenses                  strict: fails when cargo-about is missing or a license is not accepted (release builds)
//   pnpm gen:licenses --allow-missing  keeps an existing file, or writes a placeholder, when the tools are not there
//   pnpm gen:licenses --cached         dev: leaves an existing file alone, else like --allow-missing (saves the ~10 s of cargo-about)
//
// Needs cargo-about once: cargo install cargo-about --locked --features cli

import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { listNpmLicenses } from "./licenses.mjs";
import { MARKER, renderNotice } from "./notice.mjs";

const appDir = fileURLToPath(new URL("..", import.meta.url));
const repoDir = fileURLToPath(new URL("../..", import.meta.url));
const output = fileURLToPath(new URL("../public/third-party-licenses.html", import.meta.url));
const cached = process.argv.includes("--cached");
const allowMissing = cached || process.argv.includes("--allow-missing");
const isNotice = () => existsSync(output) && readFileSync(output, "utf8").includes(MARKER);

/** The Rust crates as `{ name, version, license, homepage, text }`, one entry per crate and license text. */
function listCrateLicenses() {
  const out = execFileSync("cargo", ["about", "generate", "--config", "about.toml", "--workspace", "--format", "json"], {
    cwd: repoDir,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  });
  /** @type {{ licenses: Array<{ id: string, text: string, used_by: Array<{ crate: { name: string, version: string, repository?: string | null } }> }> }} */
  const data = JSON.parse(out);
  return data.licenses.flatMap((license) =>
    license.used_by.map(({ crate }) => ({
      name: crate.name,
      version: crate.version,
      license: license.id,
      homepage: crate.repository ?? "",
      text: license.text,
    })),
  );
}

function write(html) {
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, html);
}

if (cached && isNotice()) {
  console.log("third-party-licenses.html exists, not generated again (--cached).");
  process.exit(0);
}

try {
  const packages = [...listCrateLicenses(), ...listNpmLicenses(appDir)];
  write(renderNotice(packages));
  console.log(`third-party-licenses.html written: ${packages.length} packages.`);
} catch (error) {
  if (!allowMissing) throw error;
  const reason = error instanceof Error ? error.message.split("\n")[0] : String(error);
  if (isNotice()) {
    console.warn(`gen:licenses skipped, keeping the existing third-party-licenses.html (${reason})`);
  } else {
    write(renderNotice([], reason));
    console.warn(`gen:licenses skipped, wrote a placeholder (${reason})`);
  }
}
