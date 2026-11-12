import Foundation
import Testing

@testable import SidereaPlanetsKit

@Suite("Flip and Rotate and Resize Image through the interface")
struct GeometryTests {
    private func orientation(angle: Float = 0, flipHorizontal: Bool = false, cropToFit: Bool = false) -> OrientationSettings {
        OrientationSettings(
            flipHorizontal: flipHorizontal, flipVertical: false, angle: angle, cropToFit: cropToFit, method: .bicubic)
    }

    /// 3 x 2 pixels whose red channel counts 0 ... 5.
    private let source = RgbaImage(
        width: 3, height: 2, rgba: Data((0..<6).flatMap { [UInt8($0), 10, 20, 255] }))

    @Test("A quarter turn clockwise moves the pixels and swaps width and height")
    func quarterTurn() async throws {
        let session = try PlanetsSession()
        let turned = try await session.orientPreview(source, orientation(angle: 90))
        #expect((turned.width, turned.height) == (2, 3))
        #expect(stride(from: 0, to: turned.rgba.count, by: 4).map { turned.rgba[$0] } == [3, 0, 4, 1, 5, 2])
    }

    @Test("Flipping horizontally mirrors every row; flipping twice is the picture again")
    func flip() async throws {
        let session = try PlanetsSession()
        let mirrored = try await session.orientPreview(source, orientation(flipHorizontal: true))
        #expect(stride(from: 0, to: mirrored.rgba.count, by: 4).map { mirrored.rgba[$0] } == [2, 1, 0, 5, 4, 3])
        let back = try await session.orientPreview(mirrored, orientation(flipHorizontal: true))
        #expect(back.rgba == source.rgba)
    }

    @Test("The sizes the card shows come from the core")
    func sizes() {
        #expect(orientationSize(width: 100, height: 50, orientation: orientation(angle: 90)) == ImageSize(width: 50, height: 100))
        #expect(orientationSize(width: 100, height: 50, orientation: orientation(angle: 30)) == ImageSize(width: 112, height: 94))
        let half = ResizeSettings(target: .percent(percent: 50), method: .lanczos3)
        #expect(resizeSize(width: 1024, height: 680, resize: half) == ImageSize(width: 512, height: 340))
        let wide = ResizeSettings(target: .pixels(width: 800, height: nil), method: .lanczos3)
        #expect(resizeSize(width: 1200, height: 900, resize: wide) == ImageSize(width: 800, height: 600))
    }
}
