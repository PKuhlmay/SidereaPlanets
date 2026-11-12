import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// Flip and Rotate and Resize Image in the app model, on the fake core (the stack of the fake is 1024 x 680 by its size and 6 x 4 pixels as a
/// picture; the pictures are turned by the real core): the buttons and cards, the preview, the crop on the turned picture, what Save Image is
/// told, the project file and how long the values last.
@MainActor
@Suite struct GeometryModelTests {
    let backend = FakeBackend()
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")
    let folder: URL

    init() throws {
        let defaults = try #require(UserDefaults(suiteName: "GeometryModelTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
        folder = FileManager.default.temporaryDirectory.appendingPathComponent("planets-geometry-\(UUID().uuidString)")
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

    private func applyCrop(_ rectangle: CropRectangle) {
        model.startCrop()
        model.setCropDraft(rectangle)
        model.finishCrop()
    }

    // MARK: The buttons and cards

    @Test func bothButtonsAreUnlockedAndOpenTheirCards() async {
        await stackedRecording()
        for name in [flipRotateFunction, resizeFunction] {
            #expect(isFunctionBuilt(name) && functionHelp(name) != "Not available yet")
            model.toggleFunction(name)
            #expect(model.openCards.contains(name) && model.wavelet.functions.contains(name))
        }
        model.toggleFunction(flipRotateFunction)
        #expect(!model.openCards.contains(flipRotateFunction) && model.openCards.contains(resizeFunction))
    }

    @Test func nothingChangesBeforeTheValuesDo() async {
        await stackedRecording()
        #expect(model.geometry == GeometryValues())
        #expect(model.saveGeometry == nil && !model.orientationApplies)
        #expect(model.orientedSize?.width == 1024 && model.orientedSize?.height == 680)
        #expect(model.resultSizeText == "1024 × 680 px")
        #expect(model.shownImage === model.waveletShownImage)
        #expect(await backend.wavelets.orientRequests.isEmpty)
    }

    // MARK: The preview

    @Test func aQuarterTurnTurnsTheShownPictureAndFitsTheView() async {
        await stackedRecording()
        let plain = model.shownImage
        #expect(plain?.width == 6 && plain?.height == 4)
        model.rotate(by: 90)
        await model.settleWaveletPreview()
        #expect(model.orientationApplies)
        #expect(model.shownImage?.width == 4 && model.shownImage?.height == 6)
        #expect(model.viewport.imageSize == CGSize(width: 4, height: 6))
        #expect(await backend.wavelets.orientRequests.last?.angle == 90)
        // Back to nothing: the plain picture is shown again, and the size fits again.
        model.resetOrientation()
        #expect(model.shownImage === model.waveletShownImage && !model.orientationApplies)
        #expect(model.shownImage?.width == 6)
    }

    @Test func aFreeAngleGrowsTheCanvasOfThePicture() async {
        await stackedRecording()
        model.setAngle(15)
        await model.settleWaveletPreview()
        let shown = model.shownImage
        #expect((shown?.width ?? 0) > 6 && (shown?.height ?? 0) > 4)
        // With Crop to fit the picture stays inside the turned one.
        model.setCropToFit(true)
        await model.settleWaveletPreview()
        #expect((model.shownImage?.width ?? 99) < (shown?.width ?? 0))
    }

    @Test func theStepSaveShowsTheTurnedPictureTooAndTheStackStepDoesNot() async {
        await stackedRecording()
        model.rotate(by: 90)
        await model.settleWaveletPreview()
        model.show(.save)
        await model.settleWaveletPreview()
        #expect(model.orientationApplies && model.shownImage?.width == 4)
        model.show(.stack)
        #expect(!model.orientationApplies && model.shownImage?.width == FakeBackend.stackPictureWidth)
    }

    @Test func aNewPictureFromTheCardsIsTurnedAsWell() async {
        await stackedRecording()
        model.rotate(by: 90)
        await model.settleWaveletPreview()
        let before = model.shownImage
        model.tone.gamma = 1.5
        await model.settleColours()
        await model.settleWaveletPreview()
        #expect(model.shownImage !== before)
        #expect(model.shownImage?.width == 4 && model.shownImage?.height == 6)
    }

    @Test func viewCompareTurnsTheStackOnTheLeftLikeTheImageOnTheRight() async {
        await stackedRecording()
        model.toggleFunction(compareName)
        model.rotate(by: 90)
        await model.settleWaveletPreview()
        #expect(model.isComparing)
        #expect(model.compareBeforeImage?.width == 4 && model.compareBeforeImage?.height == 6)
        #expect(model.shownImage?.width == 4)
    }

    @Test func theMarksOfTheStackAreNotDrawnOnATurnedPicture() async {
        await stackedRecording()
        #expect(!model.orientationApplies)
        model.flipHorizontally()
        #expect(model.orientationApplies)
    }

    // MARK: The crop on the turned picture

    @Test func theCropToolWorksOnThePictureAsItIsShown() async throws {
        await stackedRecording()
        model.rotate(by: 90)
        await model.settleWaveletPreview()
        #expect(model.cropImageSize?.width == 680 && model.cropImageSize?.height == 1024)
        model.startCrop()
        // 80 % of 680 x 1024 in the middle.
        let draft = try #require(model.crop.draft)
        #expect(draft == CropRectangle(minX: 68, minY: 102, maxX: 612, maxY: 921))
        model.setCropDraft(CropRectangle(minX: 0, minY: 0, maxX: 900, maxY: 1100))
        #expect(model.crop.draft == CropRectangle(minX: 0, minY: 0, maxX: 680, maxY: 1024))
        model.setCropDraft(CropRectangle(minX: 40, minY: 100, maxX: 640, maxY: 900))
        model.finishCrop()
        #expect(model.saveCrop == CropRect(x: 40, y: 100, width: 600, height: 800))
    }

    @Test func turningOrFlippingTakesAnAppliedCropAway() async {
        await stackedRecording()
        applyCrop(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        // The way the pixels are looked up changes nothing about where they are.
        model.setRotationMethod(.bilinear)
        model.setResizePercent(50)
        #expect(model.crop.applied != nil)
        model.flipHorizontally()
        #expect(model.crop.applied == nil && model.status.result == "Crop removed: the picture was flipped or turned")
        applyCrop(CropRectangle(minX: 10, minY: 10, maxX: 300, maxY: 300))
        model.setAngle(20)
        #expect(model.crop.applied == nil)
        applyCrop(CropRectangle(minX: 10, minY: 10, maxX: 300, maxY: 300))
        model.setCropToFit(true)
        #expect(model.crop.applied == nil)
    }

    @Test func theOrientationCannotChangeWhileTheCropToolIsOpen() async {
        await stackedRecording()
        model.startCrop()
        model.rotate(by: 90)
        model.flipVertically()
        model.setAngle(30)
        #expect(!model.canEditOrientation && model.geometry.orientation == OrientationValues())
        #expect(model.status.result == "Flip and Rotate: finish or cancel the crop first")
        model.cancelCrop()
        model.rotate(by: 90)
        #expect(model.geometry.orientation.angle == 90)
    }

    // MARK: Resize Image

    @Test func resizeStartsFromTheCropAndNotFromTheWholePicture() async {
        await stackedRecording()
        model.setResizePercent(50)
        #expect(model.resizeSource?.width == 1024 && model.resultSizeText == "512 × 340 px")
        applyCrop(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650))
        #expect(model.resizeSource?.width == 800 && model.resultSizeText == "400 × 300 px")
        // The card also follows the rectangle that is being dragged.
        model.startCrop()
        model.setCropDraft(CropRectangle(minX: 0, minY: 0, maxX: 200, maxY: 100))
        #expect(model.resultSizeText == "100 × 50 px")
    }

    @Test func pixelFieldsFollowTheLockAndTheResultOfARotation() async {
        await stackedRecording()
        model.setResizeMode(.pixels)
        #expect(model.resultSize?.width == 1024 && model.resultSize?.height == 680)
        model.setResizeWidth(512)
        #expect(model.resultSizeText == "512 × 340 px")
        model.setResizeLock(false)
        model.setResizeHeight(100)
        #expect(model.resultSizeText == "512 × 100 px")
    }

    // MARK: Save Image

    @Test func saveImageIsToldTheOrientationTheCropAndTheResize() async throws {
        await stackedRecording()
        model.rotate(by: 90)
        model.setCropToFit(true)
        applyCrop(CropRectangle(minX: 40, minY: 100, maxX: 640, maxY: 900))
        model.setResizePercent(50)
        model.setResizeMethod(.bicubic)
        model.show(.save)
        await model.saveImage(try format(.tiff16), choose: Dialogs(answer: folder.appendingPathComponent("a.tif")).save)
        let saved = try #require(await backend.imageSaves.first)
        #expect(saved.source == .wavelet)
        #expect(saved.crop == CropRect(x: 40, y: 100, width: 600, height: 800))
        let geometry = try #require(saved.geometry)
        #expect(geometry.orientation.angle == 90 && geometry.orientation.cropToFit && !geometry.orientation.flipHorizontal)
        #expect(geometry.resize == ResizeSettings(target: .percent(percent: 50), method: .bicubic))
    }

    @Test func saveImageSendsNoGeometryWhileNothingChanges() async throws {
        await stackedRecording()
        model.show(.save)
        await model.saveImage(try format(.png16), choose: Dialogs(answer: folder.appendingPathComponent("a.png")).save)
        #expect(await backend.imageSaves.first?.geometry == nil)
    }

    @Test func aResizeToTheSizeThePictureHasIsNotSent() async {
        await stackedRecording()
        model.setResizePercent(100)
        model.setResizeMode(.pixels)
        model.setResizeWidth(1024)
        #expect(model.saveGeometry == nil)
    }

    @Test func theStackSavedFromTheStackStepIsNeitherTurnedNorResized() async throws {
        await stackedRecording()
        model.rotate(by: 90)
        model.setResizePercent(50)
        model.show(.stack)
        let stackFormat = try #require(model.formats(for: .stack).first { $0.id == .tiff16 })
        await model.saveImage(stackFormat, choose: Dialogs(answer: folder.appendingPathComponent("s.tif")).save)
        let saved = await backend.imageSaves
        #expect(saved.first?.source == .stack && saved.first?.geometry == nil && saved.first?.crop == nil)
    }

    // MARK: Reset, stack and project

    @Test func eachCardResetsOnlyItsOwnValues() async {
        await stackedRecording()
        model.rotate(by: 90)
        model.flipVertically()
        model.setResizePercent(200)
        model.resetOrientation()
        #expect(model.geometry.orientation == OrientationValues() && model.geometry.resize.percent == 200)
        model.rotate(by: 90)
        model.resetResize()
        #expect(model.geometry.resize == ResizeValues() && model.geometry.orientation.angle == 90)
    }

    @Test func theValuesStayWithANewStackButTheCropGoes() async {
        await stackedRecording()
        model.rotate(by: 90)
        model.setResizePercent(50)
        applyCrop(CropRectangle(minX: 40, minY: 100, maxX: 640, maxY: 900))
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        await model.settleWaveletPreview()
        #expect(model.crop.applied == nil)
        #expect(model.geometry.orientation.angle == 90 && model.geometry.resize.percent == 50)
        #expect(model.shownImage?.width == 4 && model.shownImage?.height == 6)
    }

    @Test func theProjectKeepsTheValuesInItsOwnSection() async throws {
        await stackedRecording()
        model.setAngle(-12.5)
        model.flipHorizontally()
        model.setResizeMode(.pixels)
        model.setResizeWidth(800)
        let sections = model.projectSections
        let section = try #require(sections["geometry"])
        #expect(section == geometrySection(model.geometry))
        // The Windows app ignores a section it does not know; the sections it knows keep their keys.
        #expect(sections["wavelet"] != nil && sections["colour"] != nil)

        let defaults = try #require(UserDefaults(suiteName: "GeometryModelTests-\(UUID().uuidString)"))
        let other = AppModel(backend: FakeBackend(), defaults: defaults)
        other.applyProjectSections(sections)
        #expect(other.geometry == model.geometry)
        // A project without the section (an older one, or one from Windows) leaves the values as they are.
        other.applyProjectSections(sections.filter { $0.key != "geometry" })
        #expect(other.geometry == model.geometry)
    }
}
