import Foundation
import Observation
import SidereaPlanetsKit

/// What the CPUs setting needs from the Rust core (the thread pool of the session). `PlanetsSession` is the real one; the tests bring
/// a fake. Kept apart from `SessionBackend` so the calculation calls and this setting do not share a protocol.
protocol CpuControl: Sendable {
    /// The number of logical cores: the default and the upper end of the field.
    var cpuCount: Int { get }
    /// The number of threads the calculations use now.
    var cpus: Int { get }
    /// Uses this many threads from now on; returns the number in use.
    @discardableResult func setCpus(_ count: UInt32) throws -> Int
}

extension PlanetsSession: CpuControl {}

/// A whole number from 1 up to the number of cores; a smaller or larger value becomes the nearest one (as the CPUs field of the
/// Windows app does, app/src/logic/cpus.ts). Anything that is not a usable core count counts as one core.
nonisolated func clampCpus(_ value: Int, cores: Int) -> Int {
    min(max(cores, 1), max(value, 1))
}

/// The CPUs setting: how many threads all calculations use. It starts at all cores (the core's default; the original starts with 1),
/// goes from 1 up to that number and is not saved: the number of cores belongs to the computer.
@MainActor @Observable
final class CpuSettings {
    @ObservationIgnored private let control: (any CpuControl)?
    let maximum: Int
    private(set) var value: Int
    /// The text of the last failure of the core, empty when it worked.
    private(set) var error = ""

    init(control: (any CpuControl)?) {
        self.control = control
        maximum = max(control?.cpuCount ?? 1, 1)
        value = clampCpus(control?.cpus ?? 1, cores: maximum)
    }

    /// Sets the number of threads for the next calculation (a running one finishes with the number it started with).
    func set(_ requested: Int) {
        let wanted = clampCpus(requested, cores: maximum)
        do {
            value = try control?.setCpus(UInt32(wanted)) ?? wanted
            error = ""
        } catch {
            self.error = errorText(error)
        }
    }

    /// The setting of the running app: on the session behind the model, if that is the real one.
    static func live(for model: AppModel) -> CpuSettings {
        CpuSettings(control: model.backend as? any CpuControl)
    }
}
