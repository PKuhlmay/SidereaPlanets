import Testing
import UniformTypeIdentifiers

@testable import SidereaPlanets

/// A project saved by the app must be selectable in Open Project: the type the system gives a `.rkproj` file has to conform
/// to the type the open panel allows.
@MainActor
@Suite struct ProjectFileTypeTests {
    @Test func aSavedProjectFileCanBeChosenInOpenProject() throws {
        let typeOfSavedFile = try #require(UTType(filenameExtension: "rkproj"))
        #expect(typeOfSavedFile.conforms(to: SavePanels.projectType))
    }
}
