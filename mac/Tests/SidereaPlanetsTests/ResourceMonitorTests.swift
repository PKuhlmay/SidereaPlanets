import Foundation
import Testing

@testable import SidereaPlanets

/// A stand-in for the system: hands out the prepared readings one after the other; nil where a reading is missing.
private final class ScriptedSampler: ProcessSampling, @unchecked Sendable {
    private let lock = NSLock()
    private var readings: [ProcessSample?]
    private var taken = 0

    init(_ readings: [ProcessSample?]) { self.readings = readings }

    var takenCount: Int { lock.withLock { taken } }

    func sample() -> ProcessSample? {
        lock.withLock {
            taken += 1
            return readings.isEmpty ? nil : readings.removeFirst()
        }
    }
}

private func reading(cpu: Double, memory: Int64, at uptime: Double) -> ProcessSample {
    ProcessSample(cpuSeconds: cpu, memoryBytes: memory, uptime: uptime)
}

@MainActor
struct ResourceMonitorTests {
    @Test("The first reading has the memory but no CPU load yet")
    func firstReading() {
        let monitor = ResourceMonitor(source: ScriptedSampler([reading(cpu: 1, memory: 500, at: 10)]))
        monitor.refresh()
        #expect(monitor.memoryBytes == 500)
        #expect(monitor.cpuPercent == nil)
    }

    @Test("The second reading gives the CPU load between the two")
    func secondReading() {
        let monitor = ResourceMonitor(source: ScriptedSampler([
            reading(cpu: 1, memory: 500, at: 10), reading(cpu: 4, memory: 700, at: 11),
        ]))
        monitor.refresh()
        monitor.refresh()
        #expect(monitor.cpuPercent == 300)
        #expect(monitor.memoryBytes == 700)
    }

    @Test("A missing reading clears the values instead of leaving old ones")
    func missingReading() {
        let monitor = ResourceMonitor(source: ScriptedSampler([reading(cpu: 1, memory: 500, at: 10), nil]))
        monitor.refresh()
        monitor.refresh()
        #expect(monitor.memoryBytes == nil)
        #expect(monitor.cpuPercent == nil)
    }

    @Test("Stopping clears the values, and a restart does not average over the pause")
    func stopForgetsEverything() {
        let monitor = ResourceMonitor(source: ScriptedSampler([
            reading(cpu: 1, memory: 500, at: 10), reading(cpu: 2, memory: 600, at: 11), reading(cpu: 90, memory: 650, at: 500),
        ]))
        monitor.refresh()
        monitor.refresh()
        monitor.stop()
        #expect(monitor.cpuPercent == nil)
        #expect(monitor.memoryBytes == nil)
        // The first reading after the pause has no earlier one to compare with.
        monitor.refresh()
        #expect(monitor.cpuPercent == nil)
        #expect(monitor.memoryBytes == 650)
    }

    @Test("Started, the monitor polls by itself; stopped, it stops polling")
    func startAndStop() async throws {
        let sampler = ScriptedSampler((0..<10_000).map { reading(cpu: Double($0) * 0.01, memory: 1024, at: Double($0) * 0.01) })
        let monitor = ResourceMonitor(source: sampler, interval: .milliseconds(5))
        #expect(!monitor.isRunning)
        monitor.start()
        monitor.start()  // a second start does nothing
        #expect(monitor.isRunning)
        try await waitUntil { monitor.cpuPercent != nil }
        #expect(monitor.cpuPercent != nil)

        monitor.stop()
        #expect(!monitor.isRunning)
        #expect(monitor.cpuPercent == nil)
        try await Task.sleep(for: .milliseconds(30))
        let afterStop = sampler.takenCount
        try await Task.sleep(for: .milliseconds(60))
        #expect(sampler.takenCount == afterStop)
    }

    /// Waits up to two seconds for the condition; the test then fails at its own assertion.
    private func waitUntil(_ condition: @MainActor () -> Bool) async throws {
        for _ in 0..<400 where !condition() {
            try await Task.sleep(for: .milliseconds(5))
        }
    }
}
