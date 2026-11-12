import AppKit
import UniformTypeIdentifiers

/// The native dialog for Select Recording (⌘O). The types are those listed in
/// docs/formats.md: SER and AVI videos, still frames (BMP, JPEG, TIFF, PNG) and FITS frames. Several still frames
/// together form one sequence, so multiple selection is on.
@MainActor
enum RecordingPicker {
    static let videoExtensions = ["avi", "ser"]
    static let stillExtensions = ["bmp", "jpg", "jpeg", "tif", "tiff", "png"]
    static let fitsExtensions = ["fit", "fits", "fts"]

    /// The files the user picked, in name order (the order of a sequence); empty if the dialog was cancelled.
    static func choose() async -> [URL] {
        let panel = NSOpenPanel()
        panel.message = ViewStep.open.sentence
        panel.prompt = "Open"
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowsMultipleSelection = true
        panel.allowedContentTypes = (videoExtensions + stillExtensions + fitsExtensions).compactMap { UTType(filenameExtension: $0) }
        guard await panel.begin() == .OK else { return [] }
        return panel.urls.sorted { $0.lastPathComponent.localizedStandardCompare($1.lastPathComponent) == .orderedAscending }
    }
}

extension AppModel {
    /// Select: asks for files and opens them.
    func select() async {
        guard !isBusy else { return }
        await open(await RecordingPicker.choose())
    }
}
