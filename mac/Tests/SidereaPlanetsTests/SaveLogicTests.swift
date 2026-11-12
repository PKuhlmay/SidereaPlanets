import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The pure parts of the step Save and of the project file: names, what is sent to the core, the sections of the project.
@Suite struct SaveLogicTests {
    @Test func theDialogProposesTheNameOfTheRecordingWithoutItsExtension() {
        #expect(fileStem("JUPITER.AVI") == "JUPITER")
        #expect(fileStem("moon.part1.ser") == "moon.part1")
        #expect(fileStem("noextension") == "noextension")
        #expect(fileStem(".hidden") == ".hidden")
    }

    @Test func createAviSendsTheStackRequestAndItsOwnOptions() {
        var stack = StackOptions()
        stack.framerate = 0  // typed values are held to what the core can take
        stack.averageFrames = -3
        stack.sortOnQuality = true
        var data = AlignData(frameCount: 4)
        data.frames[1].checked = false
        let settings = sequenceSettings(stack, align: AlignOptions(), data: data)
        #expect(settings.framerate == 1 && settings.averageFrames == 1)
        #expect(settings.sortOnQuality)
        #expect(settings.stack.frames == [0, 2, 3])
        #expect(settings.stack.nearestPoints == 3 && settings.stack.correctGeometry)

        #expect(StackOptions().framerate == 25 && StackOptions().averageFrames == 5)
        #expect(sequenceSettings(StackOptions(), align: AlignOptions(), data: data).framerate == 25)
    }

    @Test func colourValuesThatLeaveTheImageAsItIsAreNeutral() {
        let none = ChannelShift(x: 0, y: 0)
        var colour = ColorSettings(
            shifts: ChannelShifts(red: none, green: none, blue: none), red: 100, green: 100, blue: 100,
            black: 0, white: 255, gamma: 1, contrast: 100, brightness: 0
        )
        #expect(isNeutral(colour))
        colour.gamma = 1.2
        #expect(!isNeutral(colour))
        colour.gamma = 1
        colour.shifts.blue = ChannelShift(x: 0, y: 1)
        #expect(!isNeutral(colour))
    }

    // MARK: Project sections

    @Test func jsonValuesSurviveTheTextTheCoreStores() throws {
        let value = JSON.object(["a": .number(30), "b": .bool(true), "c": .string("x"), "d": .array([.number(0.5), .null])])
        let again = try #require(JSON(text: value.text))
        #expect(again == value)
        #expect(JSON(text: "{broken") == nil)
        // A whole number is written without a decimal point, as the Windows app writes it.
        #expect(JSON.number(30).text == "30")
        #expect(JSON.bool(true).text == "true")
    }

    @Test func alignAndStackSectionsRoundTrip() {
        var options = AlignOptions()
        options.minDistance = 44
        options.intensity = .area3x3
        options.limitMode = .bestCount
        options.limitCount = 7
        var show = ShowOptions()
        show.framelist = true
        show.alignpoints = false
        var stack = StackOptions()
        stack.nearestPoints = 5
        stack.codec = "cvid"
        stack.alignedSize = .roi
        stack.averageFrames = 9

        var optionsBack = AlignOptions()
        var showBack = ShowOptions()
        var stackBack = StackOptions()
        applyAlignSection(alignSection(options, show: show), to: &optionsBack, show: &showBack)
        applyStackSection(stackSection(stack), to: &stackBack)
        #expect(optionsBack == options && showBack == show && stackBack == stack)
    }

    @Test func theAlignSectionHasTheKeysOfTheWindowsSettingsWithoutTheThreshold() throws {
        guard case .object(let keys) = alignSection(AlignOptions(), show: ShowOptions()) else { throw Missing() }
        #expect(Set(keys.keys) == [
            "showFullImage", "showRoi", "showAlignpoints", "showAligndata", "showFramelist", "showRegistrationgraph", "showPrefilter",
            "minDistance", "edgeDistance", "intensity", "lo", "hi", "showAlignment", "boxSize", "maxMovement", "limitMode",
            "limitPercent", "limitCount",
        ])
        #expect(keys["intensity"] == .string("default"))
        #expect(keys["limitMode"] == .string("lowestQuality"))
    }

    @Test func onlyKeysOfTheRightTypeAreTaken() throws {
        let section = try #require(JSON(text: """
            {"minDistance": "forty", "edgeDistance": 12.4, "lo": null, "boxSize": 55, "intensity": "nonsense",
             "showFramelist": 1, "showRoi": true, "limitMode": "bestPercent", "unknownKey": 5}
            """))
        var options = AlignOptions()
        var show = ShowOptions()
        applyAlignSection(section, to: &options, show: &show)
        #expect(options.minDistance == 30, "a string for a number is ignored")
        #expect(options.edgeDistance == 12, "a number is rounded to a whole number")
        #expect(options.lo == 30 && options.intensity == .default)
        #expect(options.boxSize == 55 && options.limitMode == .bestPercent)
        #expect(!show.framelist, "a number for a check mark is ignored")
        #expect(show.roi)
    }

    @Test func aMissingSectionKeepsEveryValue() {
        var stack = StackOptions()
        stack.nearestPoints = 8
        applyStackSection(nil, to: &stack)
        applyStackSection(.string("not a section"), to: &stack)
        #expect(stack.nearestPoints == 8)
    }

    @Test func theProjectCarriesTheRecordingAndTheSectionsAsText() throws {
        let project = makeProject(
            recording: [URL(fileURLWithPath: "/data/a.ser")], sections: ["align": .object(["boxSize": .number(30)])]
        )
        #expect(project.formatVersion == 1 && project.recording == ["/data/a.ser"])
        #expect(project.sections["align"] == #"{"boxSize":30}"#)
        #expect(sections(of: project) == ["align": .object(["boxSize": .number(30)])])
    }

    struct Missing: Error {}
}
