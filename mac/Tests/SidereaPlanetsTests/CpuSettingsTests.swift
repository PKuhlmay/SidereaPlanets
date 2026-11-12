import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// A stand-in for the thread pool of the core: remembers what it was told and clamps like the core does.
private final class FakeCores: CpuControl, @unchecked Sendable {
    struct Failure: Error, LocalizedError {
        var errorDescription: String? { "the threads could not start" }
    }

    let cpuCount: Int
    private let lock = NSLock()
    private var current: Int
    private var calls: [UInt32] = []
    var fails = false

    init(cores: Int, current: Int? = nil) {
        cpuCount = cores
        self.current = current ?? cores
    }

    var cpus: Int { lock.withLock { current } }
    var received: [UInt32] { lock.withLock { calls } }

    func setCpus(_ count: UInt32) throws -> Int {
        if fails { throw Failure() }
        return lock.withLock {
            calls.append(count)
            current = min(max(Int(count), 1), cpuCount)
            return current
        }
    }
}

struct CpuSettingsTests {
    @Test("The number is a whole number from 1 up to the number of cores")
    func clamps() {
        #expect(clampCpus(0, cores: 8) == 1)
        #expect(clampCpus(-3, cores: 8) == 1)
        #expect(clampCpus(4, cores: 8) == 4)
        #expect(clampCpus(99, cores: 8) == 8)
        #expect(clampCpus(5, cores: 0) == 1)
    }

    @MainActor
    @Test("It starts with what the core uses, which is all cores, and does not change the core")
    func defaultsToAllCores() {
        let cores = FakeCores(cores: 10)
        let settings = CpuSettings(control: cores)
        #expect(settings.value == 10)
        #expect(settings.maximum == 10)
        #expect(cores.received.isEmpty)
    }

    @MainActor
    @Test("A value goes to the core clamped, and the field shows what the core answered")
    func passesClampedValueToCore() {
        let cores = FakeCores(cores: 8)
        let settings = CpuSettings(control: cores)
        settings.set(3)
        #expect(settings.value == 3)
        settings.set(0)
        #expect(settings.value == 1)
        settings.set(500)
        #expect(settings.value == 8)
        #expect(cores.received == [3, 1, 8])
        #expect(cores.cpus == 8)
    }

    @MainActor
    @Test("A failure of the core keeps the old value and shows the text")
    func keepsValueOnFailure() {
        let cores = FakeCores(cores: 8)
        let settings = CpuSettings(control: cores)
        settings.set(2)
        cores.fails = true
        settings.set(6)
        #expect(settings.value == 2)
        #expect(settings.error == "the threads could not start")
        cores.fails = false
        settings.set(6)
        #expect(settings.value == 6)
        #expect(settings.error.isEmpty)
    }

    @MainActor
    @Test("On the real session the field changes the thread count of the core")
    func drivesTheRealSession() throws {
        let session = try PlanetsSession()
        let settings = CpuSettings(control: session)
        #expect(settings.value == session.cpuCount)
        settings.set(1)
        #expect(session.cpus == 1)
        settings.set(.max)
        #expect(session.cpus == session.cpuCount)
        #expect(settings.value == session.cpuCount)
    }

    @MainActor
    @Test("Without a real core (a fake backend) the setting is one core wide and harmless")
    func withoutCore() {
        let settings = CpuSettings(control: nil)
        settings.set(4)
        #expect(settings.maximum == 1)
        #expect(settings.value == 1)
    }
}
