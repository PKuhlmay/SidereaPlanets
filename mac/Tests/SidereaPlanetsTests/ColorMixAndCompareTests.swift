import CoreGraphics
import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// Color Mixing (the card values, the presets, what the core is sent, the project file) and View Compare (the divider, the toggle and its
/// link to Before/After), on the fake core.
@MainActor
@Suite struct ColorMixAndCompareTests {
    let backend = FakeBackend()
    let model: AppModel
    let jupiter = URL(fileURLWithPath: "/data/JUPITER.AVI")

    init() throws {
        let defaults = try #require(UserDefaults(suiteName: "ColorMixAndCompareTests-\(UUID().uuidString)"))
        model = AppModel(backend: backend, defaults: defaults)
    }

    private func stackedRecording() async {
        model.wavelet.processingAreaSide = 2
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        await model.settleWaveletPreview()
    }

    // MARK: Color Mixing, values

    @Test func theMixStartsAsTheIdentityAndLeavesTheImageAlone() {
        let mix = ColorMixValues()
        #expect(mix.isIdentity && !mix.preserveBrightness)
        #expect(mix.matrix == [100, 0, 0, 0, 100, 0, 0, 0, 100])
        #expect(mix.core.matrix == [100, 0, 0, 0, 100, 0, 0, 0, 100])
        #expect(isNeutral(ToneValues().core))
        // The buttons of the preset menu: Neutral comes first and is the identity.
        #expect(mixPresets.first?.name == "Neutral" && mixPresets.first?.matrix == ColorMixValues.identityMatrix)
        #expect(mixPresets.allSatisfy { $0.matrix.count == 9 })
    }

    @Test func aWeightIsAddressedByOutputAndInputAndHeldToTheLimits() {
        var mix = ColorMixValues()
        mix[1, 2] = 40
        #expect(mix.matrix[5] == 40 && mix[1, 2] == 40)
        mix[0, 0] = 999
        #expect(mix[0, 0] == 200)
        mix[2, 0] = -999
        #expect(mix[2, 0] == -200)
        #expect(!mix.isIdentity)
        // What the field can hold is also what the core is sent.
        mix.matrix[3] = 5000
        #expect(mix.core.matrix[3] == 200)
    }

    @Test func theMixIsPartOfWhatTheCoreIsSent() {
        var tone = ToneValues()
        tone.mix.matrix = ColorMixValues.identityMatrix
        tone.mix[0, 2] = 30
        tone.mix.preserveBrightness = true
        #expect(tone.core.mix == ColorMix(matrix: [100, 0, 30, 0, 100, 0, 0, 0, 100], preserveBrightness: true))
        // A moved weight changes the histogram before the stretch as well; Preserve brightness too.
        #expect(HistogramKey(tone.core, after: false, base: 1) != HistogramKey(ToneValues().core, after: false, base: 1))
    }

    @Test func aWeightThatIsNotNineFiniteNumbersIsIgnored() {
        #expect(mixMatrix(from: [1, 2, 3]) == nil)
        #expect(mixMatrix(from: [100, 0, 0, 0, 100, 0, 0, 0, .infinity]) == nil)
        #expect(mixMatrix(from: [100, 0, 0, 0, 100, 0, 0, 0, 400]) == [100, 0, 0, 0, 100, 0, 0, 0, 200])
    }

    // MARK: Color Mixing, model

    @Test func aMovedWeightAsksTheCoreForTheMixedImageAndNeutralAsksForNothing() async {
        await stackedRecording()
        let asked = backend.wavelets.appliedColours.count
        model.toggleFunction("Color Mixing")
        #expect(model.openCards == ["Color Mixing"])
        model.tone.mix[0, 1] = 50
        await model.settleColours()
        #expect(backend.wavelets.appliedColours.count == asked + 1)
        #expect(backend.wavelets.appliedColours.last?.mix.matrix == [100, 50, 0, 0, 100, 0, 0, 0, 100])
        #expect(model.waveletData.adjusted != nil)
        // Save image writes the mix with the other adjustments, and nothing once it is neutral again.
        #expect(model.saveAdjustments?.mix.matrix == [100, 50, 0, 0, 100, 0, 0, 0, 100])

        // Back to neutral: the image is the base again, and nothing is sent.
        model.tone.mix.matrix = ColorMixValues.identityMatrix
        await model.settleColours()
        #expect(model.waveletData.adjusted == nil)
        #expect(backend.wavelets.appliedColours.count == asked + 1)
        #expect(model.saveAdjustments == nil)
    }

    @Test func resettingTheToneValuesAlsoResetsTheMix() {
        model.tone.mix[2, 0] = -80
        model.tone.mix.preserveBrightness = true
        model.resetTone()
        #expect(model.tone.mix == ColorMixValues())
    }

    // MARK: Color Mixing, project file

    @Test func theMixSurvivesTheProjectSection() {
        var tone = ToneValues()
        tone.mix[0, 1] = -35
        tone.mix[2, 2] = 150
        tone.mix.preserveBrightness = true
        let section = waveletSection(WaveletOptions(), tone: tone)

        var options = WaveletOptions()
        var read = ToneValues()
        applyWaveletSection(section, to: &options, tone: &read)
        #expect(read.mix == tone.mix)

        // The JSON text the core stores gives the same.
        let text = JSON.object(["wavelet": section]).text
        #expect(text.contains("\"preserveBrightness\":true"))
    }

    @Test func aProjectWithoutAMixOrWithABrokenOneKeepsTheIdentity() {
        // A project of the Windows app has no mix.
        var windows = waveletSection(WaveletOptions(), tone: ToneValues())
        if case .object(var top) = windows, case .object(var card)? = top["tone"] {
            card["mix"] = nil
            top["tone"] = .object(card)
            windows = .object(top)
        }
        var tone = ToneValues()
        tone.mix[0, 0] = 10
        var options = WaveletOptions()
        applyWaveletSection(windows, to: &options, tone: &tone)
        #expect(tone.mix[0, 0] == 10, "what the file does not say stays as it is")

        var broken = ToneValues()
        if case .object(var top) = windows, case .object(var card)? = top["tone"] {
            card["mix"] = .object(["matrix": .array([.number(1), .string("x")]), "preserveBrightness": .string("yes")])
            top["tone"] = .object(card)
            applyWaveletSection(.object(top), to: &options, tone: &broken)
        }
        #expect(broken.mix == ColorMixValues())
    }

    // MARK: View Compare, the divider

    @Test func theDividerStaysInsideTheArea() {
        #expect(clampedDivider(0.5) == 0.5)
        #expect(clampedDivider(-3) == compareDividerRange.lowerBound)
        #expect(clampedDivider(7) == compareDividerRange.upperBound)
        #expect(clampedDivider(.nan) == compareDividerStart)
        #expect(dividerShare(x: 300, width: 1000) == 0.3)
        #expect(dividerShare(x: -50, width: 1000) == compareDividerRange.lowerBound)
        #expect(dividerShare(x: 5000, width: 1000) == compareDividerRange.upperBound)
        #expect(dividerShare(x: 10, width: 0) == compareDividerStart)
    }

    @Test func aLabelHidesWhenTheDividerLeavesNoRoomForIt() {
        #expect(compareLabelFits(share: 0.5, width: 800, needed: 84, side: .before))
        #expect(compareLabelFits(share: 0.5, width: 800, needed: 84, side: .after))
        #expect(!compareLabelFits(share: 0.05, width: 800, needed: 84, side: .before))
        #expect(compareLabelFits(share: 0.05, width: 800, needed: 84, side: .after))
        #expect(!compareLabelFits(share: 0.95, width: 800, needed: 84, side: .after))
    }

    @Test func theModelClampsTheDividerAndMovesItByTheKeyboardStep() {
        model.setCompareDivider(0.3)
        #expect(model.wavelet.compareDivider == 0.3)
        model.setCompareDivider(-1)
        #expect(model.wavelet.compareDivider == compareDividerRange.lowerBound)
        model.setCompareDivider(0.97)
        model.nudgeCompareDivider(by: 0.05)
        #expect(model.wavelet.compareDivider == compareDividerRange.upperBound)
    }

    // MARK: View Compare, the toggle

    @Test func theButtonSplitsTheViewAndBeforeAfterEndsItAgain() async {
        await stackedRecording()
        #expect(!model.isComparing && model.compareBeforeImage == nil)
        model.toggleFunction(compareName)
        #expect(model.isComparing)
        #expect(model.openCards.isEmpty, "View Compare is a view, not a card")
        // The left half is the stack, the right half what the image area shows otherwise.
        #expect(model.compareBeforeImage === model.waveletData.stack?.image)
        #expect(model.shownImage === model.waveletData.processed?.image)

        // Before/After shows the stack in full: the split view ends and the button comes up.
        model.setBefore(true)
        #expect(!model.isComparing && !model.compareIsOn && model.waveletData.before)
        model.setBefore(false)
        #expect(!model.isComparing, "the split view does not come back by itself")

        // Pressing View Compare while Before is on goes back to the result and splits it.
        model.setBefore(true)
        model.toggleFunction(compareName)
        #expect(model.isComparing && !model.waveletData.before)

        model.toggleFunction(compareName)
        #expect(!model.isComparing)
    }

    @Test func theSplitViewBelongsToTheWaveletsStepAndStaysInTheProject() async {
        await stackedRecording()
        model.toggleFunction(compareName)
        model.show(.stack)
        #expect(!model.isComparing)
        model.show(.wavelets)
        #expect(model.isComparing)

        // The button is stored with the other pressed buttons; the divider is not.
        let section = waveletSection(model.wavelet, tone: model.tone)
        var options = WaveletOptions()
        var tone = ToneValues()
        applyWaveletSection(section, to: &options, tone: &tone)
        #expect(options.functions == [compareName])
        #expect(options.compareDivider == compareDividerStart)
    }

    @Test func theComparisonNeedsTheStack() {
        model.toggleFunction(compareName)
        #expect(model.compareIsOn && !model.isComparing)
    }

    @Test func theTwoNewButtonsAreUnlocked() {
        #expect(isFunctionBuilt("Color Mixing") && isToneCard("Color Mixing"))
        #expect(isFunctionBuilt(compareName) && !isToneCard(compareName))
        #expect(functionNames.contains("Color Mixing") && functionNames.contains(compareName))
    }
}
