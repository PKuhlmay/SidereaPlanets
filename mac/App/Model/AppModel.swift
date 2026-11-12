import CoreGraphics
import Foundation
import Observation
import SidereaPlanetsKit

/// The five fields of the status bar (docs/design.md, "Statusleiste").
struct StatusTexts: Equatable {
    /// 0...1
    var progress = 0.0
    var action = ""
    var result = ""
    var position = ""
    var timeUsed = 0.0
    var timeLeft = 0.0
}

/// The state of the window: the recording, the step and the shown frame. Calculations happen in the Rust core behind
/// `backend`; this class only holds what the views show and decides what the buttons do.
@MainActor @Observable
final class AppModel {
    @ObservationIgnored let backend: any SessionBackend
    @ObservationIgnored private let defaults: UserDefaults
    let viewport = ImageViewport()

    /// How far the calculation has come (the results that exist).
    private(set) var progress = StepProgress()
    /// The step the left panel shows. Another step fits the image into the area again, because the area changes with the
    /// frame bar and the right panel.
    var viewStep: ViewStep = .open {
        didSet { if viewStep != oldValue { viewport.fit() } }
    }

    private(set) var details: RecordingDetails?
    private(set) var fileURLs: [URL] = []
    /// The frame the slider and the frame bar show; `image` follows once it is read.
    private(set) var frameIndex = 0
    private(set) var image: CGImage?
    private(set) var isOpening = false
    private(set) var recent: RecentList
    var status = StatusTexts()

    // Frame loading: one read at a time, only the newest wish is read next, and results of a recording that was replaced
    // meanwhile are dropped.
    @ObservationIgnored private var generation = 0
    @ObservationIgnored private var wantedFrame: Int?
    @ObservationIgnored private var pumpGeneration: Int?

    init(backend: any SessionBackend, defaults: UserDefaults = .standard) {
        self.backend = backend
        self.defaults = defaults
        recent = RecentList.load(from: defaults)
    }

    // MARK: Derived state

    var info: RecordingInfo? { details?.info }
    var frameCount: Int { Int(info?.frameCount ?? 0) }
    var hasRecording: Bool { progress.step >= .loaded }
    var stepStates: [ViewStep: StepState] {
        Dictionary(uniqueKeysWithValues: ViewStep.allCases.map { ($0, progress.state(of: $0)) })
    }
    /// A calculation is running: no pill opens and the actions are locked. Nothing is calculated yet in this version.
    var isBusy: Bool { isOpening }

    var windowTitle: String {
        makeWindowTitle(fileName: details?.fileName ?? "", busy: nil)
    }

    /// The steps that work on single frames show the frame bar.
    var showsFrameBar: Bool { hasRecording && [.alignpoints, .align, .limit].contains(viewStep) }

    /// The chip at the top left of the image area: what the image shows.
    var imageCaption: String? {
        image == nil ? nil : "Frame \(frameIndex + 1) of \(frameCount)"
    }

    /// The result of a step as the result line and the tooltip of a done pill show it. Later steps add theirs.
    func result(of step: ViewStep) -> String {
        switch step {
        case .open: details.map { "\($0.fileName) · \($0.info.frameCount) frames" } ?? ""
        default: ""
        }
    }

    // MARK: Steps

    /// A pill of the step bar: done and possible steps open; locked ones and anything during a calculation do not.
    func show(_ step: ViewStep) {
        guard !isBusy, progress.state(of: step).status != .locked else { return }
        viewStep = step
    }

    // MARK: Open

    /// Opens a recording: a video, or one or more still frames as a sequence. A failure keeps what was open.
    func open(_ urls: [URL]) async {
        guard !urls.isEmpty, !isOpening else { return }
        isOpening = true
        defer { isOpening = false }
        status.action = "Opening \(urls[0].lastPathComponent)"
        status.result = ""
        let info: RecordingInfo
        do {
            info = try await backend.openRecording(urls)
        } catch {
            status.action = ""
            status.result = "Error: \(errorText(error))"
            return
        }
        startSession(info, urls: urls)
        recent.push(urls.map(\.path))
        recent.save(to: defaults)
        await loadFrame(0)
        status.action = ""
        status.result = "Loaded \(info.frameCount) frames"
    }

    /// Puts the window into the state "recording loaded"; after Select the next open step is Alignpoints.
    private func startSession(_ info: RecordingInfo, urls: [URL]) {
        generation += 1
        wantedFrame = nil
        progress = StepProgress(step: .loaded)
        details = RecordingDetails(fileName: urls[0].lastPathComponent, info: info)
        fileURLs = urls
        frameIndex = 0
        image = nil
        status = StatusTexts()
        viewStep = progress.nextOpen
    }

    func clearRecent() {
        recent.clear()
        recent.save(to: defaults)
    }

    // MARK: Frames

    /// Shows a frame (0-based). Called for every move of the slider: only one read runs at a time, the newest wish is read
    /// next and the ones in between are dropped.
    func showFrame(_ index: Int) {
        guard hasRecording, (0..<frameCount).contains(index) else { return }
        frameIndex = index
        wantedFrame = index
        guard pumpGeneration != generation else { return }
        pumpGeneration = generation
        let current = generation
        Task { await pump(current) }
    }

    func showNextFrame(by offset: Int) {
        showFrame(min(max(frameIndex + offset, 0), max(frameCount - 1, 0)))
    }

    /// The first frame after opening: awaited, so Loaded is reported with the image on screen.
    private func loadFrame(_ index: Int) async {
        frameIndex = index
        wantedFrame = index
        pumpGeneration = generation
        await pump(generation)
    }

    private func pump(_ current: Int) async {
        defer { if pumpGeneration == current { pumpGeneration = nil } }
        while let index = wantedFrame, current == generation {
            wantedFrame = nil
            do {
                let frame = try await backend.frameImage(UInt32(index))
                guard current == generation else { return }
                viewport.setImageSize(CGSize(width: frame.width, height: frame.height))
                image = frame
            } catch {
                guard current == generation else { return }
                status.result = "Error: \(errorText(error))"
            }
        }
    }
}
