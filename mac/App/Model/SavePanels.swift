import AppKit
import SidereaPlanetsKit
import UniformTypeIdentifiers

/// The native dialogs of Save image, Save Aligned, Save Integrated, Save project and Open project. The file type is chosen before
/// (the menu of the button), so the panel writes exactly that type: it keeps the extension on the name.
@MainActor
enum SavePanels {
    /// The project file (`.rkproj`, JSON), declared in Info.plist (`UTExportedTypeDeclarations`, mac/project.yml). A type made up
    /// from the extension here would not match the type the system gives a saved file, and Open Project would grey it out.
    static let projectType = UTType(exportedAs: "de.kuhlmay.siderea.planets.project", conformingTo: .json)

    /// The URL the user chose, or `nil` if the dialog was cancelled.
    static func save(_ request: SaveRequest) async -> URL? {
        let panel = NSSavePanel()
        panel.canCreateDirectories = true
        panel.nameFieldStringValue = request.name
        panel.directoryURL = request.directory
        if let ext = request.fileExtension, let type = UTType(filenameExtension: ext) {
            panel.allowedContentTypes = [type]
            panel.allowsOtherFileTypes = false
        }
        return await panel.begin() == .OK ? panel.url : nil
    }

    /// The project the user chose, or `nil` if the dialog was cancelled.
    static func openProject(startingIn directory: URL?) async -> URL? {
        let panel = NSOpenPanel()
        panel.prompt = "Open"
        panel.message = "Open a project: the recording and the settings of the steps."
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = false
        panel.allowedContentTypes = [projectType]
        panel.directoryURL = directory
        return await panel.begin() == .OK ? panel.url : nil
    }
}

extension AppModel {
    /// ⌘S: Save image in the file type chosen last.
    func chooseAndSaveImage() async {
        guard let format = currentFormat(for: saveSource) else { return }
        await saveImage(format, choose: SavePanels.save)
    }

    func chooseAndSaveImage(_ format: FormatInfo) async { await saveImage(format, choose: SavePanels.save) }
    func chooseAndSaveVideo(_ kind: VideoKind) async { await saveVideo(kind, choose: SavePanels.save) }
    func chooseAndSaveProject() async { await saveProject(choose: SavePanels.save) }
    func chooseAndOpenProject() async { await openProject(choose: SavePanels.openProject) }
}
