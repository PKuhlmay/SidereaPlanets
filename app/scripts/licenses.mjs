// Shared helpers for `pnpm check:licenses` and `pnpm gen:licenses` (PLANETS-7): the license policy for npm packages and the
// readers for the license texts of the production dependencies.

import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * SPDX identifiers that are compatible with GPL-3.0-or-later and allowed for production npm dependencies. Keep in line with
 * `licenses.allow` in deny.toml (the Rust side); only licenses that occur in the tree belong here.
 */
export const ALLOWED = [
  "MIT",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Zlib",
  "MPL-2.0",
  "CC0-1.0",
  "0BSD",
  "Unlicense",
  "BlueOak-1.0.0",
];

/** Splits an SPDX expression into identifiers, `AND`, `OR`, `WITH` and parentheses. */
function tokenize(expression) {
  return expression.match(/\(|\)|[^\s()]+/g) ?? [];
}

/**
 * Whether an SPDX expression ("MIT", "(MIT OR Apache-2.0)", "A AND B") is satisfied by the allow list: `OR` needs one side
 * allowed, `AND` needs both. An empty or unknown license is never allowed.
 */
export function isAllowed(expression, allowed = ALLOWED) {
  const tokens = tokenize(expression);
  let pos = 0;

  // or := and ("OR" and)*     and := term ("AND" term)*     term := "(" or ")" | id ("WITH" id)?
  const parseOr = () => {
    let ok = parseAnd();
    while (tokens[pos] === "OR") {
      pos += 1;
      const right = parseAnd();
      ok = ok || right;
    }
    return ok;
  };
  const parseAnd = () => {
    let ok = parseTerm();
    while (tokens[pos] === "AND") {
      pos += 1;
      const right = parseTerm();
      ok = ok && right;
    }
    return ok;
  };
  const parseTerm = () => {
    const token = tokens[pos];
    pos += 1;
    if (token === undefined) return false;
    if (token === "(") {
      const ok = parseOr();
      if (tokens[pos] === ")") pos += 1;
      return ok;
    }
    const id = token.replace(/\+$/, "");
    if (tokens[pos] === "WITH") {
      // An exception only widens the license, so the base identifier decides.
      pos += 2;
    }
    return allowed.includes(id);
  };

  const ok = parseOr();
  return ok && pos === tokens.length;
}

/** The license file names that count as the text of a package. */
const LICENCE_FILE = /^(licen[cs]e|copying|notice)(\.|-|$)/i;

function readLicenseText(dir) {
  if (!existsSync(dir)) return "";
  const names = readdirSync(dir).filter((name) => LICENCE_FILE.test(name) && !/\.(js|ts|json)$/.test(name));
  names.sort();
  return names
    .map((name) => readFileSync(join(dir, name), "utf8").trim())
    .filter((text) => text !== "")
    .join("\n\n");
}

/**
 * The production dependencies of the app as `{ name, version, license, homepage, text }`, from `pnpm licenses list --prod`.
 * Only what the Vite build can bundle counts: dev dependencies are tools and not shipped.
 */
export function listNpmLicenses(cwd) {
  const out = execFileSync("pnpm", ["licenses", "list", "--prod", "--json"], {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    // pnpm.cmd on Windows needs a shell.
    shell: process.platform === "win32",
  });
  /** @type {Record<string, Array<{ name: string, versions: string[], paths: string[], homepage?: string }>>} */
  const groups = JSON.parse(out);
  const packages = [];
  for (const [license, entries] of Object.entries(groups)) {
    for (const entry of entries) {
      entry.versions.forEach((version, i) => {
        packages.push({
          name: entry.name,
          version,
          license,
          homepage: entry.homepage ?? "",
          text: readLicenseText(entry.paths[i] ?? entry.paths[0] ?? ""),
        });
      });
    }
  }
  packages.sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));
  return packages;
}
