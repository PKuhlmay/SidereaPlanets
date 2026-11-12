import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The pure parts of the Wavelets step: defaults, what is sent to the core, linked sliders, the Processing Area, the tone values and
/// the picture helpers. Same cases as app/src/logic/wavelet.test.ts and colour.test.ts.
@Suite struct WaveletLogicTests {
    // MARK: Values

    @Test func defaultsAreThoseOfTheWindowsApp() {
        let values = WaveletValues.defaults
        #expect(values.scheme == .linear && values.filter == .gaussian)
        #expect(values.initialLayer == 1 && values.stepIncrement == 0)
        #expect(values.layers.count == 6)
        for layer in values.layers {
            #expect(layer.enabled && layer.denoise == 0 && layer.level == 10)
            #expect(formatPreview(layer.level) == "1.0")
            #expect(abs(layer.sharpen - 0.1) < 1e-6)
        }
        let options = WaveletOptions()
        #expect(options.colour && !options.lrgb && !options.automatic && !options.hold && !options.linked)
        #expect(options.functionsOpen && options.functions.isEmpty && options.processingAreaSide == 512)
    }

    @Test func theCoreGetsTheSliderPositionAsAFactor() {
        var values = WaveletValues.defaults
        values.layers[0].level = 32
        values.layers[1].level = -50
        values.layers[2].level = 1000
        values.layers[3].enabled = false
        values.layers[4].denoise = 1.25
        values.initialLayer = 3
        values.stepIncrement = 2
        values.scheme = .dyadic
        values.filter = .default
        let core = values.core
        #expect(core.layers.map(\.gain) == [3.2, -5, 100, 1, 1, 1])
        #expect(core.layers[3].enabled == false && core.layers[4].denoise == 1.25)
        #expect(core.initial == 3 && core.step == 2 && core.scheme == .dyadic && core.filter == .default)
        // The values survive the way there and back.
        #expect(WaveletValues(core: core) == values)
    }

    @Test func wholeNumbersAreHeldToWhatTheCoreTakes() {
        var values = WaveletValues.defaults
        values.initialLayer = 0
        values.stepIncrement = -4
        #expect(values.core.initial == 1 && values.core.step == 0)
    }

    @Test func colourAndLrgbChooseTheMode() {
        #expect(colourMode(colour: true, lrgb: false) == .colour)
        #expect(colourMode(colour: true, lrgb: true) == .lrgb)
        #expect(colourMode(colour: false, lrgb: false) == .gray)
        // LRGB only counts together with Colour.
        #expect(colourMode(colour: false, lrgb: true) == .gray)
    }

    @Test func aDoAllKeyDependsOnStackValuesAndModeButNotOnTheArea() {
        var options = WaveletOptions()
        let key = DoAllKey(stack: 1, settings: options.values.core, mode: options.mode)
        options.processingAreaSide = 128
        options.showProcessingArea = true
        #expect(key == DoAllKey(stack: 1, settings: options.values.core, mode: options.mode))
        #expect(key != DoAllKey(stack: 2, settings: options.values.core, mode: options.mode))
        options.values.layers[5].sharpen = 0.5
        #expect(key != DoAllKey(stack: 1, settings: options.values.core, mode: options.mode))
        options = WaveletOptions()
        options.lrgb = true
        #expect(key != DoAllKey(stack: 1, settings: options.values.core, mode: options.mode))
    }

    // MARK: Sliders

    @Test func aSliderMovesAloneWithoutLinkedWavelets() {
        #expect(linkedLevels([10, 10, 10], moving: 1, to: 30, linked: false) == [10, 30, 10])
        // Held to the range of the slider.
        #expect(linkedLevels([10, 10, 10], moving: 0, to: 5000, linked: false) == [1000, 10, 10])
        #expect(linkedLevels([10, 10, 10], moving: 0, to: -90, linked: false) == [-50, 10, 10])
    }

    @Test func linkedWaveletsMoveAllSlidersByTheSameAmount() {
        #expect(linkedLevels([10, 20, 30], moving: 0, to: 15, linked: true) == [15, 25, 35])
        // The others stop at the ends of their range.
        #expect(linkedLevels([10, 990, -40], moving: 0, to: 30, linked: true) == [30, 1000, -20])
        #expect(linkedLevels([10, 20, -45], moving: 0, to: -10, linked: true) == [-10, 0, -50])
    }

    @Test func thePreviewButtonShowsOneDecimal() {
        #expect(formatPreview(10) == "1.0" && formatPreview(32) == "3.2" && formatPreview(-50) == "-5.0" && formatPreview(1000) == "100.0")
    }

    // MARK: Processing Area

    @Test func theProcessingAreaSitsInTheMiddleAndStaysInsideTheImage() {
        #expect(processingArea(imageWidth: 1024, imageHeight: 680, side: 512) == Region(x: 256, y: 84, width: 512, height: 512))
        // A smaller image gives a smaller area.
        #expect(processingArea(imageWidth: 300, imageHeight: 200, side: 512) == Region(x: 0, y: 0, width: 300, height: 200))
        #expect(processingArea(imageWidth: 6, imageHeight: 4, side: 2) == Region(x: 2, y: 1, width: 2, height: 2))
    }

    @Test func aPreviewIsWrittenRowByRowIntoTheWholeImage() {
        var pixels = flatImage(1, width: 4, height: 3).rgba
        let patch = flatImage(9, width: 2, height: 2)
        let preview = WaveletPreview(region: Region(x: 1, y: 1, width: 2, height: 2), rgba: patch.rgba)
        writePreview(preview, into: &pixels, imageWidth: 4)
        func value(_ x: Int, _ y: Int) -> UInt8 { pixels[(y * 4 + x) * 4] }
        #expect(value(1, 1) == 9 && value(2, 1) == 9 && value(1, 2) == 9 && value(2, 2) == 9)
        #expect(value(0, 1) == 1 && value(3, 1) == 1 && value(1, 0) == 1 && value(1, 2 + 0) == 9)
        #expect(pixels.count == 4 * 3 * 4)
    }

    // MARK: Tone

    @Test func theToneDefaultsLeaveTheImageAsItIs() {
        let tone = ToneValues()
        #expect(tone.contrast == 250 && contrastField(tone.contrast) == 100 && tone.brightness == 0)
        #expect(tone.red == 100 && tone.green == 100 && tone.blue == 100 && tone.black == 0 && tone.white == 255 && tone.gamma == 1)
        #expect(tone.maxShift == 5 && tone.clip == 0.1 && tone.histogramView == .input)
        #expect(isNeutral(tone.core))
    }

    @Test func anyChangedValueIsNotNeutral() {
        var tone = ToneValues()
        tone.gamma = 1.01
        #expect(!isNeutral(tone.core))
        tone = ToneValues()
        tone.shifts.red.x = 0.1
        #expect(!isNeutral(tone.core))
        tone = ToneValues()
        // The core gets the field value (slider / 2.5), so one step of the slider can be lost to rounding.
        tone.contrast = 251
        #expect(isNeutral(tone.core))
        tone.contrast = 253
        #expect(!isNeutral(tone.core))
        tone = ToneValues()
        tone.white = 254
        #expect(!isNeutral(tone.core))
        tone = ToneValues()
        tone.blue = 99
        #expect(!isNeutral(tone.core))
        tone = ToneValues()
        tone.brightness = -1
        #expect(!isNeutral(tone.core))
    }

    @Test func theContrastFieldShowsTheSliderDividedByTwoPointFive() {
        #expect(contrastField(250) == 100 && contrastField(1000) == 400 && contrastField(1) == 0)
        #expect(contrastSlider(100) == 250 && contrastSlider(400) == 1000)
        // Typed values outside the slider are held to it.
        #expect(contrastSlider(0) == 1 && contrastSlider(900) == 1000)
        var tone = ToneValues()
        tone.contrast = 300
        #expect(tone.core.contrast == 120)
    }

    @Test func theCoreGetsTheUnitsOfThePanels() {
        var tone = ToneValues()
        tone.red = 80
        tone.black = 12
        tone.white = 240
        tone.gamma = 1.25
        tone.brightness = -30
        tone.shifts.blue = ChannelShift(x: -1.5, y: 2)
        let core = tone.core
        #expect(core.red == 80 && core.green == 100 && core.black == 12 && core.white == 240 && core.gamma == 1.25)
        #expect(core.brightness == -30 && core.contrast == 100)
        #expect(core.shifts.blue == ChannelShift(x: -1.5, y: 2))
    }

    @Test func theHistogramBeforeTheStretchIgnoresGammaContrastAndBrightness() {
        var tone = ToneValues()
        let before = HistogramKey(tone.core, after: false, base: 1)
        tone.gamma = 2
        tone.contrast = 400
        tone.brightness = 10
        tone.black = 20
        #expect(HistogramKey(tone.core, after: false, base: 1) == before)
        // The result depends on all of them, and on RGB Balance and the shifts either way.
        #expect(HistogramKey(tone.core, after: true, base: 1) != HistogramKey(ToneValues().core, after: true, base: 1))
        tone.red = 90
        #expect(HistogramKey(tone.core, after: false, base: 1) != before)
        #expect(HistogramKey(tone.core, after: false, base: 2) != HistogramKey(tone.core, after: false, base: 1))
    }

    // MARK: Histogram

    @Test func theGraphIsLogarithmicSoABlackSkyDoesNotHideTheRest() {
        #expect(logHeight(0, max: 100) == 0)
        #expect(logHeight(100, max: 100) == 1)
        #expect(logHeight(10, max: 1_000_000) > 0.15)
        #expect(logHeight(5, max: 0) == 0)
        let data = HistogramData(channels: [[1, 9, 3], [2, 2, 40]], luma: [5, 6, 7], bins: 3)
        #expect(histogramMax(data) == 40)
    }

    @Test func theOutlineRunsFromLeftToRightAndHighCountsRiseToTheTop() {
        let points = histogramPoints([0, 10, 0], max: 10, width: 100, height: 50)
        #expect(points.map(\.x) == [0, 50, 100])
        #expect(points[0].y == 50 && points[1].y == 0 && points[2].y == 50)
    }

    @Test func theMarkersStayApartAndInsideTheScale() {
        #expect(xToLevel(0, width: 212) == 0 && xToLevel(212, width: 212) == 255 && xToLevel(-30, width: 212) == 0 && xToLevel(900, width: 212) == 255)
        #expect(levelToX(255, width: 212) == 212 && levelToX(0, width: 212) == 0)
        #expect(moveBlack(300, white: 200) == 199 && moveBlack(-5, white: 200) == 0)
        #expect(moveWhite(-5, black: 10) == 11 && moveWhite(400, black: 10) == 255)
    }

    // MARK: Difference

    @Test func theDifferenceIsTheAmountPerChannelTimesFourAndOpaque() {
        let a = flatImage(100, width: 2, height: 1)
        var other = flatImage(90, width: 2, height: 1)
        other.rgba[0] = 130  // red of the first pixel: |100 - 130| = 30
        let difference = differencePixels(a, other)
        #expect(difference?.rgba[0] == 120 && difference?.rgba[1] == 40 && difference?.rgba[2] == 40 && difference?.rgba[3] == 255)
        #expect(difference?.rgba[4] == 40)
        // Large differences stop at 255.
        #expect(differencePixels(flatImage(0, width: 1, height: 1), flatImage(200, width: 1, height: 1))?.rgba[0] == 255)
        #expect(differencePixels(a, a)?.rgba.prefix(3).allSatisfy { $0 == 0 } == true)
    }

    @Test func imagesOfDifferentSizeHaveNoDifference() {
        #expect(differencePixels(flatImage(1, width: 2, height: 2), flatImage(1, width: 3, height: 2)) == nil)
    }

    // MARK: Functions

    @Test func theFunctionsGridHasFifteenButtonsAndTheOnesThatWorkAreUnlocked() {
        #expect(functionNames.count == 15 && Set(functionNames).count == 15)
        #expect(functionNames.first == "Histogram" && functionNames.last == "Cropping Area")
        #expect(toneCards.allSatisfy { functionNames.contains($0) })
        #expect(isToneCard("Gamma") && isToneCard("RGB Align") && !isToneCard("Masking"))
        // The buttons that do nothing yet are locked: their tooltip says so.
        #expect(functionNames.filter { !isFunctionBuilt($0) }.allSatisfy { functionHelp($0) == "Not available yet" })
        #expect(functionNames.filter(isFunctionBuilt).allSatisfy { functionHelp($0) != "Not available yet" })
    }
}
