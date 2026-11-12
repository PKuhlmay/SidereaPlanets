import Darwin
import Foundation

/// Reads CPU time and memory footprint of this process through `proc_pid_rusage`.
nonisolated struct DarwinProcessSampler: ProcessSampling {
    func sample() -> ProcessSample? {
        var usage = rusage_info_current()
        let status = withUnsafeMutablePointer(to: &usage) { pointer in
            pointer.withMemoryRebound(to: rusage_info_t?.self, capacity: 1) {
                proc_pid_rusage(getpid(), RUSAGE_INFO_CURRENT, $0)
            }
        }
        guard status == 0 else { return nil }

        // CPU times come in mach absolute time units: ticks × numer / denom is nanoseconds.
        var timebase = mach_timebase_info_data_t()
        mach_timebase_info(&timebase)
        let nanoseconds = Double(usage.ri_user_time &+ usage.ri_system_time) * Double(timebase.numer) / Double(timebase.denom)

        return ProcessSample(
            cpuSeconds: nanoseconds / 1_000_000_000,
            memoryBytes: Int64(clamping: usage.ri_phys_footprint),
            uptime: ProcessInfo.processInfo.systemUptime
        )
    }
}
