import SidereaPlanetsKit
import SwiftUI

@main
struct SidereaPlanetsApp: App {
    @State private var model = AppModel.live()

    var body: some Scene {
        // One window: the app works on one recording at a time.
        Window("Siderea Planets", id: "main") {
            ContentView(model: model)
        }
        .defaultSize(width: 1440, height: 900)
        .windowToolbarStyle(.unified)
        .commands {
            AppCommands(model: model)
            HelpCommands()
        }
        HelpScene()
        Settings { SettingsView(model: model) }
    }
}

extension AppModel {
    /// The app model on the real Rust core. The session only fails if the thread pool cannot be created, and then there
    /// is nothing the app could do.
    static func live() -> AppModel {
        do {
            return AppModel(backend: try PlanetsSession())
        } catch {
            fatalError("The calculation core did not start: \(errorText(error))")
        }
    }
}

/// The menu bar: ⌘O Select Recording and Open Recent in File, zoom and frame stepping in View, Cancel (⌘.) and Pause in Run
/// (docs/design.md, "Bewusste Abweichungen").
struct AppCommands: Commands {
    let model: AppModel

    var body: some Commands {
        CommandGroup(replacing: .newItem) {
            Button("Select Recording…") { Task { await model.select() } }
                .keyboardShortcut("o")
                .disabled(model.isBusy)
            Menu("Open Recent") {
                RecentMenuItems(model: model)
                Divider()
                Button("Clear Menu") { model.clearRecent() }
                    .disabled(model.recent.entries.isEmpty)
            }
            Button("Open Project…") { Task { await model.chooseAndOpenProject() } }
                .disabled(model.isBusy)
        }
        CommandGroup(replacing: .saveItem) {
            Button("Save Image…") { Task { await model.chooseAndSaveImage() } }
                .keyboardShortcut("s")
                .disabled(!model.canSaveImage)
            Button("Save Project…") { Task { await model.chooseAndSaveProject() } }
                .disabled(!model.canSaveProject)
        }
        CommandMenu("Run") {
            Button("Cancel") { model.cancelRun() }
                .keyboardShortcut(".")
                .disabled(!model.canCancel)
            Button(model.run?.paused == true ? "Resume" : "Pause") { model.togglePause() }
                .disabled(!model.canCancel)
            Divider()
            Menu("Go to Step") { StepMenuItems(model: model) }
        }
        CommandGroup(after: .toolbar) {
            Divider()
            Button("Zoom In") { model.viewport.zoom(by: 1.25) }
                .keyboardShortcut("+")
                .disabled(model.image == nil)
            Button("Zoom Out") { model.viewport.zoom(by: 1 / 1.25) }
                .keyboardShortcut("-")
                .disabled(model.image == nil)
            Button("Zoom to Fit") { model.viewport.fit() }
                .keyboardShortcut("0")
                .disabled(model.image == nil)
            Button("Actual Size") { model.viewport.showActualSize() }
                .keyboardShortcut("1")
                .disabled(model.image == nil)
            Divider()
            Button("Previous Frame") { model.showNextFrame(by: -1) }
                .keyboardShortcut("[")
                .disabled(!model.hasRecording)
            Button("Next Frame") { model.showNextFrame(by: 1) }
                .keyboardShortcut("]")
                .disabled(!model.hasRecording)
        }
    }
}
