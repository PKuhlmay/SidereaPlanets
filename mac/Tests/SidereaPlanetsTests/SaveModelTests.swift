import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// Stands in for the file dialogs: records what they were asked and answers with a URL (or `nil`, a cancelled dialog).
@MainActor
final class Dialogs {
    var answer: URL?
    private(set) var saveRequests: [SaveRequest] = []
    private(set) var openStarts: [URL?] = []

    init(answer: URL? = nil) { self.answer = answer }

    var save: SaveChooser {
        { [self] request in
            saveRequests.append(request)
            return answer
        }
    }

    var open: ProjectChooser {
        { [self] directory in
            openStarts.append(directory)
            return answer
        }
    }
}

/// Save image, Save Aligned, Save Integrated, Save project and Open project in the app model, on the fake core.
@MainActor
@Suite struct SaveModelTests {
    let backend = FakeBackend()
    let defaults: UserDefaults
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")
    let folder: URL

    init() throws {
        defaults = try #require(UserDefaults(suiteName: "SaveModelTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
        folder = FileManager.default.temporaryDirectory.appendingPathComponent("planets-save-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    }

    private func eventually(_ condition: () async -> Bool) async -> Bool {
        for _ in 0..<400 {
            if await condition() { return true }
            try? await Task.sleep(for: .milliseconds(5))
        }
        return false
    }

    private func limitedRecording() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
    }

    private func stackedRecording() async {
        await limitedRecording()
        await model.runStack()
        #expect(model.progress.step == .stacked)
    }

    private func format(_ id: ExportFormat, _ source: ImageSource = .stack) throws -> FormatInfo {
        try #require(model.formats(for: source).first { $0.id == id })
    }

    // MARK: Save image

    @Test func saveImageWritesTheStackInTheChosenFormatAndProposesTheNameOfTheRecording() async throws {
        await stackedRecording()
        model.show(.stack)
        let dialogs = Dialogs(answer: folder.appendingPathComponent("out.tif"))
        await model.saveImage(try format(.tiff16), choose: dialogs.save)

        let saves = await backend.imageSaves
        #expect(saves.count == 1)
        #expect(saves.first?.source == .stack && saves.first?.format == .tiff16)
        #expect(saves.first?.url == folder.appendingPathComponent("out.tif"))
        #expect(saves.first?.adjustments == nil)
        #expect(dialogs.saveRequests == [SaveRequest(name: "JUPITER", directory: URL(fileURLWithPath: "/data/"), fileExtension: "tif")])
        #expect(model.status.result == "Saved out.tif")
        // The stack is not the sharpened image: the step Save stays as it is.
        #expect(!model.progress.saved && model.viewStep == .stack && !model.isBusy)
        #expect(model.saveState.format == .tiff16)
    }

    @Test func aCancelledDialogWritesNothing() async throws {
        await stackedRecording()
        model.show(.stack)
        await model.saveImage(try format(.png16), choose: Dialogs(answer: nil).save)
        #expect(await backend.imageSaves.isEmpty)
        #expect(model.status.result == "Save cancelled")
    }

    @Test func saveImageNeedsAStackAndAReAlignTakesItAway() async throws {
        await limitedRecording()
        #expect(!model.canSaveImage)
        let dialogs = Dialogs(answer: folder.appendingPathComponent("a.png"))
        await model.saveImage(try format(.png16), choose: dialogs.save)
        #expect(await backend.imageSaves.isEmpty && dialogs.saveRequests.isEmpty)

        await model.runStack()
        #expect(model.canSaveImage)
        // Aligning again drops the stack in the core; Save image stays locked until it is stacked again.
        await model.runAlign()
        #expect(!model.canSaveImage)
        await model.saveImage(try format(.png16), choose: dialogs.save)
        #expect(await backend.imageSaves.isEmpty)
        await model.runLimit()
        await model.runStack()
        #expect(model.canSaveImage)
    }

    @Test func aMissingImageIsReportedBeforeTheDialogOpens() async throws {
        await stackedRecording()
        model.show(.stack)
        backend.log.setImageError("no stacked image to save")
        let dialogs = Dialogs(answer: folder.appendingPathComponent("a.tif"))
        await model.saveImage(try format(.tiff16), choose: dialogs.save)
        #expect(model.status.result == "Save Image failed: no stacked image to save")
        #expect(dialogs.saveRequests.isEmpty && model.stepStates[.save]?.status != .done)
    }

    @Test func aFailedWriteIsReported() async throws {
        await stackedRecording()
        model.show(.stack)
        await backend.setSaveError("disk full")
        await model.saveImage(try format(.tiff16), choose: Dialogs(answer: folder.appendingPathComponent("a.tif")).save)
        #expect(model.status.result == "Save Image failed: disk full")
        #expect(!model.isBusy)
    }

    @Test func theStackStepOffersSevenFileTypesAndTheSaveStepFive() throws {
        let stack = model.formats(for: .stack)
        let wavelet = model.formats(for: .wavelet)
        #expect(stack.map(\.id) == [.bmp, .jpeg, .fits16, .fits32, .tiff16, .png16, .fits32Float])
        #expect(wavelet.map(\.id) == [.bmp, .jpeg, .fits16, .tiff16, .png16])
        #expect(stack.map(\.extension) == ["bmp", "jpg", "fit", "fit", "tif", "png", "fit"])
        #expect(stack.first { $0.id == .tiff16 }?.label == "TIFF (16 bit)")
    }

    @Test func theKeyboardShortcutReusesTheLastFileTypeIfTheStepOffersIt() async throws {
        await stackedRecording()
        model.show(.stack)
        #expect(model.currentFormat(for: .stack)?.id == .tiff16, "TIFF 16 bit is the start choice")
        await model.saveImage(try format(.fits32Float), choose: Dialogs(answer: folder.appendingPathComponent("a.fit")).save)
        #expect(model.currentFormat(for: .stack)?.id == .fits32Float)
        // The Save step does not offer 32 bit FITS: TIFF.
        #expect(model.currentFormat(for: .wavelet)?.id == .tiff16)
    }

    // MARK: The sharpened image (Save step)

    @Test func theSaveStepWritesTheSharpenedImageAndIsDoneAfterwards() async throws {
        await stackedRecording()
        await model.runDoAll()
        model.show(.save)
        let dialogs = Dialogs(answer: folder.appendingPathComponent("sharp.png"))
        await model.saveImage(try format(.png16, .wavelet), choose: dialogs.save)

        let saves = await backend.imageSaves
        #expect(saves.first?.source == .wavelet && saves.first?.format == .png16)
        #expect(model.progress.saved && model.stepStates[.save]?.status == .done)
        #expect(model.saveState.savedName == "sharp.png" && model.result(of: .save) == "sharp.png")
        #expect(model.progress.nextOpen == .save && model.viewStep == .save)
        #expect(model.status.result == "Saved sharp.png")
    }

    @Test func theSharpenedImageRunsDoAllFirst() async throws {
        await stackedRecording()
        model.show(.save)
        #expect(!model.progress.waveletsDone)
        let dialogs = Dialogs(answer: folder.appendingPathComponent("sharp.png"))
        await model.saveImage(try format(.png16, .wavelet), choose: dialogs.save)
        #expect(backend.wavelets.doAlls.count == 1)
        #expect(await backend.imageSaves.count == 1 && dialogs.saveRequests.count == 1)
        #expect(model.status.result == "Saved sharp.png" && model.progress.saved)
    }

    @Test func aNewStackTakesTheSavedMarkAway() async throws {
        await stackedRecording()
        await model.runDoAll()
        model.show(.save)
        await model.saveImage(try format(.png16, .wavelet), choose: Dialogs(answer: folder.appendingPathComponent("s.png")).save)
        #expect(model.progress.saved)
        await model.runStack()
        #expect(!model.progress.saved && model.stepStates[.save]?.status == .possible)
    }

    // MARK: Save Aligned and Save Integrated

    @Test func saveAlignedWritesTheVideoWithTheOptionsOfCreateAvi() async throws {
        await limitedRecording()
        model.stackOptions.framerate = 10
        model.stackOptions.sortOnQuality = true
        // A name typed without the ending gets it.
        let dialogs = Dialogs(answer: folder.appendingPathComponent("j_aligned"))
        await model.saveVideo(.aligned, choose: dialogs.save)

        let saves = await backend.videoSaves
        #expect(saves.count == 1 && saves.first?.integrated == false)
        #expect(saves.first?.url.lastPathComponent == "j_aligned.avi")
        #expect(saves.first?.settings.framerate == 10 && saves.first?.settings.sortOnQuality == true)
        #expect(saves.first?.settings.stack.frames == [2, 5, 9], "the frames that stay after Limit")
        #expect(dialogs.saveRequests == [SaveRequest(name: "JUPITER_aligned", directory: URL(fileURLWithPath: "/data/"), fileExtension: "avi")])
        #expect(model.status.result == "Saved j_aligned.avi (3 frames)")
        #expect(model.run == nil && model.status.progress == 0)
    }

    @Test func saveIntegratedSendsTheAverageAndTheSuffix() async throws {
        await limitedRecording()
        model.stackOptions.averageFrames = 2
        let dialogs = Dialogs(answer: folder.appendingPathComponent("j_integrated.avi"))
        await model.saveVideo(.integrated, choose: dialogs.save)

        let saves = await backend.videoSaves
        #expect(saves.first?.integrated == true && saves.first?.settings.averageFrames == 2)
        #expect(dialogs.saveRequests.first?.name == "JUPITER_integrated")
        #expect(model.status.result == "Saved j_integrated.avi (3 frames)")
    }

    @Test func theVideosNeedAlign() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        #expect(!model.canSaveVideo)
        let dialogs = Dialogs(answer: folder.appendingPathComponent("x.avi"))
        await model.saveVideo(.aligned, choose: dialogs.save)
        #expect(await backend.videoSaves.isEmpty && dialogs.saveRequests.isEmpty)
        await model.runAlign()
        #expect(model.canSaveVideo)
    }

    @Test func aVideoShowsProgressLocksTheWindowAndCanBeCancelled() async throws {
        await limitedRecording()
        await backend.setHoldsVideo(true)
        let dialogs = Dialogs(answer: folder.appendingPathComponent("c.avi"))
        let running = Task { await model.saveVideo(.aligned, choose: dialogs.save) }
        #expect(await eventually { await backend.isVideoHeld && model.status.progress > 0 })
        #expect(model.run?.name == "Save Aligned Video" && model.isBusy && model.canCancel)
        #expect(model.windowTitle.hasPrefix("Siderea Planets · Save Aligned Video"))
        #expect(model.status.action == "Writing frame: 1")
        #expect(!model.canSaveVideo && !model.canSaveImage && !model.canSaveProject)

        model.cancelRun()
        await running.value
        #expect(backend.log.cancelledOperations == [.sequence])
        #expect(model.status.result == "Cancelled" && model.run == nil)
        #expect(model.canSaveVideo)
    }

    @Test func aFailedVideoIsReported() async {
        await limitedRecording()
        await backend.setSaveError("the file is too large")
        await model.saveVideo(.integrated, choose: Dialogs(answer: folder.appendingPathComponent("e.avi")).save)
        #expect(model.status.result == "Save Averaged Video failed: the file is too large")
    }

    @Test func aCancelledVideoDialogStartsNoRun() async {
        await limitedRecording()
        await model.saveVideo(.aligned, choose: Dialogs(answer: nil).save)
        #expect(await backend.videoSaves.isEmpty)
        #expect(model.status.result == "Save cancelled")
    }

    // MARK: Project file

    /// A recording file that exists, so that Open project does not report it missing; the fake core opens anything.
    private func recordingFile(_ name: String = "JUPITER.AVI") throws -> URL {
        let url = folder.appendingPathComponent(name)
        try Data().write(to: url)
        return url
    }

    @Test func aProjectBringsBackTheRecordingAndTheSettings() async throws {
        let recording = try recordingFile()
        await model.open([recording])
        model.options.minDistance = 44
        model.options.limitMode = .bestCount
        model.options.limitCount = 4
        model.show.framelist = true
        model.stackOptions.nearestPoints = 6
        model.stackOptions.codec = "mrle"
        model.stackOptions.averageFrames = 8
        let dialogs = Dialogs(answer: folder.appendingPathComponent("session"))
        await model.saveProject(choose: dialogs.save)

        #expect(dialogs.saveRequests == [SaveRequest(name: "JUPITER", directory: recording.deletingLastPathComponent(), fileExtension: "rkproj")])
        #expect(model.status.result == "Saved session.rkproj", "the core adds the extension")
        let file = folder.appendingPathComponent("session.rkproj")
        let text = try String(contentsOf: file, encoding: .utf8)
        #expect(text.contains("\"formatVersion\": 1") && text.contains("\"align\"") && text.contains("\"stack\""))

        // A fresh window opens the project: the recording and the settings come back.
        let other = AppModel(backend: FakeBackend(), defaults: try #require(UserDefaults(suiteName: "SaveModelTests-\(UUID().uuidString)")))
        let opens = Dialogs(answer: file)
        await other.openProject(choose: opens.open)
        #expect(other.fileURLs.map(\.path) == [recording.path])
        #expect(other.options.minDistance == 44 && other.options.limitMode == .bestCount && other.options.limitCount == 4)
        #expect(other.show.framelist)
        #expect(other.stackOptions.nearestPoints == 6 && other.stackOptions.codec == "mrle" && other.stackOptions.averageFrames == 8)
        #expect(other.status.result == "Opened session.rkproj")
        #expect(other.progress.step == .loaded, "the results are not stored, only the settings")
    }

    @Test func aProjectOfAWindowsFileKeepsWhatItDoesNotKnow() async throws {
        let recording = try recordingFile()
        let file = folder.appendingPathComponent("windows.rkproj")
        let json = """
            {"formatVersion": 1, "recording": ["\(recording.path)"],
             "sections": {"align": {"boxSize": 55, "minDistance": "wrong"}, "wavelet": {"contrast": 250}, "colour": {"colour": true}}}
            """
        try json.write(to: file, atomically: true, encoding: .utf8)
        await model.openProject(choose: Dialogs(answer: file).open)
        #expect(model.options.boxSize == 55 && model.options.minDistance == 30)
        #expect(model.status.result == "Opened windows.rkproj")
    }

    @Test func aMissingRecordingFileIsNamed() async throws {
        let file = folder.appendingPathComponent("gone.rkproj")
        let json = #"{"formatVersion": 1, "recording": ["/nowhere/a.ser", "/nowhere/b.ser"], "sections": {}}"#
        try json.write(to: file, atomically: true, encoding: .utf8)
        await model.openProject(choose: Dialogs(answer: file).open)
        #expect(model.status.result == "Open Project: the recording was not found: a.ser, b.ser")
        #expect(!model.hasRecording)
    }

    @Test func aProjectWithoutARecordingAndOneFromANewerVersionAreRefused() async throws {
        let empty = folder.appendingPathComponent("empty.rkproj")
        try #"{"formatVersion": 1, "recording": [], "sections": {}}"#.write(to: empty, atomically: true, encoding: .utf8)
        await model.openProject(choose: Dialogs(answer: empty).open)
        #expect(model.status.result == "Open Project: the project has no recording")

        let newer = folder.appendingPathComponent("newer.rkproj")
        try #"{"formatVersion": 9, "recording": ["/x.ser"], "sections": {}}"#.write(to: newer, atomically: true, encoding: .utf8)
        await model.openProject(choose: Dialogs(answer: newer).open)
        #expect(model.status.result.hasPrefix("Open Project failed: ") && model.status.result.contains("newer version"))

        let broken = folder.appendingPathComponent("broken.rkproj")
        try "not json".write(to: broken, atomically: true, encoding: .utf8)
        await model.openProject(choose: Dialogs(answer: broken).open)
        #expect(model.status.result.hasPrefix("Open Project failed: not a valid project file"))
    }

    @Test func saveProjectNeedsARecordingAndACancelledDialogDoesNothing() async throws {
        let none = Dialogs(answer: folder.appendingPathComponent("x.rkproj"))
        await model.saveProject(choose: none.save)
        #expect(model.status.result == "Save Project: no recording is open" && none.saveRequests.isEmpty)
        #expect(!model.canSaveProject)

        await model.open([try recordingFile()])
        #expect(model.canSaveProject)
        await model.saveProject(choose: Dialogs(answer: nil).save)
        #expect(model.status.result == "Save cancelled")
        let opens = Dialogs(answer: nil)
        await model.openProject(choose: opens.open)
        #expect(opens.openStarts.count == 1, "the dialog starts in the folder of the recording")
        #expect(model.status.result == "Save cancelled", "a cancelled Open project says nothing")
    }
}
