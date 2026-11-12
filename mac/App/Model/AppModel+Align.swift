import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The steps Alignpoints, Align and Limit of the window: what their buttons do, and how the results move the step states. The
// Windows counterpart is app/src/align.ts and jobs.ts. The calculations run in the Rust core; this only starts them and keeps
// what they answer. Re-running Set Alignpoints or Align discards the results built on the old one in the core (`Session`), and
// `discardRating` and `progress` make the window show the same.

extension AppModel {
    // MARK: Runs

    /// Runs one calculation, one at a time. The text it returns is the result in the status bar; Cancel and a failure replace it.
    private func runJob(_ name: String, operation: SessionOperation? = nil, _ job: () async throws -> String) async {
        guard run == nil, !isOpening else { return }
        run = Run(name: name, operation: operation)
        status.progress = 0
        status.action = ""
        status.result = ""
        clock = JobClock()
        tick()
        ticker = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(for: .milliseconds(250))
                self?.tick()
            }
        }
        let result: String
        do {
            result = try await job()
        } catch {
            let cancelled = run?.cancelled == true || (error as? PlanetsError)?.isCancelled == true
            result = cancelled ? "Cancelled" : "Error: \(errorText(error))"
        }
        ticker?.cancel()
        ticker = nil
        status.result = result
        // The bar only runs during a calculation: it is empty again after a finished, cancelled or failed one.
        status.progress = 0
        tick()
        run = nil
    }

    private func tick() {
        status.timeUsed = clock.used()
        status.timeLeft = timeLeft(elapsed: status.timeUsed, progress: status.progress)
    }

    /// The core reports on its own thread; the frames come in order, so a late hop never moves the bar back.
    private func jobProgress(_ progress: OperationProgress) {
        guard run != nil, progress.total > 0, progress.fraction >= status.progress else { return }
        status.progress = progress.fraction
        status.action = "Loading frame: \(progress.done)"
    }

    /// Cancel in the toolbar (⌘.): stops the calculation that runs in the core.
    func cancelRun() {
        guard var current = run, let operation = current.operation else { return }
        current.cancelled = true
        // A paused run waits for Resume; the stop has to reach it.
        if current.paused {
            current.paused = false
            clock.resume()
            backend.pause(operation, false)
        }
        run = current
        backend.cancel(operation)
    }

    /// Pause in the toolbar; the button reads Resume while paused. The time used does not count while paused.
    func togglePause() {
        guard var current = run, let operation = current.operation, !current.cancelled else { return }
        current.paused.toggle()
        if current.paused { clock.pause() } else { clock.resume() }
        run = current
        backend.pause(operation, current.paused)
    }

    /// Whether Cancel and Pause have something to act on.
    var canCancel: Bool { run?.operation != nil && run?.cancelled == false }

    /// After a successful run the panel opens the next open step.
    private func advance() {
        viewStep = progress.nextOpen
    }

    // MARK: Alignpoints

    /// Set Alignpoints on the frame that is shown. A new set starts a new alignment.
    func setAlignpoints() async {
        guard hasRecording, !isBusy else { return }
        if let roi = align.roi, !roomForPoints(roi, edgeDistance: options.edgeDistance) {
            status.result = "The region is too small for Min distance from edge"
            return
        }
        // The frame is read now: the frame bar may move on while the points are searched, but they belong to this one.
        let frame = frameIndex
        let settings = pointSettings(options, roi: align.roi)
        await runJob("Set Alignpoints") {
            let answer = try await backend.setAlignpoints(frame: UInt32(frame), settings)
            align.show(answer)
            // Align and Stack register the other frames against this one (the core keeps the same number).
            align.referenceFrame = frame
            // Points that are set again start a new alignment.
            align.discardRating()
            if answer.points.isEmpty {
                progress = StepProgress(step: .loaded)
                return "No alignpoints found: try other Lo and Hi values"
            }
            progress = StepProgress(step: .points)
            advance()
            return "\(answer.points.count) alignpoints set"
        }
    }

    /// The Threshold slider: the weakest points are dropped. Only until Align has run.
    func setThreshold(_ threshold: Int) {
        guard progress.step == .points, !isBusy else { return }
        align.threshold = threshold
        do {
            align.show(try backend.setAlignpointThreshold(UInt64(max(threshold, 0))))
        } catch {
            status.result = "Error: \(errorText(error))"
        }
    }

    /// The slider moves once Set Alignpoints has found points and until Align has run.
    var canMoveThreshold: Bool { align.total > 0 && progress.step == .points && !isBusy }

    // MARK: R.o.Interest

    /// The region is part of Set Alignpoints and Align, so it only changes before Align.
    var canDrawRegion: Bool { (progress.step == .loaded || progress.step == .points) && !isBusy }

    /// The next drag in the image draws the region; pressed again, the region is dropped.
    func toggleRegionOfInterest() {
        if isDrawingRegion {
            isDrawingRegion = false
            align.roi = nil
            show.roi = false
        } else {
            isDrawingRegion = true
        }
    }

    /// A finished rectangle of the drag, in image pixels: ends the drawing and shows the rectangle.
    func selectRegion(_ rect: RoiSettings) {
        align.roi = rect
        isDrawingRegion = false
        show.roi = true
    }

    // MARK: Align

    /// Aligns every frame on the alignpoints and rates how sharp each one is.
    func runAlign() async {
        guard progress.step >= .points, !isBusy else { return }
        let settings = alignSettings(options, roi: align.roi)
        await runJob("Align", operation: .align) {
            // The core drops its results when Align starts, so the window does the same.
            align.discardRating()
            if progress.step >= .aligned { progress = StepProgress(step: .points) }
            let aligned = try await backend.align(settings) { [weak self] update in
                Task { @MainActor in self?.jobProgress(update) }
            }
            let percent = qualityPercent(aligned.frames.map(\.quality))
            for rating in aligned.frames {
                let i = Int(rating.index)
                guard align.frames.indices.contains(i) else { continue }
                align.frames[i].quality = percent[i]
                align.frames[i].difference = Double(rating.difference)
            }
            progress = StepProgress(step: .aligned)
            advance()
            return "Alignment finished."
        }
    }

    // MARK: Limit

    /// Keeps the sharpest frames. A Limit after Stack makes that stack obsolete, so the step falls back to Limit.
    func runLimit() async {
        guard progress.step >= .aligned, !isBusy else { return }
        let settings = limitSettings(options)
        await runJob("Limit") {
            let answer = try await backend.limit(settings)
            let kept = Set(answer.frames.map(Int.init))
            for i in align.frames.indices { align.frames[i].checked = kept.contains(i) }
            progress = StepProgress(step: .limited)
            advance()
            let qualities = answer.frames.compactMap { align.frames.indices.contains(Int($0)) ? align.frames[Int($0)].quality : nil }
            let messages = limitMessages(kept: answer.frames.count, keptQualities: qualities)
            status.action = messages.action
            return messages.result
        }
    }

    // MARK: Framelist

    func setFrameChecked(_ index: Int, _ checked: Bool) {
        guard align.frames.indices.contains(index) else { return }
        align.frames[index].checked = checked
    }

    func checkAllFrames(_ checked: Bool) {
        for i in align.frames.indices { align.frames[i].checked = checked }
    }

    /// No function yet: the reference program was never observed emptying the list (docs/offene-fragen.md).
    func emptyList() {
        status.result = "Empty List: not available yet"
    }
}
