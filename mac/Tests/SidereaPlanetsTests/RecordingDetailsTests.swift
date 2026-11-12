import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

@Suite struct RecordingDetailsTests {
    private func details(observer: String? = nil, fps: Double? = 25) -> RecordingDetails {
        var info = FakeBackend.info(frames: 10)
        info.observer = observer
        info.fps = fps
        return RecordingDetails(fileName: "JUPITER.AVI", info: info)
    }

    @Test func summaryAndStepBarLineLikeTheWindowsApp() {
        #expect(details().summary == "DIB · 2×2 · 10 frames")
        #expect(details().stepBarLine == "JUPITER.AVI · DIB · 2×2 · 10 frames")
    }

    @Test func rowsListObserverOnlyWhereTheFileHasOne() {
        let labels = details().rows.map(\.label)
        #expect(labels == ["Format", "Codec", "Size", "Bit depth", "Frames", "Colour", "Frame rate"])
        #expect(details(observer: "P. K.").rows.map(\.label).last == "Observer")
        #expect(details(fps: nil).rows.map(\.label).contains("Frame rate") == false)
    }

    @Test func frameRate() {
        #expect(RecordingDetails.formatRate(25) == "25 fps")
        #expect(RecordingDetails.formatRate(29.97) == "29.97 fps")
    }
}
