import CoreGraphics
import Foundation
import SidereaPlanetsKit

// The step Save and the project file: Save image, Save Aligned, Save Integrated, Save project and Open project. The Windows
// counterpart is app/src/saving.ts. The files are written by the Rust core (`Session`); this decides what is saved, asks for the
// name and reports. The dialogs are passed in (`choose`), so the tests run the same code without a window.

/// Asks where to write a file; `nil` is a cancelled dialog. The real one is `SavePanels.save`.
typealias SaveChooser = @MainActor (SaveRequest) async -> URL?
/// Asks which project file to open, starting in the given folder; `nil` is a cancelled dialog. The real one is `SavePanels.openProject`.
typealias ProjectChooser = @MainActor (URL?) async -> URL?

extension AppModel {
    // MARK: What can be saved

    /// The image Save image writes: the sharpened one in the steps Wavelets and Save, the stack in all others (`imageKind` of workflow.ts).
    var saveSource: ImageSource { viewStep == .wavelets || viewStep == .save ? .wavelet : .stack }

    /// Save image needs the stack. A new Align (or Set Alignpoints, Limit) drops it in the core and in `stacked`, so the button is
    /// locked until the stack is made again.
    var canSaveImage: Bool { stacked != nil && !isBusy }

    /// The file types of Save image in a step: seven for the stack, five for the sharpened image (`export_formats`).
    func formats(for source: ImageSource) -> [FormatInfo] { exportFormats(source: source) }

    /// The file type of ⌘S: the one chosen last, if the step offers it, else TIFF 16 bit.
    func currentFormat(for source: ImageSource) -> FormatInfo? {
        let list = formats(for: source)
        return list.first { $0.id == saveState.format } ?? list.first { $0.id == .tiff16 } ?? list.first
    }

    /// Save project is possible with any recording; Open project and every save only while nothing runs.
    var canSaveProject: Bool { hasRecording && !isBusy }

    // MARK: Hooks of the Wavelets step (MAC-008)

    /// The colour and tone values the sharpened image is written with (`colourPayload` of colour.ts): all cards and Contrast/Brightness, `nil`
    /// while they leave the image as it is and for the stack.
    var saveAdjustments: ColorSettings? {
        guard saveSource == .wavelet else { return nil }
        let values = tone.core
        return isNeutral(values) ? nil : values
    }

    /// Makes the sharpened image match the current wavelet values: runs Do All if the layers, Colour or LRGB changed since the last one
    /// or none ran yet (`doAllIsUpToDate() || runDoAll()`). `false` when it was cancelled or failed; then nothing is saved.
    func ensureProcessedImage() async -> Bool {
        if doAllIsCurrent { return true }
        return await runDoAll()
    }

    // MARK: Save image

    /// Save image in `format`: the stack in the Stack step, the sharpened image (with the colour and tone values applied) in Wavelets and
    /// Save. A pending Do All runs first; then the dialog opens. A saved sharpened image marks the step Save done.
    func saveImage(_ format: FormatInfo, choose: SaveChooser) async {
        guard canSaveImage else { return }
        let source = saveSource
        saveState.format = format.id
        if source == .wavelet, !(await ensureProcessedImage()) { return }
        // Fail before the dialog if there is nothing to write (the core knows, the window may be behind).
        do { try backend.requireImage(source) } catch {
            status.result = failureText("Save Image", error)
            return
        }
        let request = SaveRequest(name: fileStem(details?.fileName ?? ""), directory: recordingFolder, fileExtension: format.extension)
        guard let url = await choose(request) else {
            status.result = "Save cancelled"
            return
        }
        saveState.isWriting = true
        defer { saveState.isWriting = false }
        do {
            let written = try await backend.saveImage(source, format: format.id, to: url, adjustments: saveAdjustments)
            status.result = "Saved \(written.lastPathComponent)"
            if source == .wavelet {
                saveState.savedName = written.lastPathComponent
                progress.saved = true
                advance()
            }
        } catch {
            status.result = failureText("Save Image", error)
        }
    }

    // MARK: Save Aligned and Save Integrated

    /// Create AVI needs Align: the video is made from the checked frames and alignpoints, so it can be written before Stack.
    var canSaveVideo: Bool { progress.step >= .aligned && !isBusy }

    /// The dialog comes first, then the video is written as a run with progress, Cancel and Pause (`saveVideo` of saving.ts).
    func saveVideo(_ kind: VideoKind, choose: SaveChooser) async {
        guard canSaveVideo else { return }
        let settings = sequenceSettings(stackOptions, align: options, data: align)
        let request = SaveRequest(name: fileStem(details?.fileName ?? "") + kind.suffix, directory: recordingFolder, fileExtension: "avi")
        guard let chosen = await choose(request) else {
            status.result = "Save cancelled"
            return
        }
        // The core writes exactly the path it gets; a name typed without the ending gets it here (the dialog adds it as well).
        let url = URL(fileURLWithPath: withAviExtension(path: chosen.path))
        await runJob(kind.name, operation: .sequence) {
            status.action = "Writing frame: 0"
            let handler: ProgressHandler = { [weak self] update in
                Task { @MainActor in self?.jobProgress(update, label: "Writing frame") }
            }
            let video: SavedVideo
            switch kind {
            case .aligned: video = try await backend.saveAligned(to: url, settings, progress: handler)
            case .integrated: video = try await backend.saveIntegrated(to: url, settings, progress: handler)
            }
            return "Saved \(URL(fileURLWithPath: video.path).lastPathComponent) (\(video.frames) frames)"
        }
    }

    // MARK: Project file

    /// The sections of the project file, one per step that has settings, with the layout of the Windows app (`buildProject` of
    /// projectfile.ts): `wavelet` holds all settings of the Wavelet tab, `colour` is `{colour, lrgb}`.
    var projectSections: [String: JSON] {
        [
            "align": alignSection(options, show: show), "stack": stackSection(stackOptions),
            "wavelet": waveletSection(wavelet, tone: tone), "colour": colourSection(wavelet),
            // Mac only (MAC-018): the Windows app does not know the section and ignores it.
            "refine": refineSection(tone.refine),
        ]
    }

    /// Puts the settings of a loaded project into the window. The Wavelets step stays locked until Stack, so the values of the file would
    /// be gone before anyone saw them: they are kept for the first stack, Hold or not.
    func applyProjectSections(_ sections: [String: JSON]) {
        applyAlignSection(sections["align"], to: &options, show: &show)
        applyStackSection(sections["stack"], to: &stackOptions)
        var waveletOptions = wavelet
        var toneValues = tone
        applyWaveletSection(sections["wavelet"], to: &waveletOptions, tone: &toneValues)
        applyColourSection(sections["colour"], to: &waveletOptions)
        applyRefineSection(sections["refine"], to: &toneValues.refine)
        wavelet = waveletOptions
        tone = toneValues
        keepWaveletValuesForNextStack()
    }

    /// Save project: the recording and the settings of the steps. The results of Align and Stack are not stored; they are calculated again.
    func saveProject(choose: SaveChooser) async {
        guard hasRecording else {
            status.result = "Save Project: no recording is open"
            return
        }
        guard !isBusy else { return }
        let request = SaveRequest(name: fileStem(details?.fileName ?? ""), directory: recordingFolder, fileExtension: "rkproj")
        guard let url = await choose(request) else {
            status.result = "Save cancelled"
            return
        }
        do {
            let project = makeProject(recording: fileURLs, sections: projectSections)
            let written = try projectSave(project: project, path: url.path)
            status.result = "Saved \(URL(fileURLWithPath: written).lastPathComponent)"
        } catch {
            status.result = failureText("Save Project", error)
        }
    }

    /// Open project: opens the recording of the project and puts its settings back (`openProject` of saving.ts).
    func openProject(choose: ProjectChooser) async {
        guard !isBusy else { return }
        guard let url = await choose(recordingFolder) else { return }
        do {
            let loaded = try projectLoad(path: url.path)
            let project = loaded.project
            if !loaded.missingFiles.isEmpty {
                let names = loaded.missingFiles.map { URL(fileURLWithPath: $0).lastPathComponent }
                status.result = "Open Project: the recording was not found: \(names.joined(separator: ", "))"
                return
            }
            if project.recording.isEmpty {
                status.result = "Open Project: the project has no recording"
                return
            }
            await open(project.recording.map { URL(fileURLWithPath: $0) })
            // `open` reports its own error; the settings only apply if the recording is the one that is open now.
            guard fileURLs.map(\.path) == project.recording else { return }
            applyProjectSections(sections(of: project))
            status.result = "Opened \(url.lastPathComponent)"
        } catch {
            status.result = failureText("Open Project", error)
        }
    }

    /// The folder of the open recording: where the dialogs start.
    var recordingFolder: URL? { fileURLs.first?.deletingLastPathComponent() }
}
