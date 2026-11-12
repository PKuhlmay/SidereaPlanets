import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The values of the cards Denoise/Deringing and Masking: defaults, what is sent to the core, the check for "changes nothing" and the
/// part of the project file they are saved in (MAC-018).
@Suite struct RefineLogicTests {
    // MARK: Values

    @Test func theDefaultsChangeNothingAndComeFromTheCore() {
        let defaults = RefineValues.defaults
        #expect(defaults == RefineValues())
        #expect(isNeutral(defaults.core))
        #expect(defaults.denoise == 0 && defaults.deringing == 0 && defaults.radius == 4)
        #expect(!defaults.maskOn && defaults.threshold == 20 && defaults.feather == 4 && !defaults.invert && !defaults.showMask)
    }

    @Test func theValuesGoToTheCoreInTheUnitsOfTheCards() {
        var values = RefineValues()
        values.denoise = 35
        values.deringing = 70
        values.radius = 7
        values.maskOn = true
        values.threshold = 90
        values.feather = 12
        values.invert = true
        values.showMask = true
        let core = values.core
        #expect(core.denoise == 35 && core.deringing == 70 && core.deringingRadius == 7)
        #expect(core.mask == MaskSettings(enabled: true, threshold: 90, feather: 12, invert: true))
        // Show mask only decides what the window shows: it is not sent.
        var other = values
        other.showMask = false
        #expect(other.core == core)
    }

    @Test func theCoreValuesAreCutToTheLimitsOfTheCards() {
        let values = RefineValues(core: RefineSettings(
            denoise: 250, deringing: -4, deringingRadius: 99,
            mask: MaskSettings(enabled: true, threshold: 999, feather: 500, invert: false)
        ))
        #expect(values.denoise == 100 && values.deringing == 0 && values.radius == 10)
        #expect(values.threshold == 255 && values.feather == 50)
    }

    @Test func anythingButNeutralValuesAreNotNeutral() {
        var values = RefineValues()
        values.denoise = 1
        #expect(!isNeutral(values.core))
        values = RefineValues()
        values.deringing = 1
        #expect(!isNeutral(values.core))
        values = RefineValues()
        values.maskOn = true
        #expect(!isNeutral(values.core))
        // A radius, threshold or feather alone changes nothing while the strengths are 0 and the mask is off.
        values = RefineValues()
        values.radius = 9
        values.threshold = 200
        values.feather = 30
        values.invert = true
        #expect(isNeutral(values.core))
    }

    @Test func theCardsOfTheColourValuesCountForNeutral() {
        var tone = ToneValues()
        #expect(isNeutral(tone.core))
        tone.refine.denoise = 20
        #expect(!isNeutral(tone.core) && tone.core.refine.denoise == 20)
    }

    @Test func eachResetBringsBackItsOwnCardOnly() {
        var values = RefineValues()
        values.denoise = 30
        values.deringing = 60
        values.radius = 8
        values.maskOn = true
        values.threshold = 90
        values.feather = 20
        values.invert = true
        values.showMask = true

        var denoise = values
        denoise.resetDenoise()
        #expect(denoise.denoise == 0 && denoise.deringing == 0 && denoise.radius == 4)
        #expect(denoise.maskOn && denoise.threshold == 90 && denoise.showMask)

        var mask = values
        mask.resetMask()
        #expect(!mask.maskOn && mask.threshold == 20 && mask.feather == 4 && !mask.invert && !mask.showMask)
        #expect(mask.denoise == 30 && mask.deringing == 60 && mask.radius == 8)
    }

    @Test func theCardsHaveATooltip() {
        #expect(functionHelp("Denoise/Deringing") != "Not available yet")
        #expect(functionHelp("Masking") != "Not available yet")
    }

    // MARK: Project file

    @Test func theCardsAreSavedWithoutShowMask() {
        var values = RefineValues()
        values.denoise = 25
        values.maskOn = true
        values.threshold = 44
        values.showMask = true
        guard case .object(let refine) = refineSection(values) else {
            Issue.record("no object")
            return
        }
        #expect(refine["denoise"] == .number(25) && refine["maskOn"] == .bool(true) && refine["threshold"] == .number(44))
        #expect(refine["showMask"] == nil, "Show mask is not saved")
    }

    @Test func theCardsComeBackFromTheirSection() {
        let values = RefineValues(core: RefineSettings(
            denoise: 25, deringing: 50, deringingRadius: 9,
            mask: MaskSettings(enabled: true, threshold: 44, feather: 17, invert: true)
        ))
        var loaded = RefineValues()
        applyRefineSection(refineSection(values), to: &loaded)
        #expect(loaded == values)
    }

    @Test func aProjectWithoutTheSectionLeavesTheValuesAsTheyAre() {
        var values = RefineValues()
        values.denoise = 12
        applyRefineSection(nil, to: &values)
        #expect(values.denoise == 12)
    }

    @Test func numbersOutsideTheLimitsInAProjectAreCutAndWrongTypesAreIgnored() {
        let refine = JSON.object([
            "denoise": .number(400), "deringing": .string("much"), "radius": .number(0), "maskOn": .number(1),
            "threshold": .number(-5), "feather": .number(75), "invert": .bool(true),
        ])
        var values = RefineValues()
        applyRefineSection(refine, to: &values)
        #expect(values.denoise == 100 && values.deringing == 0 && values.radius == 1)
        #expect(!values.maskOn && values.threshold == 0 && values.feather == 50 && values.invert)
    }
}

