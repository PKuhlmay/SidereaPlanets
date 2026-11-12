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

@Suite("De-rotation through SidereaPlanetsKit", .enabled(if: jupiter != nil, "JUPITER.AVI not found in re-local/"))
struct DerotationFlowTests {
    private func pointsAndStackSettings(_ session: PlanetsSession, derotation: DerotationSettings?) async throws -> StackSettings {
        let points = try await session.setAlignpoints(frame: 0, defaultPointSettings)
        return StackSettings(
            referenceFrame: 0, points: points.points.map { PointPosition(x: $0.x, y: $0.y) }, frames: Array(0..<10),
            boxSize: 30, maxMovement: 5, nearestPoints: 3, correctGeometry: true, normalize: false, edgeSkip: 0, roi: nil,
            derotation: derotation)
    }

    @Test("The recording suggests a start time, the panel gets span, rotation and the disc, and Stack runs with De-rotate images")
    func derotationOnTheRealRecording() async throws {
        let session = try PlanetsSession()
        let info = try await session.openRecording([jupiter!])
        // An AVI has no time stamps: the start is guessed from the date of the file and says so.
        #expect(!info.hasTimestamps && info.startTime != nil && info.startTimeFromFileDate)
        #expect(derotationPlanetFromName(text: jupiter!.lastPathComponent) == .jupiter)

        let settings = DerotationSettings(
            planet: .jupiter, cameraAngleDeg: 0, mirrored: false, startTimeUtc: info.startTime, useFileTimestamps: true)
        let stackSettings = try await pointsAndStackSettings(session, derotation: settings)
        let answer = try await session.derotationInfo(settings, referenceFrame: 0, frames: Array(0..<10))
        #expect(answer.problem == nil)
        // Ten frames of a video are a fraction of a second: nothing turns, and the panel says so.
        #expect(answer.spanSeconds > 0 && answer.spanSeconds < 2 && answer.rotationDegrees < 0.1)
        let disc = try #require(answer.disc)
        #expect(disc.semiMajor > 50 && disc.semiMinor <= disc.semiMajor && disc.x > 0 && disc.x < 1024 && disc.y > 0 && disc.y < 680)
        #expect(disc.ring.isEmpty)

        let stacked = try await session.stack(stackSettings)
        #expect(stacked.width == 1024 && stacked.height == 680 && stacked.frames == 10)
    }

    @Test("Without a start time the panel says what is missing and Stack fails with the same words")
    func missingStartTime() async throws {
        let session = try PlanetsSession()
        _ = try await session.openRecording([jupiter!])
        let settings = DerotationSettings(
            planet: .jupiter, cameraAngleDeg: 0, mirrored: false, startTimeUtc: nil, useFileTimestamps: true)
        let answer = try await session.derotationInfo(settings, referenceFrame: 0, frames: Array(0..<10))
        #expect(answer.problem == "De-rotation needs the start time of the recording" && answer.disc == nil)
        let stackSettings = try await pointsAndStackSettings(session, derotation: settings)
        await #expect(throws: PlanetsError.Failed(message: "De-rotation needs the start time of the recording")) {
            _ = try await session.stack(stackSettings)
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
