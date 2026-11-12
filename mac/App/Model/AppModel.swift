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

/// A running calculation (Set Alignpoints, Align, Limit, Stack): its name for the window title and whether it can be cancelled and paused.
struct Run: Equatable {
    var name: String
    /// The core operation Cancel and Pause act on; `nil` for the calls that are over in a moment.
    var operation: SessionOperation?
    var paused = false
    var cancelled = false
}

/// The state of the window: the recording, the step and the shown frame. Calculations happen in the Rust core behind
/// `backend`; this class only holds what the views show and decides what the buttons do.
@MainActor @Observable
final class AppModel {
    @ObservationIgnored let backend: any SessionBackend
    @ObservationIgnored private let defaults: UserDefaults
    let viewport = ImageViewport()

    /// How far the calculation has come (the results that exist). Written by the open and run functions only.
    var progress = StepProgress() {
        didSet {
            // Set Alignpoints, Align or Limit ran again (or another recording opened): the stack made on the old result is gone.
            if progress.step < .stacked { stacked = nil }
            // A step that became locked (Set Alignpoints or Align ran again and dropped what came after) cannot stay open.
            if progress.state(of: viewStep).status == .locked { viewStep = progress.nextOpen }
        }
    }
    /// The step the left panel shows. Another step fits the image into the area again, because the area changes with the
    /// frame bar and the right panel.
    var viewStep: ViewStep = .open {
        didSet {
            guard viewStep != oldValue else { return }
            syncViewport()
            viewport.fit()
        }
    }

    /// The values of the panels Alignpoints, Align and Limit, and the check marks of the Show menu.
    var options = AlignOptions()
    var show = ShowOptions()
    /// The points, the rating of the frames and the region of the Align tab.
    var align = AlignData()
    /// The values of the Stack panel and the check marks of its Show menu.
    var stackOptions = StackOptions()
    /// What the step Save keeps: the file type of Save image, the name it wrote last, a write under way.
    var saveState = SaveState()
    /// The stack, from Stack until Set Alignpoints, Align or Limit runs again.
    var stacked: StackedImage?
    /// The next drag in the image draws the region (R.o.Interest).
    var isDrawingRegion = false
    /// A calculation is running: no pill opens and the actions are locked.
    var run: Run?
    @ObservationIgnored var clock = JobClock()
    @ObservationIgnored var ticker: Task<Void, Never>?
    /// The steps whose Advanced section is open; kept until closed again, also after a restart.
    private(set) var advancedOpen: Set<ViewStep>

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
        advancedOpen = Self.loadAdvanced(from: defaults)
    }

    // MARK: Derived state

    var info: RecordingInfo? { details?.info }
    var frameCount: Int { Int(info?.frameCount ?? 0) }
    var hasRecording: Bool { progress.step >= .loaded }
    var stepStates: [ViewStep: StepState] {
        Dictionary(uniqueKeysWithValues: ViewStep.allCases.map { ($0, progress.state(of: $0)) })
    }
    /// Opening or a calculation is running: no pill opens and the actions are locked.
    var isBusy: Bool { isOpening || run != nil || saveState.isWriting }

    var windowTitle: String {
        makeWindowTitle(fileName: details?.fileName ?? "", busy: run.map { (name: $0.name, progress: status.progress) })
    }

    /// The steps that work on single frames show the frame bar.
    var showsFrameBar: Bool { hasRecording && [.alignpoints, .align, .limit].contains(viewStep) }

    /// The Stack step (and the ones after it) show the stack once it exists, the earlier steps and the Stack step before it show the
    /// frame (`pickImage` of app/src/logic/workflow.ts).
    var showsStack: Bool { stacked != nil && [.stack, .wavelets, .save].contains(viewStep) }

    /// The image the image area shows.
    var shownImage: CGImage? { showsStack ? stacked?.image : image }

    /// The chip at the top left of the image area: what the image shows.
    var imageCaption: String? {
        if let stacked, showsStack { return "Stack of \(stacked.frames) frames" }
        return image == nil ? nil : "Frame \(frameIndex + 1) of \(frameCount)"
    }

    /// The viewport follows the size of the image that is shown (the stack has the size of the frames, but the view must not rely on it).
    private func syncViewport() {
        if let shown = shownImage { viewport.setImageSize(CGSize(width: shown.width, height: shown.height)) }
    }

    /// The result of a step as the result line and the tooltip of a done pill show it. Later steps add theirs.
    func result(of step: ViewStep) -> String {
        switch step {
        case .open: details.map { "\($0.fileName) · \($0.info.frameCount) frames" } ?? ""
        case .alignpoints: "\(align.points.count) set"
        case .align: "best quality \(Int((align.bestQuality ?? 0).rounded())) %"
        case .limit: "\(align.keptCount) of \(frameCount) kept"
        case .stack: stacked.map { stackResultText(frames: $0.frames, width: $0.width, height: $0.height) } ?? ""
        case .save: saveState.savedName
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
        guard !urls.isEmpty, !isBusy else { return }
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
        align = AlignData(frameCount: Int(info.frameCount))
        isDrawingRegion = false
        details = RecordingDetails(fileName: urls[0].lastPathComponent, info: info)
        fileURLs = urls
        frameIndex = 0
        image = nil
        status = StatusTexts()
        viewStep = progress.nextOpen
    }

    // MARK: Advanced

    private static let advancedKey = "rk-advanced"

    private static func loadAdvanced(from defaults: UserDefaults) -> Set<ViewStep> {
        Set((defaults.array(forKey: advancedKey) as? [Int] ?? []).compactMap(ViewStep.init(rawValue:)))
    }

    func toggleAdvanced(_ step: ViewStep) {
        if advancedOpen.contains(step) { advancedOpen.remove(step) } else { advancedOpen.insert(step) }
        defaults.set(advancedOpen.map(\.rawValue).sorted(), forKey: Self.advancedKey)
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
                image = frame
                if !showsStack { syncViewport() }
            } catch {
                guard current == generation else { return }
                status.result = "Error: \(errorText(error))"
            }
        }
    }
}
