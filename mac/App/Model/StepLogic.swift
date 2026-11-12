// Step logic of the window: the seven steps of the step bar, which one is done, possible or locked, which one opens
// next, and the texts that go with them. Same rules as app/src/logic/workflow.ts (docs/design.md, "Zustände").

/// How far the calculation has come: the results that exist.
nonisolated enum Step: Int, Comparable, Sendable {
    case empty = -1, loaded, points, aligned, limited, stacked

    static func < (lhs: Step, rhs: Step) -> Bool { lhs.rawValue < rhs.rawValue }
}

/// The seven steps of the step bar; the left panel shows one of them.
nonisolated enum ViewStep: Int, CaseIterable, Identifiable, Sendable {
    case open, alignpoints, align, limit, stack, wavelets, save

    var id: Int { rawValue }

    /// 1...7, as in "Step 2 of 7".
    var number: Int { rawValue + 1 }

    /// Name in the step bar and in the panel header.
    var label: String {
        switch self {
        case .open: "Open"
        case .alignpoints: "Alignpoints"
        case .align: "Align"
        case .limit: "Best Frames"
        case .stack: "Stack"
        case .wavelets: "Wavelets"
        case .save: "Save"
        }
    }

    /// Label of the action button.
    var action: String {
        switch self {
        case .open: "Select Recording…"
        case .alignpoints: "Set Alignpoints"
        case .align: "Align"
        case .limit: "Keep Best Frames"
        case .stack: "Stack"
        case .wavelets: "Do All"
        case .save: "Save Image"
        }
    }

    /// The sentence under the name in the panel header.
    var sentence: String {
        switch self {
        case .open: "Choose a recording: a SER or AVI video or a series of images. You can also drop it onto the window."
        case .alignpoints: "Places marks on surface details that every frame is lined up on. Move the slider left for more points, right for fewer."
        case .align: "Lines up every frame on the alignpoints and rates how sharp each one is."
        case .limit: "Keeps the sharpest frames and drops the rest. Only the kept frames go into the stack."
        case .stack: "Adds the kept frames into one image with far less noise."
        case .wavelets: "Sharpens the stack. Raise the first layers for fine detail, then press Do All to apply them to the whole image."
        case .save: "Writes the finished image to a file."
        }
    }

    /// What the step is, for the tooltip of its pill (docs/design.md, "Texte und Schreibweise").
    var explanation: String {
        switch self {
        case .open: "Open: choose the video or the images to work on."
        case .alignpoints: "Alignpoints: reference marks on surface details that every frame is lined up on."
        case .align: "Align: lines up every frame on the alignpoints and rates how sharp each frame is."
        case .limit: "Best Frames: choose which frames go into the stack, the sharpest ones."
        case .stack: "Stack: averages the kept frames into one image with far less noise."
        case .wavelets: "Wavelets: sharpens the stack in layers from fine to coarse detail."
        case .save: "Save: writes the finished image to a file."
        }
    }

    /// What the action button does, for its tooltip.
    var actionExplanation: String {
        switch self {
        case .open: "Choose the video or the images to work on."
        case .alignpoints: "Places alignpoints automatically on the frame that is shown."
        case .align: "Lines up every frame on the alignpoints and rates how sharp each frame is."
        case .limit: "Keeps the frames chosen in the settings above and leaves out the others."
        case .stack: "Averages the kept frames into one image with far less noise."
        case .wavelets: "Applies the sharpening to the whole image (the preview shows only the preview area)."
        case .save: "Writes the finished image to a file."
        }
    }

    /// The label of the button of a done step: running it again.
    var againLabel: String { self == .open ? "Select Another Recording…" : "\(action) Again" }

    /// The step before this one in the bar, `nil` before Open.
    var previous: ViewStep? { ViewStep(rawValue: rawValue - 1) }

    /// The step after this one in the bar, `nil` after Save.
    var next: ViewStep? { ViewStep(rawValue: rawValue + 1) }

    /// The result the step needs before it can run. Save needs only Stack: Save image runs a pending Do All itself (RK-009).
    fileprivate var requires: Step {
        switch self {
        case .open: .empty
        case .alignpoints: .loaded
        case .align: .points
        case .limit: .aligned
        case .stack: .limited
        case .wavelets, .save: .stacked
        }
    }

    /// Whether the step is built in this version; the others show their panel frame and a note.
    var isBuilt: Bool { self == .open || self == .alignpoints || self == .align || self == .limit || self == .stack || self == .wavelets || self == .save }
}

/// Everything the step states depend on. `waveletsDone`: the result of Do All matches the stack and the current wavelet
/// values. `saved`: Save image wrote the sharpened image since then.
nonisolated struct StepProgress: Equatable, Sendable {
    var step: Step = .empty
    var waveletsDone = false
    var saved = false

    /// Whether each step's result exists (Wavelets and Save only count with a stack).
    private func isDone(_ id: ViewStep) -> Bool {
        let stacked = step >= .stacked
        switch id {
        case .open: return step >= .loaded
        case .alignpoints: return step >= .points
        case .align: return step >= .aligned
        case .limit: return step >= .limited
        case .stack: return stacked
        case .wavelets: return stacked && waveletsDone
        case .save: return stacked && waveletsDone && saved
        }
    }

    /// The step that opens next: the first one that is not done; Save once everything is done.
    var nextOpen: ViewStep {
        ViewStep.allCases.first { !isDone($0) } ?? .save
    }

    func state(of id: ViewStep) -> StepState {
        if isDone(id) { return StepState(status: .done, hint: "") }
        if step >= id.requires { return StepState(status: .possible, hint: "") }
        return StepState(status: .locked, hint: lockHint)
    }

    /// The tooltip of a locked pill names the next open step's action ("Run Align first").
    private var lockHint: String {
        nextOpen == .open ? "Open a recording first" : "Run \(nextOpen.action) first"
    }

    /// Whether running a done step again throws away results of later steps (the hint under "‹Action› again").
    /// Do All and Save image build nothing that a later step depends on.
    func againDiscards(_ id: ViewStep) -> Bool {
        if id == .wavelets || id == .save { return false }
        return ViewStep.allCases.dropFirst(id.number).contains { isDone($0) }
    }
}

nonisolated enum StepStatus: Sendable {
    case done, possible, locked
}

nonisolated struct StepState: Equatable, Sendable {
    let status: StepStatus
    /// Tooltip of a locked pill: the step that is missing. Empty otherwise.
    let hint: String
}

/// The window title: `Siderea Planets — ‹file›`, with step and percent during a calculation (docs/design.md, "Fensteraufbau").
nonisolated func makeWindowTitle(fileName: String, busy: (name: String, progress: Double)?) -> String {
    if let busy { return "Siderea Planets · \(busy.name) \(Int((busy.progress * 100).rounded()))%" }
    return fileName.isEmpty ? "Siderea Planets" : "Siderea Planets — \(fileName)"
}

/// "00:01:05" for 65 seconds.
nonisolated func formatDuration(_ seconds: Double) -> String {
    let total = max(0, Int(seconds.rounded()))
    return String(format: "%02d:%02d:%02d", total / 3600, (total % 3600) / 60, total % 60)
}
