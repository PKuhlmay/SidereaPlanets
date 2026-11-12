import Foundation

// Ported from Siderea (Stackr: SidereaCore/ProcessUsage.swift), reduced to CPU load and memory. Keep the text formats in step with
// Siderea: the suite shows the same readout in the status bar.

/// One reading of a process: the CPU time it has used so far, the memory it holds, and when the reading was taken.
nonisolated struct ProcessSample: Sendable, Equatable {
    /// User plus system CPU time since the process started, in seconds.
    var cpuSeconds: Double
    /// Physical memory the process holds, in bytes.
    var memoryBytes: Int64
    /// Monotonic clock at the time of the reading, in seconds. Only differences are meaningful.
    var uptime: Double
}

/// Where readings come from. The platform provides one (`DarwinProcessSampler`), the math below stays portable and testable.
nonisolated protocol ProcessSampling: Sendable {
    func sample() -> ProcessSample?
}

nonisolated enum ProcessUsage {
    /// Share of all cores at which the load counts as high (the status bar highlights it).
    static let highLoadFraction = 0.9

    /// CPU load between two readings in percent of one core: 100 means one core fully busy, a process running on four cores at once
    /// reaches 400. Nil when the readings are not in order or no time has passed.
    static func cpuPercent(from earlier: ProcessSample, to later: ProcessSample) -> Double? {
        let elapsed = later.uptime - earlier.uptime
        let used = later.cpuSeconds - earlier.cpuSeconds
        guard elapsed > 0, used >= 0 else { return nil }
        return used / elapsed * 100
    }

    /// CPU load as shown in the status bar: the share of the whole machine (`percent` of one core divided by `coreCount`), so six busy
    /// cores of twelve read 50 % instead of 600 %. Whole percent, padded to three digits so the readout does not jitter ("  7 %",
    /// " 50 %", "100 %"), and capped at 100 % (a sample can measure a little more CPU time than wall time times cores). Nil where
    /// there is no reading to show (missing, negative, non-finite, no cores); the view shows its dash then.
    static func cpuReading(_ percent: Double?, coreCount: Int) -> String? {
        guard let percent, percent.isFinite, percent >= 0, coreCount > 0 else { return nil }
        return String(format: "%3d %%", Int(min(percent / Double(coreCount), 100).rounded()))
    }

    /// Process memory in the short unit names B, KB, MB, GB, TB, counted in 1024s: no decimals up to 1 KB and from 100 on, one decimal
    /// in between, in the decimal separator of `locale`. The unit names are the same in every language.
    static func byteText(_ bytes: Int64, locale: Locale = .current) -> String {
        let base = 1024.0
        let units = ["B", "KB", "MB", "GB", "TB"]
        var value = Double(max(bytes, 0))
        var unit = 0
        while value >= base, unit < units.count - 1 {
            value /= base
            unit += 1
        }
        guard unit > 0 else { return "\(Int(value)) B" }

        // Round to one decimal first; rounding can reach the next unit (1023.96 KB reads 1024.0), then show 1.0 of it.
        func tenths(_ value: Double) -> Double { (value * 10).rounded() / 10 }
        if tenths(value) >= base, unit < units.count - 1 {
            value = tenths(value) / base
            unit += 1
        }
        // From 100 on (after rounding, so 99.97 reads "100") no decimal is shown.
        let decimals = tenths(value) >= 100 ? 0 : 1
        return String(format: "%.\(decimals)f", locale: locale, value) + " " + units[unit]
    }

    /// The load counts as high when the process keeps nearly every core busy (`highLoadFraction` of `coreCount` times 100 percent). No
    /// value, a value that is no reading (non-finite, negative) or no cores is never high.
    static func isHigh(cpuPercent: Double?, coreCount: Int) -> Bool {
        guard let cpuPercent, cpuPercent.isFinite, cpuPercent >= 0, coreCount > 0 else { return false }
        return cpuPercent >= highLoadFraction * 100 * Double(coreCount)
    }
}
