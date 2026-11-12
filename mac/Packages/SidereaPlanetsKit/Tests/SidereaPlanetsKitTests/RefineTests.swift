import Foundation
import Testing

@testable import SidereaPlanetsKit

/// Denoise/Deringing and Masking through the interface (MAC-018): the defaults come from the core and the mask preview needs a stack.
@Suite("Denoise/Deringing and Masking through the interface")
struct RefineInterfaceTests {
    @Test("The defaults of the core change nothing")
    func defaults() {
        let d = refineDefaultSettings()
        #expect(d.denoise == 0 && d.deringing == 0 && d.deringingRadius == 4)
        #expect(!d.mask.enabled && d.mask.threshold == 20 && d.mask.feather == 4 && !d.mask.invert)
    }

    @Test("The mask preview without a stack fails with the text of the session")
    func noStack() async throws {
        let session = try PlanetsSession()
        do {
            _ = try await session.refineMaskOverlay(refineDefaultSettings())
            Issue.record("there is no stack to make the mask from")
        } catch let error as PlanetsError {
            #expect(error.message.contains("no stacked image"), "\(error.message)")
        }
    }
}
