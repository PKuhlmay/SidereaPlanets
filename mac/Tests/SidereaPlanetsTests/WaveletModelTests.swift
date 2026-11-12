import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The step Wavelets in the app model, on the fake core: when the preview is asked for and what it shows, Process, Do All with
/// Colour and LRGB, Cancel and Pause, Before, the schemes, the Functions cards, Contrast/Brightness and Copy To, and what happens to all
/// of it when the stack goes.
@MainActor
@Suite struct WaveletModelTests {
    let backend = FakeBackend()
    let defaults: UserDefaults
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")

    init() throws {
        defaults = try #require(UserDefaults(suiteName: "WaveletModelTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
    }

    private func eventually(_ condition: () async -> Bool) async -> Bool {
        for _ in 0..<400 {
            if await condition() { return true }
            try? await Task.sleep(for: .milliseconds(5))
        }
        return false
    }

    /// A recording that went through all steps up to Stack; the Wavelets step is open and its first preview is in the image. The
    /// Processing Area is 2 x 2 in the middle of the 6 x 4 stack of the fake, so the preview is easy to find in the pixels.
    private func stackedRecording(area: Int = 2) async {
        model.wavelet.processingAreaSide = area
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        await model.settleWaveletPreview()
        #expect(model.progress.step == .stacked)
    }

    /// The red value of one pixel of the 6 x 4 pictures of the fake.
    private func pixel(_ image: RgbaImage?, _ x: Int, _ y: Int) -> UInt8? {
        image.map { $0.rgba[(y * Int($0.width) + x) * 4] }
    }

    // MARK: Entering the step

    @Test func theStepOpensAfterStackAndShowsAPreviewOfTheProcessingArea() async {
        await stackedRecording()
        #expect(model.viewStep == .wavelets)
        #expect(model.stepStates[.wavelets]?.status == .possible)

        let previews = backend.wavelets.previews
        #expect(previews.count == 1)
        #expect(previews[0].area == Region(x: 2, y: 1, width: 2, height: 2))
        #expect(previews[0].mode == .colour)
        #expect(previews[0].settings == WaveletValues.defaults.core)

        // The preview is written into a copy of the stack: 200 in the area, 100 around it.
        let processed = model.waveletData.processed?.pixels
        #expect(pixel(processed, 2, 1) == 200 && pixel(processed, 3, 2) == 200)
        #expect(pixel(processed, 0, 0) == 100 && pixel(processed, 5, 3) == 100 && pixel(processed, 1, 1) == 100)
        #expect(model.shownImage === model.waveletData.processed?.image)
        #expect(model.imageCaption == "Sharpened stack")
        // The first preview leaves the size of the stack in the status bar; the next one tells how long it took.
        #expect(model.status.action == "Stacked 3 frames · 1024×680" && model.status.result == "Value range 0–255")
        model.moveLevel(0, to: 12)
        #expect(model.status.action == "Updating preview…")
        await model.settleWaveletPreview()
        #expect(model.status.result == "Preview updated")
        #expect(model.status.action.hasPrefix("Preview calculated in ") && model.status.action.hasSuffix(" ms"))
    }

    @Test func theStackIsShownUntilThePixelsAndThePreviewAreThere() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        #expect(model.waveletData.processed == nil && model.waveletData.stack == nil)
        await model.runStack()
        #expect(model.shownImage != nil)
        await model.settleWaveletPreview()
        #expect(model.waveletData.stack != nil && model.waveletData.processed != nil)
    }

    @Test func theStepIsOnlyOpenWithAStack() async {
        await model.open([jupiter])
        await model.setAlignpoints()
        #expect(model.stepStates[.wavelets]?.status == .locked)
        model.show(.wavelets)
        #expect(model.viewStep != .wavelets)
        await model.runProcess()
        #expect(backend.wavelets.previews.isEmpty)
        #expect(await model.runDoAll() == false)
    }

    // MARK: Live preview

    @Test func aMovedSliderAsksForANewPreviewWithTheNewFactor() async {
        await stackedRecording()
        model.moveLevel(0, to: 32)
        await model.settleWaveletPreview()
        let previews = backend.wavelets.previews
        #expect(previews.count == 2)
        #expect(previews[1].settings?.layers[0].gain == 3.2 && previews[1].settings?.layers[1].gain == 1)
        #expect(model.status.result == "Preview updated")
    }

    @Test func changesWhileAPreviewRunsAreFoldedIntoOneMoreRequest() async {
        await stackedRecording()
        backend.wavelets.delayOfPreview = .milliseconds(60)
        model.moveLevel(0, to: 11)
        #expect(await eventually { backend.wavelets.previews.count == 2 })
        // The request for 1.1 is under way; the others come meanwhile.
        for level in 12...15 { model.moveLevel(0, to: level) }
        await model.settleWaveletPreview()
        let previews = backend.wavelets.previews
        #expect(previews.count == 3)
        #expect(previews[1].settings?.layers[0].gain == 1.1)
        #expect(previews[2].settings?.layers[0].gain == 1.5)
    }

    @Test func linkedWaveletsTakeAllSlidersAlong() async {
        await stackedRecording()
        model.wavelet.linked = true
        model.moveLevel(1, to: 30)
        #expect(model.wavelet.values.layers.map(\.level) == [30, 30, 30, 30, 30, 30])
        model.wavelet.linked = false
        model.moveLevel(0, to: 12)
        #expect(model.wavelet.values.layers.map(\.level) == [12, 30, 30, 30, 30, 30])
        await model.settleWaveletPreview()
    }

    @Test func resetWaveletsPutsTheLayersBack() async {
        await stackedRecording()
        model.wavelet.values.layers[2].denoise = 3
        model.wavelet.values.layers[2].enabled = false
        model.moveLevel(4, to: 99)
        model.resetWavelets()
        #expect(model.wavelet.values.layers == WaveletValues.defaults.layers)
        await model.settleWaveletPreview()
    }

    @Test func theColourModeAndTheProcessingAreaStartAPreviewToo() async {
        await stackedRecording()
        model.wavelet.lrgb = true
        await model.settleWaveletPreview()
        #expect(backend.wavelets.previews.last?.mode == .lrgb)
        model.wavelet.colour = false
        await model.settleWaveletPreview()
        #expect(backend.wavelets.previews.last?.mode == .gray)
        model.wavelet.processingAreaSide = 4
        await model.settleWaveletPreview()
        #expect(backend.wavelets.previews.last?.area == Region(x: 1, y: 0, width: 4, height: 4))
        // A change that does not decide the result asks for nothing.
        let count = backend.wavelets.previews.count
        model.wavelet.showProcessingArea = true
        model.wavelet.hold = true
        model.wavelet.functionsOpen = false
        await model.settleWaveletPreview()
        #expect(backend.wavelets.previews.count == count)
    }

    @Test func noPreviewOutsideTheWaveletsStep() async {
        await stackedRecording()
        let count = backend.wavelets.previews.count
        model.show(.stack)
        model.moveLevel(0, to: 50)
        await model.settleWaveletPreview()
        #expect(backend.wavelets.previews.count == count)
        // Back in the step the preview follows the values.
        model.show(.wavelets)
        await model.settleWaveletPreview()
        #expect(backend.wavelets.previews.count == count + 1)
        #expect(backend.wavelets.previews.last?.settings?.layers[0].gain == 5)
    }

    // MARK: Process

    @Test func processCalculatesThePreviewNow() async {
        await stackedRecording()
        await model.runProcess()
        #expect(backend.wavelets.previews.count == 2)
        #expect(model.status.result == "Preview updated")
        #expect(model.run == nil && !model.isBusy && model.status.progress == 0)
    }

    // MARK: Do All

    @Test func doAllSendsTheValuesAndModeShowsTheResultAndOpensSave() async {
        await stackedRecording()
        model.moveLevel(1, to: 22)
        model.wavelet.lrgb = true
        await model.settleWaveletPreview()
        await model.doAll()

        #expect(backend.wavelets.doAlls.count == 1)
        #expect(backend.wavelets.doAlls[0].mode == .lrgb)
        #expect(backend.wavelets.doAlls[0].settings == model.wavelet.values.core)
        #expect(backend.wavelets.doAlls[0].settings.layers[1].gain == 2.2)
        #expect(model.status.result == "Do All finished")
        #expect(model.progress.waveletsDone && model.stepStates[.wavelets]?.status == .done)
        #expect(model.viewStep == .save)
        #expect(model.result(of: .wavelets) == "Sharpening applied")
        // The whole image is the result: 50 everywhere, also where the preview had been.
        #expect(pixel(model.waveletData.processed?.pixels, 0, 0) == 50 && pixel(model.waveletData.processed?.pixels, 2, 1) == 50)
        #expect(model.shownImage === model.waveletData.processed?.image)
        #expect(model.imageCaption == "Sharpened stack")
        #expect(model.run == nil && model.status.progress == 0)
    }

    @Test func colourOffGivesGrayAndLrgbWithoutColourDoesNotCount() async {
        await stackedRecording()
        model.wavelet.colour = false
        model.wavelet.lrgb = true
        await model.runDoAll()
        #expect(backend.wavelets.doAlls.last?.mode == .gray)
        model.wavelet.colour = true
        model.wavelet.lrgb = false
        await model.runDoAll()
        #expect(backend.wavelets.doAlls.last?.mode == .colour)
    }

    @Test func theResultOnlyCountsWhileItMatchesTheValuesOnScreen() async {
        await stackedRecording()
        await model.doAll()
        #expect(model.progress.waveletsDone)
        model.progress.saved = true

        model.moveLevel(0, to: 20)
        #expect(!model.progress.waveletsDone && model.stepStates[.wavelets]?.status == .possible)
        // A changed result takes the check mark of Save away for good.
        #expect(!model.progress.saved)
        model.moveLevel(0, to: 10)
        #expect(model.progress.waveletsDone)
        #expect(!model.progress.saved)
        model.wavelet.lrgb = true
        #expect(!model.progress.waveletsDone)
        model.wavelet.lrgb = false
        #expect(model.progress.waveletsDone)
        await model.settleWaveletPreview()
    }

    @Test func enteringTheStepWithAFinishedDoAllKeepsItsImage() async {
        await stackedRecording()
        await model.doAll()
        let count = backend.wavelets.previews.count
        model.show(.wavelets)
        await model.settleWaveletPreview()
        // The preview must not paint over the result.
        #expect(backend.wavelets.previews.count == count)
        #expect(pixel(model.waveletData.processed?.pixels, 2, 1) == 50)
    }

    @Test func doAllShowsProgressAndLocksTheWindowUntilItEnds() async {
        await stackedRecording()
        backend.wavelets.holdsDoAll = true
        let running = Task { await model.doAll() }
        #expect(await eventually { model.status.progress == 0.5 })
        #expect(model.run?.name == "Do All" && model.run?.operation == .wavelet && model.isBusy && model.canCancel)
        #expect(model.status.action == "Applying to whole image…")
        #expect(model.windowTitle == "Siderea Planets · Do All 50%")
        model.show(.stack)
        #expect(model.viewStep == .wavelets)

        backend.wavelets.releaseDoAll()
        await running.value
        #expect(model.status.result == "Do All finished" && model.run == nil && model.status.progress == 0)
        #expect(model.viewStep == .save)
    }

    @Test func cancelStopsDoAllAndLeavesTheEarlierResult() async {
        await stackedRecording()
        let before = model.waveletData.processed?.image
        backend.wavelets.holdsDoAll = true
        let running = Task { await model.doAll() }
        #expect(await eventually { model.canCancel && model.status.progress > 0 })
        model.cancelRun()
        await running.value

        #expect(backend.log.cancelledOperations == [.wavelet])
        #expect(model.status.result == "Cancelled")
        #expect(model.run == nil && !model.isBusy && model.status.progress == 0)
        #expect(!model.progress.waveletsDone && model.viewStep == .wavelets)
        #expect(model.waveletData.processed?.image === before)
    }

    @Test func pauseAndResumeReachTheCore() async {
        await stackedRecording()
        backend.wavelets.holdsDoAll = true
        let running = Task { await model.doAll() }
        #expect(await eventually { model.canCancel })
        model.togglePause()
        #expect(model.run?.paused == true)
        model.togglePause()
        #expect(backend.log.pauseCalls == [true, false])
        backend.wavelets.releaseDoAll()
        await running.value
    }

    @Test func automaticAppliesTheValuesToTheWholeImageOnceTheyStopChanging() async {
        await stackedRecording()
        model.wavelet.automatic = true
        model.moveLevel(0, to: 20)
        model.moveLevel(0, to: 25)
        #expect(await eventually { !backend.wavelets.doAlls.isEmpty })
        await model.settleWaveletPreview()
        #expect(backend.wavelets.doAlls.count == 1)
        #expect(backend.wavelets.doAlls[0].settings.layers[0].gain == 2.5)
        // Unlike the button it does not move on to Save.
        #expect(model.viewStep == .wavelets && model.progress.waveletsDone)
    }

    // MARK: Before

    @Test func beforeShowsTheStackWithoutAnythingAppliedAndEndsWithTheStep() async {
        await stackedRecording()
        #expect(model.shownImage === model.waveletData.processed?.image)
        model.waveletData.before = true
        #expect(model.shownImage === model.waveletData.stack?.image)
        #expect(!model.showsSharpened && model.imageCaption == "Stack of 3 frames")
        model.show(.stack)
        #expect(!model.waveletData.before)
    }

    // MARK: Hold

    @Test func aNewStackStartsTheLayersOverUnlessHoldIsOn() async {
        await stackedRecording()
        model.moveLevel(0, to: 40)
        model.wavelet.values.scheme = .dyadic
        model.show(.stack)
        await model.runStack()
        #expect(model.wavelet.values.layers == WaveletValues.defaults.layers)
        // Only the layers start over; the structure stays.
        #expect(model.wavelet.values.scheme == .dyadic)

        model.wavelet.hold = true
        model.moveLevel(0, to: 40)
        model.show(.stack)
        await model.runStack()
        #expect(model.wavelet.values.layers[0].level == 40)

        // Open project keeps the values of its file for the first stack, Hold or not.
        model.wavelet.hold = false
        model.keepWaveletValuesForNextStack()
        model.show(.stack)
        await model.runStack()
        #expect(model.wavelet.values.layers[0].level == 40)
        model.show(.stack)
        await model.runStack()
        #expect(model.wavelet.values.layers[0].level == 10)
        await model.settleWaveletPreview()
    }

    @Test func aNewStackDropsTheOldResult() async {
        await stackedRecording()
        await model.doAll()
        #expect(model.progress.waveletsDone)
        model.show(.stack)
        await model.runStack()
        #expect(!model.progress.waveletsDone && model.stepStates[.wavelets]?.status == .possible)
        await model.settleWaveletPreview()
        #expect(pixel(model.waveletData.processed?.pixels, 2, 1) == 200)
    }

    @Test func aligningAgainTakesTheStackAndWhatWasMadeOfItButKeepsTheClipboard() async {
        await stackedRecording()
        model.copyTo()
        #expect(model.waveletData.clipboard != nil)
        await model.runAlign()
        #expect(model.stacked == nil && model.progress.step == .aligned)
        #expect(model.waveletData.stack == nil && model.waveletData.processed == nil && model.waveletData.adjusted == nil)
        #expect(model.waveletData.clipboard != nil)
        #expect(model.progress.waveletsDone == false)
    }

    // MARK: Schemes

    private func temporaryScheme() -> URL {
        FileManager.default.temporaryDirectory.appendingPathComponent("scheme-\(UUID().uuidString).rwv")
    }

    @Test func aSchemeSavedAndLoadedGivesTheSameValues() async throws {
        await stackedRecording()
        model.wavelet.values.scheme = .dyadic
        model.wavelet.values.filter = .default
        model.wavelet.values.initialLayer = 2
        model.wavelet.values.stepIncrement = 1
        model.wavelet.values.layers[2].level = 55
        model.wavelet.values.layers[2].denoise = 1.5
        model.wavelet.values.layers[4].enabled = false
        let saved = model.wavelet.values.core
        let url = temporaryScheme()
        defer { try? FileManager.default.removeItem(at: url) }

        model.saveScheme(url)
        #expect(model.status.result == "Scheme saved: \(url.lastPathComponent)")
        #expect(FileManager.default.fileExists(atPath: url.path))

        model.wavelet.values = WaveletValues.defaults
        await model.loadScheme(url)
        #expect(model.wavelet.values.core == saved)
        #expect(model.wavelet.values.layers[2].level == 55 && model.wavelet.values.scheme == .dyadic)
        // The preview of the new values has run, and its message did not replace this one.
        #expect(backend.wavelets.previews.last?.settings == saved)
        #expect(model.status.result == "Scheme loaded: \(url.lastPathComponent)")
    }

    @Test func aBrokenSchemeReportsTheErrorAndChangesNothing() async throws {
        await stackedRecording()
        let url = temporaryScheme()
        defer { try? FileManager.default.removeItem(at: url) }
        try "this is not a scheme".write(to: url, atomically: true, encoding: .utf8)
        let before = model.wavelet.values
        await model.loadScheme(url)
        #expect(model.status.result.hasPrefix("Load Scheme failed: "))
        #expect(model.wavelet.values == before)

        await model.loadScheme(url.appendingPathExtension("missing"))
        #expect(model.status.result.hasPrefix("Load Scheme failed: "))
    }

    // MARK: Functions

    @Test func aFunctionsButtonOpensItsCardAndASecondClickClosesIt() async {
        await stackedRecording()
        model.toggleFunction("Gamma")
        model.toggleFunction("Histogram")
        #expect(model.wavelet.functions == ["Gamma", "Histogram"] && model.openCards == ["Gamma", "Histogram"])
        model.toggleFunction("Gamma")
        #expect(model.openCards == ["Histogram"])
        model.closeFunction("Histogram")
        #expect(model.openCards.isEmpty)
        // The values stay with a closed card.
        model.tone.gamma = 1.3
        model.toggleFunction("Gamma")
        model.closeFunction("Gamma")
        #expect(model.tone.gamma == 1.3)
        await model.settleColours()
    }

    @Test func aButtonWithoutACardSaysSoInTheStatusBar() async {
        await stackedRecording()
        model.toggleFunction("Show Line Graph")
        #expect(model.status.result == "Show Line Graph: not available yet")
        #expect(model.wavelet.functions.isEmpty)
    }

    @Test func theCardsAreOnlyShownInTheWaveletsStep() async {
        await stackedRecording()
        model.toggleFunction("RGB Balance")
        #expect(model.openCards == ["RGB Balance"])
        model.show(.stack)
        #expect(model.openCards.isEmpty)
    }

    // MARK: Colour and tone

    @Test func aCardValueIsAppliedToTheBaseAndShownInTheImage() async {
        await stackedRecording()
        #expect(model.waveletData.adjusted == nil)
        model.tone.gamma = 1.4
        await model.settleColours()

        #expect(backend.wavelets.appliedColours.count == 1)
        let sent = backend.wavelets.appliedColours[0]
        #expect(sent.gamma == 1.4 && sent.contrast == 100 && sent.brightness == 0 && sent.black == 0 && sent.white == 255)
        // The base goes over once: the stack with the preview in it.
        #expect(backend.wavelets.colourSources.count == 1)
        #expect(pixel(backend.wavelets.colourSources[0], 2, 1) == 200 && pixel(backend.wavelets.colourSources[0], 0, 0) == 100)
        #expect(pixel(model.waveletData.adjusted?.pixels, 0, 0) == 7)
        #expect(model.shownImage === model.waveletData.adjusted?.image)

        // Contrast and Brightness count too; the Contrast field shows the slider divided by 2.5.
        model.tone.contrast = 300
        model.tone.brightness = -20
        await model.settleColours()
        let last = backend.wavelets.appliedColours.last
        #expect(last?.contrast == 120 && last?.brightness == -20 && last?.gamma == 1.4)
        #expect(backend.wavelets.colourSources.count == 1)

        // Before shows the base without them.
        model.waveletData.before = true
        #expect(model.shownImage === model.waveletData.stack?.image)
        model.waveletData.before = false

        // Everything neutral again: the base is shown without a round trip.
        let calls = backend.wavelets.appliedColours.count
        model.tone = ToneValues()
        await model.settleColours()
        #expect(model.waveletData.adjusted == nil && model.shownImage === model.waveletData.processed?.image)
        #expect(backend.wavelets.appliedColours.count == calls)
    }

    @Test func theCardsApplyToTheNewBaseAfterAPreview() async {
        await stackedRecording()
        model.tone.gamma = 1.4
        await model.settleColours()
        model.moveLevel(0, to: 20)
        await model.settleWaveletPreview()
        await model.settleColours()
        #expect(backend.wavelets.colourSources.count == 2)
        #expect(backend.wavelets.appliedColours.count == 2)
    }

    @Test func theCardsApplyToTheResultOfDoAll() async {
        await stackedRecording()
        await model.doAll()
        model.tone.red = 120
        await model.settleColours()
        #expect(pixel(backend.wavelets.colourSources.last, 0, 0) == 50)
        #expect(backend.wavelets.appliedColours.last?.red == 120)
        #expect(model.shownImage === model.waveletData.adjusted?.image)
    }

    @Test func aChangeThatLeavesTheValuesAloneAppliesNothing() async {
        await stackedRecording()
        model.tone.holdContrast = true
        model.tone.clip = 2
        model.tone.maxShift = 9
        model.tone.histogramView = .result
        await model.settleColours()
        #expect(backend.wavelets.appliedColours.isEmpty)
    }

    @Test func resetContrastPutsBackOneHundredAndZero() async {
        await stackedRecording()
        model.tone.contrast = 500
        model.tone.brightness = 40
        model.tone.gamma = 2
        model.resetContrast()
        #expect(model.tone.contrast == 250 && model.tone.brightness == 0 && model.tone.gamma == 2)
        await model.settleColours()
    }

    @Test func autoAlignPutsTheShiftsIntoTheFields() async {
        await stackedRecording()
        await model.autoAlign()
        #expect(model.tone.shifts.red == ChannelShift(x: 1.5, y: -0.5) && model.tone.shifts.blue == ChannelShift(x: -2, y: 1))
        #expect(model.status.result == "RGB Align: shifts found")
        await model.settleColours()
        #expect(backend.wavelets.appliedColours.last?.shifts.blue.x == -2)
    }

    @Test func autoAlignWithoutAnImageSaysSo() async {
        await model.autoAlign()
        #expect(model.status.result == "RGB Align: no image yet")
        await model.autoLevels()
        #expect(model.status.result == "Histogram: no image yet")
    }

    @Test func autoLevelsRoundsTheBlackAndWhitePoint() async {
        await stackedRecording()
        await model.autoLevels()
        #expect(model.tone.black == 12 && model.tone.white == 231)
        await model.settleColours()
    }

    @Test func theHistogramAsksForTheInputOrTheResult() async {
        await stackedRecording()
        let input = await model.histogram(after: false)
        let result = await model.histogram(after: true)
        #expect(input?.bins == 3 && result?.luma == [1, 4, 1])
        let asked = backend.wavelets.histogramRequests
        #expect(asked.map(\.after) == [false, true])
    }

    @Test func resetToneBringsEveryCardBackButKeepsHold() async {
        await stackedRecording()
        model.tone.gamma = 2
        model.tone.red = 50
        model.tone.black = 20
        model.tone.holdContrast = true
        model.resetTone()
        #expect(isNeutral(model.tone.core) && model.tone.holdContrast)
        await model.settleColours()
    }

    // MARK: Copy To

    @Test func copyToKeepsTheImageOnScreenAndToggleSwitchesBetweenBoth() async {
        await stackedRecording()
        #expect(model.waveletData.clipboard == nil)
        model.copyTo()
        #expect(model.status.result == "Image remembered")
        let copy = model.waveletData.clipboard?.image
        #expect(copy != nil && copy === model.waveletData.processed?.image)

        model.tone.gamma = 1.4
        await model.settleColours()
        #expect(model.shownImage === model.waveletData.adjusted?.image)
        model.showToneView(.clipboard)
        #expect(model.shownImage === copy)
        model.showToneView(.current)
        #expect(model.shownImage === model.waveletData.adjusted?.image)
    }

    @Test func theClipboardImageCannotBeChosenBeforeACopy() async {
        await stackedRecording()
        model.showToneView(.clipboard)
        #expect(model.waveletData.toneView == .current)
        model.loadTo()
        #expect(model.status.result == "Restore: nothing remembered yet")
        model.showDifference()
        #expect(model.status.result == "Difference: nothing remembered yet")
        #expect(model.waveletData.toneView == .current)
    }

    @Test func differenceShowsTheAmplifiedAmountAndFollowsTheCurrentImage() async {
        await stackedRecording()
        model.copyTo()
        model.tone.gamma = 1.4
        await model.settleColours()
        model.showDifference()
        #expect(model.status.result == "Difference shown" && model.waveletData.toneView == .difference)
        // |7 - 100| * 4 stops at 255.
        #expect(pixel(model.waveletData.difference?.pixels, 0, 0) == 255)
        #expect(model.shownImage === model.waveletData.difference?.image)

        // Back to unchanged: the current image is the copy again, so nothing differs.
        model.tone = ToneValues()
        await model.settleColours()
        #expect(model.waveletData.toneView == .difference)
        #expect(pixel(model.waveletData.difference?.pixels, 0, 0) == 0)
    }

    @Test func loadToMakesTheCopyTheProcessedImageAndResetsTheCards() async {
        await stackedRecording()
        model.copyTo()
        let copy = model.waveletData.clipboard?.image
        model.moveLevel(0, to: 20)
        model.tone.gamma = 1.4
        await model.settleWaveletPreview()
        await model.settleColours()

        model.loadTo()
        #expect(model.status.result == "Remembered image restored")
        #expect(model.waveletData.processed?.image === copy)
        #expect(isNeutral(model.tone.core) && model.waveletData.toneView == .current)
        await model.settleColours()
        #expect(model.waveletData.adjusted == nil && model.shownImage === copy)
    }

    @Test func copyToWithoutAnImageSaysSo() {
        model.copyTo()
        #expect(model.status.result == "Remember: no image yet")
    }

    // MARK: Denoise/Deringing and Masking (MAC-018)

    @Test func denoiseAndDeringingValuesGoToTheCoreWithTheCards() async {
        await stackedRecording()
        model.toggleFunction("Denoise/Deringing")
        #expect(model.openCards == ["Denoise/Deringing"])
        model.tone.refine.denoise = 40
        model.tone.refine.deringing = 80
        model.tone.refine.radius = 6
        await model.settleColours()

        let sent = backend.wavelets.appliedColours.last
        #expect(sent?.refine.denoise == 40 && sent?.refine.deringing == 80 && sent?.refine.deringingRadius == 6)
        #expect(sent?.refine.mask.enabled == false)
        #expect(pixel(model.waveletData.adjusted?.pixels, 0, 0) == FakeBackend.colourValue)
    }

    @Test func theMaskValuesGoToTheCoreOnlyWhenTheMaskIsOn() async {
        await stackedRecording()
        model.tone.refine.threshold = 60
        model.tone.refine.feather = 9
        model.tone.refine.invert = true
        await model.settleColours()
        #expect(backend.wavelets.appliedColours.isEmpty, "a mask that is off changes nothing, so there is nothing to apply")
        model.tone.refine.maskOn = true
        await model.settleColours()
        let mask = backend.wavelets.appliedColours.last?.refine.mask
        #expect(mask == MaskSettings(enabled: true, threshold: 60, feather: 9, invert: true))
    }

    @Test func showMaskAsksForThePreviewAndShowsItInsteadOfTheImage() async {
        await stackedRecording()
        model.toggleFunction("Masking")
        model.tone.refine.threshold = 50
        model.tone.refine.showMask = true
        await model.settleColours()

        #expect(backend.wavelets.maskOverlayRequests.last?.mask.threshold == 50)
        #expect(pixel(model.waveletData.maskOverlay?.pixels, 0, 0) == FakeBackend.maskValue)
        #expect(model.shownImage === model.waveletData.maskOverlay?.image)

        // It follows the values.
        model.tone.refine.threshold = 70
        await model.settleColours()
        #expect(backend.wavelets.maskOverlayRequests.last?.mask.threshold == 70)

        // Before shows the stack, and closing the card brings the image back.
        model.waveletData.before = true
        #expect(model.shownImage === model.waveletData.stack?.image)
        model.waveletData.before = false
        model.closeFunction("Masking")
        await model.settleColours()
        #expect(model.waveletData.maskOverlay == nil)
        #expect(model.shownImage === model.waveletData.processed?.image)
    }

    @Test func showMaskDoesNothingWhileTheCardIsClosed() async {
        await stackedRecording()
        model.tone.refine.showMask = true
        await model.settleColours()
        #expect(backend.wavelets.maskOverlayRequests.isEmpty && model.waveletData.maskOverlay == nil)
        model.toggleFunction("Masking")
        await model.settleColours()
        #expect(backend.wavelets.maskOverlayRequests.count == 1)
    }

    @Test func showMaskIsNotPartOfTheResult() async {
        await stackedRecording()
        model.tone.refine.showMask = true
        #expect(isNeutral(model.tone.core))
        model.tone.refine.denoise = 10
        #expect(model.saveAdjustments?.refine.denoise == 10)
        model.tone.refine.denoise = 0
        #expect(model.saveAdjustments == nil)
        await model.settleColours()
    }

    @Test func theCardsTravelInTheirOwnSectionOfTheProjectFile() async {
        await stackedRecording()
        model.tone.refine = RefineValues(core: RefineSettings(
            denoise: 30, deringing: 60, deringingRadius: 8,
            mask: MaskSettings(enabled: true, threshold: 90, feather: 20, invert: true)
        ))
        let sections = model.projectSections
        // The layout of the Windows sections stays as it is: the cards are an extra section they ignore.
        guard case .object(let wavelet)? = sections["wavelet"], case .object(let tone)? = wavelet["tone"] else {
            Issue.record("no wavelet section")
            return
        }
        #expect(wavelet["refine"] == nil && tone["refine"] == nil)
        #expect(sections["refine"] == refineSection(model.tone.refine))

        model.tone.refine = RefineValues()
        model.applyProjectSections(sections)
        #expect(model.tone.refine.denoise == 30 && model.tone.refine.maskOn && model.tone.refine.threshold == 90)
        // A project from the Windows app has no such section: the values stay.
        var windows = sections
        windows["refine"] = nil
        model.tone.refine.denoise = 5
        model.applyProjectSections(windows)
        #expect(model.tone.refine.denoise == 5)
        await model.settleColours()
    }

    @Test func resetToneAlsoBringsTheNewCardsBack() async {
        await stackedRecording()
        model.tone.refine = RefineValues(core: RefineSettings(
            denoise: 30, deringing: 60, deringingRadius: 8,
            mask: MaskSettings(enabled: true, threshold: 90, feather: 20, invert: true)
        ))
        model.resetTone()
        #expect(model.tone.refine == RefineValues.defaults && isNeutral(model.tone.core))
        await model.settleColours()
    }
}
