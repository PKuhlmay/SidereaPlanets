import Testing

@testable import SidereaPlanets

/// The Show menu dims the check marks that change nothing on screen (MAC-014).
@Suite struct ShowMenuLogicTests {
    @Test func entriesWithoutAnEffectAreNotAvailable() {
        let frameSteps = showItems(for: .limit)
        #expect(frameSteps.filter { !$0.isAvailable }.map(\.title) == ["Show Full Image", "Show Align Data", "Show Prefilter"])
        #expect(showItems(for: .stack).filter { !$0.isAvailable }.map(\.title) == ["Show Full Image", "Show Stack Graph", "Show Stack List"])
        #expect(showItems(for: .wavelets).filter { !$0.isAvailable }.map(\.title) == ["Show Full Image"])
    }

    @Test func entriesWithAnEffectStayAvailable() {
        #expect(showItems(for: .align).filter(\.isAvailable).map(\.title) == ["Show Region", "Show Alignpoints", "Show Frame List", "Show Quality Graph"])
        #expect(showItems(for: .wavelets).filter(\.isAvailable).map(\.title) == ["Show Alignpoints", "Show Preview Area"])
    }
}
