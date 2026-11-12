import Foundation
import Testing

@testable import SidereaPlanetsKit

/// JUPITER.AVI from `re-local/testdata/astropix/` (not in Git, see docs/testdaten.md): `$PLANETS_RE_LOCAL`, or the
/// `re-local/` of the repository this package lives in. `nil` if the video is not there; the tests are skipped then.
private let jupiter: URL? = {
    let repoRoot = URL(fileURLWithPath: #filePath)
        .deletingLastPathComponent()  // SidereaPlanetsKitTests
        .deletingLastPathComponent()  // Tests
        .deletingLastPathComponent()  // SidereaPlanetsKit
        .deletingLastPathComponent()  // Packages
        .deletingLastPathComponent()  // mac
        .deletingLastPathComponent()  // repository root
    var candidates = [repoRoot.appendingPathComponent("re-local")]
    if let override = ProcessInfo.processInfo.environment["PLANETS_RE_LOCAL"] {
        candidates.insert(URL(fileURLWithPath: override), at: 0)
    }
    return candidates
        .map { $0.appendingPathComponent("testdata/astropix/JUPITER.AVI") }
        .first { FileManager.default.fileExists(atPath: $0.path) }
}()

/// The defaults of the window (`app/src/logic/settings.ts`), as in `crates/session/tests/jupiter_flow.rs`.
private let defaultPointSettings = PointSettings(
    minDistance: 30, edgeDistance: 20, maxMovement: 5, roi: nil,
    intensity: .default, lo: 30, hi: 230, boxSize: 30
)

/// Collects what the progress callback reports; it is called from the calculating thread.
private final class ProgressLog: @unchecked Sendable {
    private let lock = NSLock()
    private var entries: [OperationProgress] = []

    func add(_ progress: OperationProgress) {
        lock.withLock { entries.append(progress) }
    }

    var all: [OperationProgress] { lock.withLock { entries } }
}

@Suite("JUPITER.AVI through SidereaPlanetsKit", .enabled(if: jupiter != nil, "JUPITER.AVI not found in re-local/"))
struct JupiterFlowTests {
    @Test("Open, frame 1, alignpoints, align, limit, stack")
    func fullChain() async throws {
        let session = try PlanetsSession()

        // Open and read frame 1.
        let info = try await session.openRecording([jupiter!])
        #expect(info.width == 1024 && info.height == 680)
        #expect(info.frameCount == 10)
        let frame = try await session.readFrame(0)
        #expect(frame.width == 1024 && frame.height == 680)
        #expect(frame.rgba.count == 1024 * 680 * 4)
        #expect(frame.makeCGImage()?.width == 1024)

        // Alignpoints with the defaults: 78 after the threshold, 10 % either way (docs/testdaten.md).
        let points = try await session.setAlignpoints(frame: 0, defaultPointSettings)
        #expect(points.points.count >= 71 && points.points.count <= 85, "\(points.points.count) points")
        #expect(Int(points.total) >= points.points.count)

        // Align: progress reaches the frame count and carries the operation.
        let log = ProgressLog()
        let aligned = try await session.align(
            AlignSettings(boxSize: 30, maxMovement: 5, roi: nil), progress: log.add)
        #expect(aligned.frames.count == 10)
        #expect(aligned.ranking.first == 0, "frame 1 is the best one")
        #expect(log.all.map(\.done) == Array(1...10))
        #expect(log.all.allSatisfy { $0.operation == .align && $0.total == 10 })

        // Limit: all ten frames stay at 80 %.
        let limited = try session.limit(.lowestQuality(value: 80))
        #expect(limited.frames.count == 10)
        #expect(limited.pointShifts.count == points.points.count)

        // Stack with the settings the window sends.
        let stacked = try await session.stack(
            StackSettings(
                referenceFrame: 0,
                points: points.points.map { PointPosition(x: $0.x, y: $0.y) },
                frames: limited.frames,
                boxSize: 30, maxMovement: 5, nearestPoints: 3,
                correctGeometry: true, normalize: false, edgeSkip: 0, roi: nil))
        #expect(stacked.width == 1024 && stacked.height == 680)
        #expect(stacked.frames == 10)
        let image = try await session.stackedImage()
        #expect(image.width == 1024 && image.height == 680)
        #expect(image.rgba.count == 1024 * 680 * 4)
    }

    @Test("Cancel from the progress callback stops Align with the session's text")
    func cancelStopsAlign() async throws {
        let session = try PlanetsSession()
        _ = try await session.openRecording([jupiter!])
        _ = try await session.setAlignpoints(frame: 0, defaultPointSettings)

        do {
            _ = try await session.align(AlignSettings(boxSize: 30, maxMovement: 5, roi: nil)) { _ in
                session.cancel(.align)
            }
            Issue.record("Align was not cancelled")
        } catch let error as PlanetsError {
            #expect(error.isCancelled)
            #expect(error.message == "cancelled")
        }
    }
}

@Suite("PlanetsSession without a recording")
struct SessionBasicsTests {
    @Test("A step without its input fails with the text of the session")
    func errorsCarryTheSessionText() async throws {
        let session = try PlanetsSession()
        await #expect(throws: PlanetsError.NoRecording(message: "no recording is open")) {
            _ = try await session.readFrame(0)
        }
        await #expect(throws: PlanetsError.NotStacked(message: "stack first")) {
            _ = try await session.stackedImage()
        }
    }

    @Test("The CPUs field clamps to 1 up to the number of cores")
    func cpus() throws {
        let session = try PlanetsSession()
        #expect(try session.setCpus(0) == 1)
        #expect(session.cpus == 1)
        #expect(try session.setCpus(.max) == session.cpuCount)
    }
}
