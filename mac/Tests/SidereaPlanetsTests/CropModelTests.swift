import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The crop tool in the app model, on the fake core (the stack of the fake is 1024 x 680): opening and closing, the formats, Done, Cancel
/// and Reset, what Save Image is told, and when the rectangle goes away.
@MainActor
@Suite struct CropModelTests {
    let backend = FakeBackend()
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")
    let folder: URL

    init() throws {
        let defaults = try #require(UserDefaults(suiteName: "CropModelTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
        folder = FileManager.default.temporaryDirectory.appendingPathComponent("planets-crop-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    }

    private func stackedRecording() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        await model.settleWaveletPreview()
        #expect(model.progress.step == .stacked && model.viewStep == .wavelets)
    }

    private func format(_ id: ExportFormat) throws -> FormatInfo {
        try #require(model.formats(for: .wavelet).first { $0.id == id })
    }

    // MARK: Opening and the formats

    @Test func theToolOpensOnAMiddleRectangleOfTheStack() async throws {
        await stackedRecording()
        #expect(model.canCrop && !model.crop.isEditing)
        model.startCrop()
        #expect(model.crop.isEditing && model.crop.format == .free)
        let draft = try #require(model.crop.draft)
        // 80 % of 1024 x 680 (819 x 544), in the middle.
        #expect(draft == CropRectangle(minX: 102, minY: 68, maxX: 921, maxY: 612))
        #expect(model.cropSizeText == "819 × 544 px")
        #expect(model.crop.applied == nil)
    }

    @Test func cropIsOnlyPossibleWhereTheSharpenedStackIsShown() async {
        model.startCrop()
        #expect(!model.crop.isEditing)
        await stackedRecording()
        model.show(.stack)
        #expect(!model.canCrop)
        model.startCrop()
        #expect(!model.crop.isEditing)
        model.show(.wavelets)
        model.startCrop()
        #expect(model.crop.isEditing)
    }

    @Test func aFixedFormatReshapesTheRectangleAndKeepsItInsideTheImage() async throws {
        await stackedRecording()
        model.startCrop()
        for (format, long, short) in [(CropFormat.square, 1, 1), (.threeByTwo, 3, 2), (.fourByThree, 4, 3)] {
            model.setCropFormat(format)
            let rect = try #require(model.crop.draft)
            #expect(model.crop.format == format && model.cropAspect == format.aspect)
            // The shape is exact to half a pixel; the rectangle stays inside 1024 x 680 and keeps lying.
            #expect(abs(Double(rect.width) * Double(short) - Double(rect.height) * Double(long)) <= Double(long))
            #expect(rect.minX >= 0 && rect.minY >= 0 && rect.maxX <= 1024 && rect.maxY <= 680)
            #expect(rect.width >= rect.height)
        }
        // Free lets go of the shape and keeps the rectangle.
        let before = model.crop.draft
        model.setCropFormat(.free)
        #expect(model.cropAspect == nil && model.crop.draft == before)
    }

    @Test func aDraftOutsideTheImageIsPulledBackInside() async throws {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: -50, minY: -10, maxX: 2000, maxY: 700))
        #expect(model.crop.draft == CropRectangle(minX: 0, minY: 0, maxX: 1024, maxY: 680))
        model.setCropDraft(CropRectangle(minX: 900, minY: 600, maxX: 905, maxY: 603))
        let small = try #require(model.crop.draft)
        #expect(small.width >= CropRectangle.minimumSize && small.height >= CropRectangle.minimumSize && small.maxX <= 1024 && small.maxY <= 680)
    }

    @Test func theToolDoesNothingWhileNoToolIsOpen() async {
        await stackedRecording()
        model.setCropDraft(CropRectangle(minX: 10, minY: 10, maxX: 100, maxY: 100))
        model.setCropFormat(.square)
        #expect(model.crop.draft == nil)
    }

    // MARK: Done, Cancel, Reset

    @Test func doneKeepsTheRectangleForSaving() async {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()
        #expect(!model.crop.isEditing && model.crop.draft == nil)
        #expect(model.crop.applied == CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        #expect(model.cropSizeText == "800 × 600 px")
        #expect(model.saveCrop == CropRect(x: 100, y: 50, width: 800, height: 600))
    }

    @Test func cancelThrowsAwayWhatWasChangedSinceTheToolOpened() async {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()

        model.startCrop()
        // The tool opens on the applied rectangle.
        #expect(model.crop.draft == CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.setCropDraft(CropRectangle(minX: 0, minY: 0, maxX: 200, maxY: 200))
        model.cancelCrop()
        #expect(!model.crop.isEditing && model.crop.draft == nil)
        #expect(model.crop.applied == CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
    }

    @Test func resetGoesBackToTheWholeImageAndDoneThenCropsNothing() async {
        await stackedRecording()
        model.startCrop()
        model.setCropFormat(.fourByThree)
        model.resetCrop()
        #expect(model.crop.isEditing && model.crop.draft == CropRectangle.full(imageWidth: 1024, imageHeight: 680))
        #expect(model.crop.format == .free)
        model.finishCrop()
        #expect(model.crop.applied == nil && model.saveCrop == nil)
    }

    @Test func aRectangleThatCoversTheWholeImageIsNotACrop() async {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle.full(imageWidth: 1024, imageHeight: 680))
        model.finishCrop()
        #expect(model.crop.applied == nil && model.saveCrop == nil)
    }

    @Test func removeTakesTheAppliedRectangleAway() async {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()
        model.removeCrop()
        #expect(model.crop.applied == nil && model.saveCrop == nil && model.cropSizeText == nil)
    }

    // MARK: Save Image

    @Test func saveImageIsToldTheRectangleInTheWaveletsAndSaveSteps() async throws {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()
        model.show(.save)
        await model.saveImage(try format(.tiff16), choose: Dialogs(answer: folder.appendingPathComponent("a.tif")).save)
        let saves = await backend.imageSaves
        #expect(saves.count == 1 && saves.first?.source == .wavelet)
        #expect(saves.first?.crop == CropRect(x: 100, y: 50, width: 800, height: 600))
    }

    @Test func saveImageWritesTheWholeImageWithoutACrop() async throws {
        await stackedRecording()
        model.show(.save)
        await model.saveImage(try format(.png16), choose: Dialogs(answer: folder.appendingPathComponent("a.png")).save)
        #expect(await backend.imageSaves.first?.crop == nil)
    }

    @Test func theStackSavedFromTheStackStepIsNotCropped() async throws {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()
        model.show(.stack)
        let stackFormat = try #require(model.formats(for: .stack).first { $0.id == .tiff16 })
        await model.saveImage(stackFormat, choose: Dialogs(answer: folder.appendingPathComponent("s.tif")).save)
        let saves = await backend.imageSaves
        #expect(saves.first?.source == .stack && saves.first?.crop == nil)
    }

    // MARK: How long the rectangle lasts

    @Test func theRectangleStaysThroughTheStepsAndAfterSaving() async throws {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()
        model.show(.save)
        await model.saveImage(try format(.tiff16), choose: Dialogs(answer: folder.appendingPathComponent("a.tif")).save)
        model.show(.wavelets)
        #expect(model.crop.applied == CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
    }

    @Test func aNewStackTakesTheRectangleAway() async {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()
        // Aligning again drops the stack, the rectangle goes with it, and the next stack starts without one.
        await model.runAlign()
        #expect(model.crop.applied == nil && !model.crop.isEditing)
        await model.runLimit()
        await model.runStack()
        #expect(model.crop.applied == nil)
    }

    @Test func anotherRecordingTakesTheRectangleAway() async {
        await stackedRecording()
        model.startCrop()
        model.finishCrop()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        model.finishCrop()
        await model.open([URL(fileURLWithPath: "/data/SATURN.AVI")])
        #expect(model.crop == CropState())
    }

    @Test func leavingTheStepsOfTheSharpenedImageClosesTheTool() async {
        await stackedRecording()
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        // Wavelets to Save keeps the tool open: both show the same image.
        model.show(.save)
        #expect(model.crop.isEditing)
        model.show(.stack)
        #expect(!model.crop.isEditing && model.crop.draft == nil && model.crop.applied == nil)
    }

    // MARK: The Crop button of the Adjustments panel

    @Test func theCropButtonOpensAndClosesTheTool() async {
        await stackedRecording()
        #expect(isFunctionBuilt(cropFunction) && functionHelp(cropFunction) != "Not available yet")
        model.toggleFunction(cropFunction)
        #expect(model.crop.isEditing && model.openCards.contains(cropFunction))
        // Closing the card is Cancel, and the button is not one of the cards that stay in `wavelet.functions`.
        #expect(!model.wavelet.functions.contains(cropFunction))
        model.toggleFunction(cropFunction)
        #expect(!model.crop.isEditing && !model.openCards.contains(cropFunction))
    }

    @Test func theCardOfTheToolAlsoShowsInTheStepSave() async {
        await stackedRecording()
        model.startCrop()
        model.show(.save)
        #expect(model.openCards == [cropFunction])
    }

    // MARK: View Compare

    @Test func openingTheToolEndsTheSplitView() async {
        await stackedRecording()
        model.toggleFunction(compareName)
        #expect(model.isComparing)
        model.startCrop()
        #expect(model.crop.isEditing && !model.isComparing && !model.compareIsOn && model.compareBeforeImage == nil)
    }

    @Test func theSplitViewCannotBeTurnedOnWhileTheToolIsOpenButAfterwards() async {
        await stackedRecording()
        model.startCrop()
        model.toggleFunction(compareName)
        #expect(!model.compareIsOn && !model.isComparing)
        #expect(model.status.result == "View Compare: finish or cancel the crop first")
        model.finishCrop()
        model.toggleFunction(compareName)
        #expect(model.isComparing)
    }
}
