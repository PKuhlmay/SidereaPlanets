import AppKit
import SwiftUI

/// The Help menu (Siderea Planets Help, ⌘?) and About in the app menu. Settings… (⌘,) comes with the `Settings` scene.
struct HelpCommands: Commands {
    @Environment(\.openWindow) private var openWindow

    var body: some Commands {
        CommandGroup(replacing: .help) {
            Button("Siderea Planets Help") { openWindow(id: HelpWindow.id) }
                .keyboardShortcut("?", modifiers: .command)
        }
        CommandGroup(replacing: .appInfo) {
            Button("About Siderea Planets") { Self.showAbout() }
        }
    }

    /// The standard About panel (name, version and copyright come from the Info.plist) with a short credit.
    static func showAbout() {
        let credits = NSAttributedString(
            string: "A rebuild of the workflow of the reference program for stacking and sharpening planetary and lunar recordings. "
                + "Not affiliated with the original.",
            attributes: [.font: NSFont.systemFont(ofSize: NSFont.smallSystemFontSize), .foregroundColor: NSColor.secondaryLabelColor]
        )
        NSApp.orderFrontStandardAboutPanel(options: [.credits: credits])
        NSApp.activate()
    }
}
