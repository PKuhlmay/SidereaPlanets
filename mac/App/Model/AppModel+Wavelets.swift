import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The step Wavelets of the window: the live preview of the Processing Area, Process, Do All, the schemes, and the Functions cards
// with Contrast/Brightness and Copy To. The Windows counterparts are app/src/wavelet.ts, colour.ts and preview.ts. The
// calculation runs in the Rust core; this decides when to ask, keeps the pictures that come back and moves the step states.

/// A picture as the pixels the core takes and gives, and as the image the canvas draws.
struct Shot {
    let pixels: RgbaImage
    let image: CGImage

    init?(_ pixels: RgbaImage) {
        guard let image = pixels.makeCGImage() else { return nil }
        self.pixels = pixels
        self.image = image
    }
}

/// The pictures of the Wavelets and Save steps. All of them are the size of the stack.
struct WaveletData {
    /// Counts the stacks: results of an older one are dropped when they come back.
    var stackVersion = 0
    /// The stack, as pixels: what *Before* shows and what a preview is written into.
    var stack: Shot?
    /// The stack with the preview of the Processing Area in it, or the result of Do All.
    var processed: Shot?
    /// The base with the cards and Contrast/Brightness applied; `nil` while they are all neutral.
    var adjusted: Shot?
    /// Copy To: the image kept for comparison, and the difference to it.
    var clipboard: Shot?
    var difference: Shot?
    /// Show mask of the card Masking: the base with the part the sharpening applies to tinted blue; `nil` while it is not wanted.
    var maskOverlay: Shot?
    var toneView = ToneView.current
    /// *Before* in the zoom bar: the stack without anything applied.
    var before = false
    /// The stack and the values the last finished Do All was made for.
    var lastDoAll: DoAllKey?
    /// Counts the base images (stack, preview, Do All result): the cards start from the base, so a new one needs a new source.
    var baseVersion = 0
    /// Flip and Rotate (MAC-017): the shown picture and the stack as View Compare shows it, with the orientation applied.
    var oriented: [OrientedSlot: OrientedShot] = [:]

    /// The image the cards start from: the sharpened one, or the stack before anything ran.
    var base: Shot? { processed ?? stack }
    /// What the steps show without *Before*: the base with the cards applied.
    var current: Shot? { adjusted ?? base }
}

/// The bookkeeping of the running requests; not shown anywhere, so views do not watch it.
@MainActor
final class WaveletWork {
    var sourceTask: Task<Void, Never>?
    /// Only one preview request runs at a time; changes that come meanwhile are folded into one more.
    var previewTask: Task<Void, Never>?
    var previewAgain = false
    var previewWaitsForRun = false
    /// The first preview of a new stack leaves the status bar alone: it still tells the size of the stack (screenshot 08 of the design).
    var quietPreview = false
    /// Counts the finished Do All runs: a preview that was under way when one finished must not paint over its result.
    var doAllCount = 0
    var automaticTask: Task<Void, Never>?
    var colourTask: Task<Void, Never>?
    var coloursAgain = false
    var maskTask: Task<Void, Never>?
    var maskAgain = false
    /// The flipped and rotated copies of the shown pictures: one request at a time, the newest values win.
    var orientTask: Task<Void, Never>?
    var orientAgain = false
    /// The pixels the preview is written into: a copy of the stack, so a new preview costs the area, not the whole image.
    var working: Data?
    /// The base the core's colour source was made from.
    var sentBase: Int?
    /// Open project keeps the values of the file for the first stack, Hold or not.
    var keepForNextStack = false
}

extension AppModel {
    // MARK: Derived state

    /// Grayscale recordings have no colour to process: Colour and LRGB are locked, LRGB also without Colour; RGB Align and Balance too.
    var isMono: Bool { details?.info.color == "Mono" }

    /// The image the steps Wavelets and Save show (what the cards and Copy To make of it, or the plain stack with Before).
    var waveletShownImage: CGImage? { waveletShownShot?.image }

    /// The same as pixels: Flip and Rotate starts from them.
    var waveletShownShot: Shot? {
        let data = waveletData
        if data.before { return data.stack ?? data.base }
        if maskOverlayWanted, data.toneView == .current, let overlay = data.maskOverlay { return overlay }
        let current = data.current ?? data.base
        switch data.toneView {
        case .current: return current
        case .clipboard: return data.clipboard ?? current
        case .difference: return data.difference ?? current
        }
    }

    /// Whether the picture shown is the sharpened one: the chip says "Sharpened stack" then.
    var showsSharpened: Bool { waveletData.processed != nil && !waveletData.before }

    /// The Processing Area in image pixels, `nil` before there is a stack.
    var processingAreaRegion: Region? {
        // The pixels are the truth once they are here; the size of the stack is the same.
        if let pixels = waveletData.stack?.pixels {
            return processingArea(imageWidth: Int(pixels.width), imageHeight: Int(pixels.height), side: wavelet.processingAreaSide)
        }
        guard let stacked else { return nil }
        return processingArea(imageWidth: stacked.width, imageHeight: stacked.height, side: wavelet.processingAreaSide)
    }

    var currentDoAllKey: DoAllKey {
        DoAllKey(stack: waveletData.stackVersion, settings: wavelet.values.core, mode: wavelet.mode)
    }

    /// Whether the result of the last Do All (what Save image writes) matches the stack and the values on screen.
    var doAllIsCurrent: Bool { stacked != nil && waveletData.lastDoAll == currentDoAllKey }

    /// The preview is only calculated in the Wavelets step, where the values are changed: the Save step shows what Do All made, and
    /// entering a step with a finished Do All for these values must not paint the preview over that image.
    var previewWanted: Bool { viewStep == .wavelets && stacked != nil && !doAllIsCurrent }

    /// Open project calls this after it put the values of the file into the panel: they stay for the first stack, Hold or not.
    func keepWaveletValuesForNextStack() {
        waveletWork.keepForNextStack = true
    }

    // MARK: Hooks (called from the didSet of the stored properties)

    /// A new stack, or none: the pictures and results of the old one are gone. Without Hold the layers start over.
    func waveletStackChanged() {
        let work = waveletWork
        work.sourceTask = nil
        work.previewAgain = false
        work.automaticTask?.cancel()
        work.automaticTask = nil
        work.working = nil
        work.sentBase = nil
        var data = WaveletData(stackVersion: waveletData.stackVersion + 1)
        data.clipboard = waveletData.clipboard
        data.baseVersion = waveletData.baseVersion + 1
        waveletData = data
        guard stacked != nil else { return }
        if work.keepForNextStack {
            work.keepForNextStack = false
        } else if !wavelet.hold {
            wavelet.values.layers = WaveletValues.defaults.layers
        }
        work.quietPreview = true
        let version = data.stackVersion
        work.sourceTask = Task { [weak self] in await self?.loadWaveletSource(version) }
        syncWaveletsDone()
    }

    /// The panel changed: values, colour mode or Processing Area start a preview (and Automatic a Do All); the rest only changes the window.
    func waveletOptionsChanged(from old: WaveletOptions) {
        guard wavelet != old else { return }
        if wavelet.functions != old.functions {
            requestMaskOverlay()
            // View Compare shows the stack on the left, which has to be flipped and rotated as well.
            requestOriented()
        }
        let decisive = wavelet.values != old.values || wavelet.mode != old.mode || wavelet.processingAreaSide != old.processingAreaSide
        guard decisive else { return }
        syncWaveletsDone()
        triggerPreview()
    }

    /// A calculation ended: a preview that waited for it starts now.
    func waveletRunEnded() {
        guard waveletWork.previewWaitsForRun else { return }
        waveletWork.previewWaitsForRun = false
        triggerPreview()
    }

    /// The step changed: *Before* starts over, and the Wavelets step shows its preview.
    func waveletViewChanged() {
        waveletData.before = false
        status.position = ""
        requestMaskOverlay()
        requestOriented()
        triggerPreview()
    }

    /// Wavelets is done while the result of Do All matches the stack and the values on screen. A changed result takes the check mark of
    /// Save away for good.
    func syncWaveletsDone() {
        let done = doAllIsCurrent
        if progress.waveletsDone != done { progress.waveletsDone = done }
        if !done && progress.saved { progress.saved = false }
    }

    // MARK: The stack as pixels

    private func loadWaveletSource(_ version: Int) async {
        defer { if waveletData.stackVersion == version { waveletWork.sourceTask = nil } }
        do {
            let pixels = try await backend.stackedImage()
            guard waveletData.stackVersion == version, let shot = Shot(pixels) else { return }
            waveletData.stack = shot
            baseChanged()
        } catch {
            guard waveletData.stackVersion == version else { return }
            status.result = failureText("Showing the stack", error)
        }
    }

    /// Waits for the pixels of the stack; `false` if there is none.
    func ensureWaveletSource() async -> Bool {
        if waveletData.stack != nil { return true }
        guard stacked != nil else { return false }
        if let task = waveletWork.sourceTask { await task.value }
        return waveletData.stack != nil
    }

    /// The base image is another one: the cards apply to it, and the difference follows it.
    private func baseChanged() {
        waveletData.baseVersion += 1
        requestColours()
        requestMaskOverlay()
        refreshDifferenceIfShown()
    }

    // MARK: Preview

    /// Calculates the Processing Area with the current values and shows it in the image (the status bar says how long it took).
    func triggerPreview() {
        guard previewWanted else { return }
        let work = waveletWork
        // A calculation runs (Stack opens this step before it has ended): the preview waits for its end.
        if run != nil {
            work.previewWaitsForRun = true
            return
        }
        if !work.quietPreview { status.action = "Updating preview…" }
        work.previewAgain = true
        // Automatic: once the values stop changing, apply them to the whole image as well.
        work.automaticTask?.cancel()
        work.automaticTask = nil
        if wavelet.automatic {
            work.automaticTask = Task { [weak self] in
                try? await Task.sleep(for: .milliseconds(600))
                guard !Task.isCancelled, let self, self.stacked != nil else { return }
                await self.runDoAll()
            }
        }
        guard work.previewTask == nil else { return }
        work.previewTask = Task { [weak self] in
            guard let self else { return }
            while work.previewAgain {
                work.previewAgain = false
                guard await ensureWaveletSource() else { break }
                let quiet = work.quietPreview
                work.quietPreview = false
                let text = await renderPreview(reportsTime: !quiet)
                if !quiet { status.result = text }
            }
            work.previewTask = nil
        }
    }

    /// One preview into the image; returns the text for the status bar. `cancelled` is asked once the area is calculated: the core
    /// does not stop in the middle of such a small area, so Cancel drops the result when it arrives.
    func renderPreview(reportsTime: Bool = true, cancelled: () -> Bool = { false }) async -> String {
        guard let stack = waveletData.stack, let area = processingAreaRegion else { return "No stacked image: run Stack first" }
        let version = waveletData.stackVersion
        let doAllCount = waveletWork.doAllCount
        // Time from the request to the finished drawing in the image area, shown in the status bar (docs/gui-toolkit.md).
        let started = ContinuousClock.now
        do {
            let preview = try await backend.waveletPreview(wavelet.values.core, area: area, mode: wavelet.mode)
            guard waveletData.stackVersion == version, waveletWork.doAllCount == doAllCount else { return status.result }
            if cancelled() { return "Cancelled" }
            var pixels = waveletWork.working ?? stack.pixels.rgba
            waveletWork.working = nil
            writePreview(preview, into: &pixels, imageWidth: Int(stack.pixels.width))
            waveletWork.working = pixels
            guard let shot = Shot(RgbaImage(width: stack.pixels.width, height: stack.pixels.height, rgba: pixels)) else {
                return "Preview failed: \(FrameConversionError().localizedDescription)"
            }
            setProcessed(shot)
            let elapsed = ContinuousClock.now - started
            let milliseconds = Int((Double(elapsed.components.seconds) * 1000 + Double(elapsed.components.attoseconds) / 1e15).rounded())
            if reportsTime { status.action = "Preview calculated in \(milliseconds) ms" }
            return "Preview updated"
        } catch {
            return failureText("Preview", error)
        }
    }

    private func setProcessed(_ shot: Shot) {
        waveletData.processed = shot
        baseChanged()
    }

    /// Update Preview: calculates the preview area now. Cancel drops the result.
    func runProcess() async {
        guard stacked != nil, !isBusy else { return }
        await runJob("Update Preview", operation: .wavelet) {
            guard await ensureWaveletSource() else { throw PlanetsError.Failed(message: "no stacked image") }
            return await renderPreview(cancelled: { run?.cancelled == true })
        }
    }

    // MARK: Do All

    /// Do All: applies the values to the whole image, with the colour mode of Colour and LRGB, and opens the Save step. Cancel and
    /// Pause reach the core between two layers; a cancelled run leaves the earlier result, here and in the core.
    func doAll() async {
        if await runDoAll() { advance() }
    }

    /// Do All that tells whether the run finished; Cancel and an error give `false`.
    @discardableResult
    func runDoAll() async -> Bool {
        guard stacked != nil, !isBusy else { return false }
        // The values are read now: they belong to this run even if a slider moves while it works.
        let key = currentDoAllKey
        var finished = false
        await runJob("Do All", operation: .wavelet) {
            status.action = "Applying to whole image…"
            _ = await ensureWaveletSource()
            let result = try await backend.waveletDoAll(key.settings, mode: key.mode) { [weak self] update in
                Task { @MainActor in self?.doAllProgress(update) }
            }
            guard let shot = Shot(result) else { throw FrameConversionError() }
            guard waveletData.stackVersion == key.stack else { return "Do All stopped: the stack changed, run it again" }
            // The next preview starts from the stack again, so that its area is not painted over this result.
            waveletWork.working = nil
            waveletWork.doAllCount += 1
            setProcessed(shot)
            waveletData.lastDoAll = key
            syncWaveletsDone()
            finished = true
            return "Do All finished"
        }
        return finished
    }

    /// The core reports each finished layer; the text stays "Applying to whole image…".
    private func doAllProgress(_ update: OperationProgress) {
        guard run != nil, update.total > 0, update.fraction >= status.progress else { return }
        status.progress = update.fraction
    }

    // MARK: Layers

    /// *Reset Wavelets*: the six layers back to their defaults.
    func resetWavelets() {
        wavelet.values.layers = WaveletValues.defaults.layers
    }

    /// A moved slider; with Use Linked Wavelets it takes the others along.
    func moveLevel(_ index: Int, to level: Int) {
        let next = linkedLevels(wavelet.values.layers.map(\.level), moving: index, to: level, linked: wavelet.linked)
        for (i, value) in next.enumerated() where wavelet.values.layers[i].level != value { wavelet.values.layers[i].level = value }
    }

    // MARK: Schemes

    /// Load Scheme: the file replaces scheme, filter, Initial Layer, Step Incr. and the six layers. The new values start a preview; its
    /// message must not replace this one.
    func loadScheme(_ url: URL) async {
        do {
            let loaded = WaveletValues(core: try loadWaveletScheme(path: url.path))
            wavelet.values = loaded
            await settleWaveletPreview()
            status.result = "Scheme loaded: \(url.lastPathComponent)"
        } catch {
            status.result = failureText("Load Scheme", error)
        }
    }

    func saveScheme(_ url: URL) {
        do {
            try saveWaveletScheme(path: url.path, settings: wavelet.values.core)
            status.result = "Scheme saved: \(url.lastPathComponent)"
        } catch {
            status.result = failureText("Save Scheme", error)
        }
    }

    /// Waits until the preview requests and the cards have caught up with the values (for tests and for the message after Load Scheme).
    func settleWaveletPreview() async {
        // Every task clears its own slot when it ends.
        while let task = waveletWork.sourceTask ?? waveletWork.previewTask ?? waveletWork.colourTask ?? waveletWork.orientTask { await task.value }
    }

    // MARK: Functions

    /// A Functions button: opens or closes its card; the buttons without a card say so in the status bar.
    func toggleFunction(_ name: String) {
        if name == compareName && crop.isEditing {
            status.result = "View Compare: finish or cancel the crop first"
            return
        }
        if name == cropFunction {
            if crop.isEditing { cancelCrop() } else { startCrop() }
            return
        }
        guard isFunctionBuilt(name) else {
            status.result = "\(name): not available yet"
            return
        }
        if wavelet.functions.contains(name) {
            closeFunction(name)
        } else {
            wavelet.functions.append(name)
            // The split view and the stack in full exclude each other.
            if name == compareName { waveletData.before = false }
        }
    }

    /// Closes a card: the Functions button comes up, the values stay.
    func closeFunction(_ name: String) {
        wavelet.functions.removeAll { $0 == name }
    }

    /// The cards that are open in the Wavelets step, in the order the buttons were pressed.
    /// The crop tool, while it is open, is one of them in the steps Wavelets and Save, and the first, so its buttons are always in view.
    var openCards: [String] {
        let cards = viewStep == .wavelets ? wavelet.functions.filter(isToneCard) : []
        return crop.isEditing && (viewStep == .wavelets || viewStep == .save) ? [cropFunction] + cards : cards
    }

    /// Contrast/Brightness back to 100 and 0.
    func resetContrast() {
        tone.contrast = 250
        tone.brightness = 0
    }

    /// Puts all card values and Contrast/Brightness back to unchanged.
    func resetTone() {
        let hold = tone.holdContrast
        tone = ToneValues()
        tone.holdContrast = hold
    }

    // MARK: Colour and tone

    /// A card value changed: the cards apply to the base image again.
    func toneChanged(from old: ToneValues) {
        if old.core != tone.core { requestColours() }
        if old.refine != tone.refine { requestMaskOverlay() }
    }

    // MARK: Show mask

    /// Whether the image area shows the mask: Show mask is on, the card Masking is open and the step is Wavelets.
    var maskOverlayWanted: Bool { tone.refine.showMask && openCards.contains("Masking") }

    /// Asks the core for the mask preview (or drops it when it is not wanted). Like the cards: one request at a time, the newest
    /// values win.
    func requestMaskOverlay() {
        let work = waveletWork
        guard maskOverlayWanted else {
            work.maskAgain = false
            if waveletData.maskOverlay != nil { waveletData.maskOverlay = nil }
            return
        }
        work.maskAgain = true
        guard work.maskTask == nil else { return }
        work.maskTask = Task { [weak self] in
            guard let self else { return }
            while work.maskAgain {
                work.maskAgain = false
                await applyMaskOverlay()
            }
            work.maskTask = nil
        }
    }

    private func applyMaskOverlay() async {
        guard await ensureColourSource() else { return }
        let version = waveletData.baseVersion
        do {
            let result = try await backend.refineMaskOverlay(tone.refine.core)
            guard maskOverlayWanted else { return }
            if version == waveletData.baseVersion, let shot = Shot(result) {
                waveletData.maskOverlay = shot
            } else {
                // The base changed while the core worked: the next round starts from the new one.
                waveletWork.maskAgain = true
            }
        } catch is CancellationError {
            return
        } catch {
            status.result = failureText("Show mask", error)
        }
    }

    /// Applies the values to the base. While one request runs, later changes only set a flag, so dragging a slider costs one request at a
    /// time and the last position always wins.
    func requestColours() {
        let work = waveletWork
        work.coloursAgain = true
        guard work.colourTask == nil else { return }
        work.colourTask = Task { [weak self] in
            guard let self else { return }
            while work.coloursAgain {
                work.coloursAgain = false
                await applyColours()
            }
            work.colourTask = nil
        }
    }

    private func applyColours() async {
        let values = tone.core
        guard waveletData.stack != nil || stacked != nil else {
            waveletData.adjusted = nil
            return
        }
        guard !isNeutral(values) else {
            if waveletData.adjusted != nil {
                waveletData.adjusted = nil
                refreshDifferenceIfShown()
            }
            return
        }
        guard await ensureColourSource() else {
            waveletData.adjusted = nil
            return
        }
        let version = waveletData.baseVersion
        do {
            let result = try await backend.applyColors(values)
            // The base may have changed while the core worked; the next round then starts from the new one.
            if version == waveletData.baseVersion, let shot = Shot(result) {
                waveletData.adjusted = shot
                refreshDifferenceIfShown()
            } else {
                waveletWork.coloursAgain = true
            }
        } catch {
            waveletData.adjusted = nil
            status.result = failureText("Color and tone", error)
        }
    }

    /// Sends the base to the core, once per base: sending it is the expensive part.
    func ensureColourSource() async -> Bool {
        guard await ensureWaveletSource(), let base = waveletData.base else { return false }
        let version = waveletData.baseVersion
        if waveletWork.sentBase != version {
            do {
                _ = try await backend.setColorSource(base.pixels)
                waveletWork.sentBase = version
            } catch is CancellationError {
                return false
            } catch {
                status.result = failureText("Color and tone", error)
                return false
            }
        }
        return true
    }

    /// The histogram of the base with the card values applied: before the stretch, or the final result.
    func histogram(after: Bool) async -> HistogramData? {
        guard await ensureColourSource() else { return nil }
        do {
            return try await backend.colorHistogram(tone.core, after: after)
        } catch is CancellationError {
            // The card asked for a newer one meanwhile.
            return nil
        } catch {
            status.result = failureText("Color and tone", error)
            return nil
        }
    }

    /// RGB Align, Auto: puts the shifts that move red and blue onto green into the fields.
    func autoAlign() async {
        guard await ensureColourSource() else {
            status.result = "RGB Align: no image yet"
            return
        }
        do {
            tone.shifts = try await backend.estimateRgbAlign(maxShift: wholeNumber(tone.maxShift, minimum: 1))
            status.result = "RGB Align: shifts found"
        } catch {
            status.result = failureText("RGB Align", error)
        }
    }

    /// Histogram, Auto: sets black and white point so that `clip` percent are cut off at each end.
    func autoLevels() async {
        guard await ensureColourSource() else {
            status.result = "Histogram: no image yet"
            return
        }
        do {
            let found = try await backend.autoHistogramPoints(tone.core, clip: Float(tone.clip))
            tone.black = Int(found.black.rounded())
            tone.white = Int(found.white.rounded())
        } catch {
            status.result = failureText("Histogram", error)
        }
    }

    /// Waits until the cards have caught up (for tests).
    func settleColours() async {
        while let task = waveletWork.colourTask ?? waveletWork.maskTask { await task.value }
    }

    // MARK: Copy To

    /// Copy To: keeps the image on screen (with all cards and Contrast/Brightness) as the Clipboard Image.
    func copyTo() {
        guard let image = waveletData.current else {
            status.result = "Remember: no image yet"
            return
        }
        waveletData.clipboard = image
        waveletData.difference = nil
        waveletData.toneView = .current
        status.result = "Image remembered"
    }

    /// Load to: the copied image becomes the processed image. Its cards are in it already, so they go back to unchanged; applying them
    /// again would count twice.
    func loadTo() {
        guard let copy = waveletData.clipboard else {
            status.result = "Restore: nothing remembered yet"
            return
        }
        waveletData.processed = copy
        waveletData.adjusted = nil
        resetTone()
        waveletData.toneView = .current
        baseChanged()
        status.result = "Remembered image restored"
    }

    /// Difference: the amount of the difference between the image and the copy, amplified; it follows the current image while shown.
    func showDifference() {
        guard waveletData.clipboard != nil else {
            status.result = "Difference: nothing remembered yet"
            return
        }
        guard updateDifference() else {
            status.result = "Difference: the images differ in size"
            return
        }
        waveletData.toneView = .difference
        status.result = "Difference shown"
    }

    /// The Toggle group: Current Image or Clipboard Image.
    func showToneView(_ view: ToneView) {
        if view == .clipboard, waveletData.clipboard == nil { return }
        waveletData.toneView = view
    }

    private func updateDifference() -> Bool {
        guard let now = waveletData.current, let copy = waveletData.clipboard,
              let pixels = differencePixels(now.pixels, copy.pixels), let shot = Shot(pixels)
        else {
            waveletData.difference = nil
            return false
        }
        waveletData.difference = shot
        return true
    }

    private func refreshDifferenceIfShown() {
        if waveletData.toneView == .difference, !updateDifference() { waveletData.toneView = .current }
    }
}
