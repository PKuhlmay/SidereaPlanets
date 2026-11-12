import Foundation
import Observation

/// The app's own CPU load and memory, refreshed about once a second while the monitor is running. The status bar reads `cpuPercent` and
/// `memoryBytes`; the math lives in `ProcessUsage`, the readings come from an injectable `ProcessSampling`.
@MainActor @Observable
final class ResourceMonitor {
    /// Percent of one core (100 = one core fully busy); nil until two readings exist.
    private(set) var cpuPercent: Double?
    private(set) var memoryBytes: Int64?

    @ObservationIgnored private let source: any ProcessSampling
    @ObservationIgnored private let interval: Duration
    @ObservationIgnored private var previous: ProcessSample?
    @ObservationIgnored private var task: Task<Void, Never>?

    init(source: any ProcessSampling = DarwinProcessSampler(), interval: Duration = .seconds(1)) {
        self.source = source
        self.interval = interval
    }

    var isRunning: Bool { task != nil }

    /// Starts polling; does nothing if it already runs.
    func start() {
        guard task == nil else { return }
        task = Task { [weak self, interval] in
            while !Task.isCancelled {
                // A monitor that is gone (its status bar was rebuilt without `stop`) ends its loop.
                guard let self else { return }
                self.refresh()
                try? await Task.sleep(for: interval)
            }
        }
    }

    /// Stops polling and forgets the last reading, so a restart does not average over the pause.
    func stop() {
        task?.cancel()
        task = nil
        previous = nil
        cpuPercent = nil
        memoryBytes = nil
    }

    /// Takes one reading. Without a reading (the source fails) the values go back to nil, so the status bar shows a dash instead of a
    /// stale number.
    func refresh() {
        guard let current = source.sample() else {
            previous = nil
            cpuPercent = nil
            memoryBytes = nil
            return
        }
        memoryBytes = current.memoryBytes
        cpuPercent = previous.flatMap { ProcessUsage.cpuPercent(from: $0, to: current) }
        previous = current
    }
}
