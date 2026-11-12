import SwiftUI

/// The window (docs/design.md, "Fensteraufbau"): step bar, left panel with the image area, frame bar and status bar under the
/// native toolbar. Dark and light follow macOS.
struct ContentView: View {
    let model: AppModel

    var body: some View {
        VStack(spacing: 0) {
            StepBar(model: model)
            HStack(spacing: 0) {
                StepPanel(model: model) { Task { await model.select() } }
                ImageArea(model: model)
                if model.viewStep == .wavelets && model.wavelet.functionsOpen { FunctionsPanel(model: model) }
            }
            if model.showsFrameBar {
                FrameBar(model: model)
            }
            StatusBar(status: model.status)
        }
        .background(RK.bg)
        .frame(minWidth: RKMetrics.minWindowSize.width, minHeight: RKMetrics.minWindowSize.height)
        .tint(RK.accent)
        .navigationTitle(model.windowTitle)
        .toolbar { AppToolbar(model: model) }
        .dropDestination(for: URL.self) { urls, _ in
            Task { await model.open(urls) }
            return !urls.isEmpty
        }
        #if DEBUG
        .task { await model.applyLaunchArguments() }
        #endif
    }
}

/// The toolbar: Select and MRU at the left, Cancel and Pause in the middle (they act on a running Align; Cancel is ⌘. in the
/// Process menu), Open project and Save project. Help and CPUs follow with their tickets.
struct AppToolbar: ToolbarContent {
    let model: AppModel

    var body: some ToolbarContent {
        ToolbarItemGroup(placement: .navigation) {
            Button("Select Recording…") { Task { await model.select() } }
                .disabled(model.isBusy)
                .help("Select a recording: a video or a series of images (⌘O)")
            Menu("MRU") {
                RecentMenuItems(model: model)
            }
            .help("Recently opened recordings")
            // Not in the original: the project file keeps the recording and the settings of a session (docs/formats.md).
            Button { Task { await model.chooseAndOpenProject() } } label: { Label("Open", systemImage: "folder") }
                .labelStyle(.titleAndIcon)
                .disabled(model.isBusy)
                .help("Open project")
            Button { Task { await model.chooseAndSaveProject() } } label: { Label("Save", systemImage: "square.and.arrow.down") }
                .labelStyle(.titleAndIcon)
                .disabled(!model.canSaveProject)
                .help("Save project")
        }
        ToolbarItemGroup(placement: .principal) {
            CancelButton(model: model)
            Button(model.run?.paused == true ? "Resume" : "Pause") { model.togglePause() }
                .disabled(!model.canCancel)
                .help("Hold the calculation; press again to go on")
        }
    }
}

/// The recent recordings as menu items (toolbar MRU and File > Open Recent): newest first, the files of a sequence
/// as "name (+n)", with the folder as tooltip.
struct RecentMenuItems: View {
    let model: AppModel

    var body: some View {
        if model.recent.entries.isEmpty {
            Text("No recent files")
        }
        ForEach(model.recent.entries, id: \.self) { paths in
            Button(RecentList.label(for: paths)) {
                Task { await model.open(paths.map { URL(fileURLWithPath: $0) }) }
            }
            .disabled(model.isBusy)
            .help(paths.joined(separator: "\n"))
        }
    }
}

/// Cancel is filled in the accent colour while a calculation runs (docs/design.md, "Während einer Berechnung").
private struct CancelButton: View {
    let model: AppModel

    var body: some View {
        if model.canCancel {
            Button("Cancel") { model.cancelRun() }
                .buttonStyle(.borderedProminent)
                .tint(RK.accent)
                .help("Stop the calculation (⌘.)")
        } else {
            Button("Cancel") {}
                .disabled(true)
                .help("Stop the calculation (⌘.)")
        }
    }
}
