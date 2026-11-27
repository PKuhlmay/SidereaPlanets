import { describe, expect, it } from "vitest";
import {
  CHECK_INTERVAL_MS,
  RELEASES_URL,
  checkFailureText,
  checkOutcome,
  compareVersions,
  installBlock,
  isCheckDue,
  parseVersion,
  shouldCheckAtStart,
} from "./updates";
import tauriConfig from "../../src-tauri/tauri.conf.json?raw";

const sign = (n: number): number => Math.sign(n);

describe("parseVersion", () => {
  it("reads major, minor, patch and the pre-release identifiers", () => {
    expect(parseVersion("1.2.3")).toEqual({ major: 1, minor: 2, patch: 3, prerelease: [] });
    expect(parseVersion("1.0.0-rc.1")).toEqual({ major: 1, minor: 0, patch: 0, prerelease: ["rc", "1"] });
  });

  it("accepts a leading v and ignores build metadata", () => {
    expect(parseVersion("v0.2.0")).toEqual({ major: 0, minor: 2, patch: 0, prerelease: [] });
    expect(parseVersion("1.0.0+abc123")).toEqual({ major: 1, minor: 0, patch: 0, prerelease: [] });
  });

  it("rejects anything that is not a version", () => {
    expect(parseVersion("")).toBeNull();
    expect(parseVersion("1.2")).toBeNull();
    expect(parseVersion("latest")).toBeNull();
    expect(parseVersion("1.2.3-")).toBeNull();
  });
});

describe("compareVersions", () => {
  it("compares the numbers as numbers, not as text", () => {
    expect(sign(compareVersions("0.1.0", "0.1.0"))).toBe(0);
    expect(sign(compareVersions("0.10.0", "0.9.0"))).toBe(1);
    expect(sign(compareVersions("1.0.0", "2.0.0"))).toBe(-1);
    expect(sign(compareVersions("1.2.10", "1.2.9"))).toBe(1);
  });

  it("puts a pre-release before its release: rc.1 < rc.2 < 1.0.0", () => {
    expect(sign(compareVersions("1.0.0-rc.1", "1.0.0-rc.2"))).toBe(-1);
    expect(sign(compareVersions("1.0.0-rc.2", "1.0.0"))).toBe(-1);
    expect(sign(compareVersions("1.0.0", "1.0.0-rc.2"))).toBe(1);
    expect(sign(compareVersions("1.0.0-rc.10", "1.0.0-rc.9"))).toBe(1);
  });

  it("follows the pre-release order of semver", () => {
    const ordered = ["1.0.0-alpha", "1.0.0-alpha.1", "1.0.0-alpha.beta", "1.0.0-beta", "1.0.0-beta.2", "1.0.0-beta.11", "1.0.0-rc.1", "1.0.0"];
    for (let i = 1; i < ordered.length; i++) expect(sign(compareVersions(ordered[i - 1], ordered[i]))).toBe(-1);
  });

  it("does not let build metadata change the order", () => {
    expect(sign(compareVersions("1.0.0+1", "1.0.0+2"))).toBe(0);
  });

  it("throws on a text that is not a version", () => {
    expect(() => compareVersions("1.0", "1.0.0")).toThrow();
  });
});

describe("isCheckDue", () => {
  const now = 1_700_000_000_000;

  it("is due when there was no check yet", () => {
    expect(isCheckDue(null, now)).toBe(true);
  });

  it("is not due within a day of the last check and due from a day on", () => {
    expect(isCheckDue(now - 60_000, now)).toBe(false);
    expect(isCheckDue(now - CHECK_INTERVAL_MS + 1, now)).toBe(false);
    expect(isCheckDue(now - CHECK_INTERVAL_MS, now)).toBe(true);
  });

  it("checks when the last check lies in the future, as after a clock change", () => {
    expect(isCheckDue(now + 60_000, now)).toBe(true);
  });
});

describe("shouldCheckAtStart", () => {
  const now = 1_700_000_000_000;

  it("never checks when the automatic check is off", () => {
    expect(shouldCheckAtStart(false, null, now)).toBe(false);
    expect(shouldCheckAtStart(false, now - 2 * CHECK_INTERVAL_MS, now)).toBe(false);
  });

  it("checks when it is on and a day has passed", () => {
    expect(shouldCheckAtStart(true, null, now)).toBe(true);
    expect(shouldCheckAtStart(true, now - 2 * CHECK_INTERVAL_MS, now)).toBe(true);
    expect(shouldCheckAtStart(true, now - 1000, now)).toBe(false);
  });
});

describe("checkOutcome", () => {
  it("tells a manual check that nothing is new, and an automatic one nothing at all", () => {
    expect(checkOutcome("manual", null, null)).toBe("upToDate");
    expect(checkOutcome("automatic", null, null)).toBe("silent");
  });

  it("shows a new version to both kinds of check when none was skipped", () => {
    expect(checkOutcome("manual", "1.1.0", null)).toBe("show");
    expect(checkOutcome("automatic", "1.1.0", null)).toBe("show");
  });

  it("does not report a skipped version again to an automatic check, but still to a manual one", () => {
    expect(checkOutcome("automatic", "1.1.0", "1.1.0")).toBe("silent");
    expect(checkOutcome("manual", "1.1.0", "1.1.0")).toBe("show");
  });

  it("reports a version newer than the skipped one to an automatic check", () => {
    expect(checkOutcome("automatic", "1.1.1", "1.1.0")).toBe("show");
    expect(checkOutcome("automatic", "1.0.0", "1.0.0-rc.2")).toBe("show");
    expect(checkOutcome("automatic", "1.0.0-rc.2", "1.0.0-rc.1")).toBe("show");
  });

  it("keeps quiet about an older version than the skipped one", () => {
    expect(checkOutcome("automatic", "1.0.0-rc.1", "1.0.0-rc.2")).toBe("silent");
    expect(checkOutcome("automatic", "1.0.0-rc.2", "1.0.0")).toBe("silent");
  });

  it("ignores a stored skip that is not a version", () => {
    expect(checkOutcome("automatic", "1.1.0", "garbage")).toBe("show");
  });
});

describe("installBlock", () => {
  it("lets the update install when nothing runs and the installation can replace itself", () => {
    expect(installBlock(false, true)).toBeNull();
  });

  it("blocks it during a calculation and says why", () => {
    expect(installBlock(true, true)).toMatch(/calculation is running/i);
  });

  it("blocks it for an installation that cannot update itself", () => {
    expect(installBlock(false, false)).toMatch(/cannot update itself/i);
  });
});

describe("checkFailureText", () => {
  it("names the failure and keeps the message of the error", () => {
    expect(checkFailureText(new Error("error sending request"))).toBe("Could not check for updates: error sending request");
    expect(checkFailureText("offline")).toBe("Could not check for updates: offline");
  });
});

describe("the release location", () => {
  it("is the one the updater endpoint of tauri.conf.json points to", () => {
    const config = JSON.parse(tauriConfig) as {
      plugins: { updater: { endpoints: string[] } };
    };
    expect(config.plugins.updater.endpoints).toEqual([`${RELEASES_URL}/latest/download/latest.json`]);
  });
});
