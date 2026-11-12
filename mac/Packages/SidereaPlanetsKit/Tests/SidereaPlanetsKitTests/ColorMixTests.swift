import Foundation
import Testing

@testable import SidereaPlanetsKit

@Suite("Color Mixing through the interface")
struct ColorMixTests {
    private func settings(matrix: [Float], preserveBrightness: Bool = false) -> ColorSettings {
        let none = ChannelShift(x: 0, y: 0)
        return ColorSettings(
            shifts: ChannelShifts(red: none, green: none, blue: none),
            mix: ColorMix(matrix: matrix, preserveBrightness: preserveBrightness),
            red: 100, green: 100, blue: 100, black: 0, white: 255, gamma: 1, contrast: 100, brightness: 0)
    }

    private let source = RgbaImage(width: 2, height: 1, rgba: Data([200, 100, 50, 255, 10, 20, 30, 255]))

    @Test("The identity leaves the pixels as they are, a swap of red and blue swaps them")
    func identityAndSwap() async throws {
        let session = try PlanetsSession()
        _ = try await session.setColorSource(source)

        let same = try await session.applyColors(settings(matrix: [100, 0, 0, 0, 100, 0, 0, 0, 100]))
        #expect(same.rgba == source.rgba)

        let swapped = try await session.applyColors(settings(matrix: [0, 0, 100, 0, 100, 0, 100, 0, 0]))
        #expect(Array(swapped.rgba) == [50, 100, 200, 255, 30, 20, 10, 255])
    }

    @Test("Preserve brightness keeps the luminance of a pixel")
    func preserveBrightness() async throws {
        let session = try PlanetsSession()
        _ = try await session.setColorSource(source)
        let mixed = try await session.applyColors(
            settings(matrix: [100, 0, 0, 100, 0, 0, 0, 0, 100], preserveBrightness: true))
        func luma(_ bytes: [UInt8]) -> Double {
            0.299 * Double(bytes[0]) + 0.587 * Double(bytes[1]) + 0.114 * Double(bytes[2])
        }
        let before = luma(Array(source.rgba.prefix(3)))
        let after = luma(Array(mixed.rgba.prefix(3)))
        #expect(abs(before - after) < 1.5, "\(before) vs \(after)")
    }
}
