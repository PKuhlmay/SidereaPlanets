import { describe, expect, it } from "vitest";
import { createMonitor } from "./resource";
import type { ProcessSample } from "./logic/processUsage";

// The monitor behind the CPU and RAM readout of the status bar (RK-019), with readings handed in by the test.
function fake(readings: (ProcessSample | null)[]) {
  return createMonitor(async () => readings.shift() ?? null);
}

describe("the resource monitor", () => {
  it("shows the memory at once and the CPU load from the second reading on", async () => {
    const monitor = fake([
      { cpuSeconds: 10, memoryBytes: 1_572_864, uptime: 100 },
      { cpuSeconds: 12, memoryBytes: 2_097_152, uptime: 101 },
    ]);
    await monitor.refresh();
    expect(monitor.state).toEqual({ cpuPercent: null, memoryBytes: 1_572_864 });
    await monitor.refresh();
    expect(monitor.state).toEqual({ cpuPercent: 200, memoryBytes: 2_097_152 });
  });

  it("forgets everything when a reading fails or the monitor stops, so no stale value stays on screen", async () => {
    const monitor = fake([
      { cpuSeconds: 1, memoryBytes: 10, uptime: 1 },
      { cpuSeconds: 2, memoryBytes: 10, uptime: 2 },
      null,
      { cpuSeconds: 3, memoryBytes: 10, uptime: 3 },
    ]);
    await monitor.refresh();
    await monitor.refresh();
    await monitor.refresh();
    expect(monitor.state).toEqual({ cpuPercent: null, memoryBytes: null });
    // After a failed reading the next one starts over: no load averaged across the gap.
    await monitor.refresh();
    expect(monitor.state.cpuPercent).toBeNull();
    monitor.stop();
    expect(monitor.state).toEqual({ cpuPercent: null, memoryBytes: null });
  });
});
