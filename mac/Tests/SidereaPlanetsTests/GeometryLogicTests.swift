import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The pure parts of Flip and Rotate and Resize Image (MAC-017): the values, their limits, the sizes and the part of the project file.
@Suite struct GeometryLogicTests {
    // MARK: Flip and Rotate

    @Test func anglesWrapIntoMinus180To180InTenths() {
        #expect(OrientationValues.wrapped(190) == -170)
        #expect(OrientationValues.wrapped(-190) == 170)
        #expect(OrientationValues.wrapped(180) == 180)
        #expect(OrientationValues.wrapped(-180) == 180)
        #expect(OrientationValues.wrapped(360) == 0)
        #expect(OrientationValues.wrapped(12.34) == 12.3)
    }

    @Test func theRotateButtonsTurnTheScreenImage() {
        var values = OrientationValues()
        values.rotate(by: 90)
        #expect(values.angle == 90)
        values.rotate(by: 90)
        #expect(values.angle == 180)
        values.rotate(by: 90)
        #expect(values.angle == -90)
        values.rotate(by: -90)
        values.rotate(by: -90)
        values.rotate(by: -90)
        #expect(values.angle == 0)
        values.rotate(by: 180)
        values.rotate(by: 180)
        #expect(values.isNeutral)
    }

    @Test func aFlipMirrorsWhatIsOnScreenSoItChangesTheSignOfTheAngle() {
        // The core mirrors first and turns after; to mirror the picture as it is seen, the angle turns the other way.
        var values = OrientationValues()
        values.setAngle(30)
        values.flipHorizontally()
        #expect(values.flipHorizontal && !values.flipVertical && values.angle == -30)
        values.flipHorizontally()
        #expect(!values.flipHorizontal && values.angle == 30)
        values.flipVertically()
        #expect(values.flipVertical && values.angle == -30)
        values.flipVertically()
        #expect(values.angle == 30)
        // Without an angle a flip is just the flag.
        var plain = OrientationValues()
        plain.flipHorizontally()
        #expect(plain.flipHorizontal && plain.angle == 0 && !plain.isNeutral)
    }

    @Test func theAngleIsHeldToTheLimitsAndToTenths() {
        var values = OrientationValues()
        values.setAngle(250)
        #expect(values.angle == 180)
        values.setAngle(-999)
        #expect(values.angle == 180)  // -180 and 180 are the same picture; the value is kept as 180
        values.setAngle(15.26)
        #expect(values.angle == 15.3)
        values.setAngle(0.04)
        #expect(values.angle == 0 && values.isNeutral)
    }

    @Test func theValuesGoToTheCoreAsTheyAre() {
        var values = OrientationValues()
        values.setAngle(-12.5)
        values.flipVertical = true
        values.cropToFit = true
        values.method = .bilinear
        let core = values.core
        #expect(core.angle == -12.5 && core.flipVertical && !core.flipHorizontal && core.cropToFit && core.method == .bilinear)
        #expect(OrientationValues().core.method == .bicubic)
    }

    @Test func theSummaryNamesWhatChanges() {
        var values = OrientationValues()
        #expect(values.summary == nil)
        values.setAngle(15)
        #expect(values.summary == "turned 15.0° clockwise")
        values.flipHorizontally()
        #expect(values.summary == "turned 15.0° counterclockwise, flipped horizontally")
    }

    @Test func theLayoutIgnoresTheMethod() {
        var values = OrientationValues()
        let before = values.layout
        values.method = .bilinear
        #expect(values.layout == before)
        values.cropToFit = true
        #expect(values.layout != before)
    }

    // MARK: Resize Image

    private let source = (width: 1200, height: 900)

    @Test func aPercentageScalesBothSides() {
        var values = ResizeValues()
        #expect(values.isNeutral(source: source) && values.size(source: source) == (1200, 900))
        values.percent = 50
        #expect(values.size(source: source) == (600, 450))
        #expect(!values.isNeutral(source: source))
        values.percent = 400
        #expect(values.size(source: source) == (4800, 3600))
    }

    @Test func aLockedSideDecidesTheOther() {
        var values = ResizeValues()
        values.switchToPixels(source: source)
        #expect(values.mode == .pixels && values.width == 1200 && values.height == 900)
        values.setWidth(800, source: source)
        #expect(values.driver == .width && values.size(source: source) == (800, 600))
        values.setHeight(450, source: source)
        #expect(values.driver == .height && values.size(source: source) == (600, 450))
        #expect(values.core.target == .pixels(width: nil, height: 450))
        // Typed sides are held to what the core takes.
        values.setWidth(99_999, source: source)
        #expect(values.width == ResizeLimits.side.upperBound)
        values.setWidth(0, source: source)
        #expect(values.width == 1)
    }

    @Test func withoutTheLockBothSidesAreFixed() {
        var values = ResizeValues()
        values.switchToPixels(source: source)
        values.setLock(false, source: source)
        #expect(!values.lockAspect)
        values.setWidth(800, source: source)
        #expect(values.size(source: source) == (800, 900))
        values.setHeight(100, source: source)
        #expect(values.size(source: source) == (800, 100))
        #expect(values.core.target == .pixels(width: 800, height: 100))
        // A picture of another size keeps both numbers.
        #expect(values.size(source: (width: 10, height: 10)) == (800, 100))
        // The lock back on: the side typed last decides again.
        values.setLock(true, source: source)
        #expect(values.size(source: source) == (1200 * 100 / 900, 100))
    }

    @Test func theMethodsAreInTheOrderOfTheMenuWithLanczosFirst() {
        #expect(ResizeMethod.allCases == [.lanczos3, .bicubic, .bilinear, .nearest])
        #expect(ResizeValues().method == .lanczos3)
        #expect(ResizeMethod.allCases.map(\.title) == ["Lanczos3", "Bicubic", "Bilinear", "Nearest Neighbor"])
        #expect(ResizeValues().core.method == .lanczos3)
    }

    @Test func theSizeTextHasTheUnit() {
        #expect(sizeText(1200, 900) == "1200 × 900 px")
    }

    // MARK: Project file

    @Test func theSectionKeepsEveryValue() {
        var values = GeometryValues()
        values.orientation.setAngle(-37.5)
        values.orientation.flipHorizontal = true
        values.orientation.cropToFit = true
        values.orientation.method = .bilinear
        values.resize.mode = .pixels
        values.resize.percent = 250
        values.resize.width = 800
        values.resize.height = 600
        values.resize.lockAspect = false
        values.resize.driver = .height
        values.resize.method = .nearest
        var loaded = GeometryValues()
        applyGeometrySection(geometrySection(values), to: &loaded)
        #expect(loaded == values)
    }

    @Test func aMissingSectionKeepsTheValuesAndWrongValuesAreCut() {
        var values = GeometryValues()
        values.orientation.setAngle(10)
        applyGeometrySection(nil, to: &values)
        #expect(values.orientation.angle == 10)

        let broken: JSON = .object([
            "angle": .number(900), "flipHorizontal": .string("yes"), "rotationMethod": .string("lanczos"),
            "resize": .object(["percent": .number(5000), "width": .number(-4), "height": .number(500_000_000), "method": .string("nearest")]),
        ])
        applyGeometrySection(broken, to: &values)
        #expect(values.orientation.angle == 180 && !values.orientation.flipHorizontal && values.orientation.method == .bicubic)
        #expect(values.resize.percent == 400 && values.resize.width == 0 && values.resize.height == ResizeLimits.side.upperBound)
        #expect(values.resize.method == .nearest)
    }
}
