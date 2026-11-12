#if DEBUG
import AppKit
import Foundation

extension AppModel {
    /// Debug builds only, for screenshots and hand checks. Arguments (AppKit reads `-key value` pairs as defaults):
    ///
    ///     open -a SidereaPlanets.app --args -open /path/to/JUPITER.AVI -appearance light
    ///
    /// `-open` loads a recording at launch (several files separated by a colon), `-appearance light|dark` overrides macOS,
    /// `-step open|alignpoints|…` shows that step and `-frame 5` that frame (1-based) once the recording is open.
    func applyLaunchArguments() async {
        let defaults = UserDefaults.standard
        switch defaults.string(forKey: "appearance") {
        case "light": NSApp.appearance = NSAppearance(named: .aqua)
        case "dark": NSApp.appearance = NSAppearance(named: .darkAqua)
        default: break
        }
        if let paths = defaults.string(forKey: "open") {
            await open(paths.split(separator: ":").map { URL(fileURLWithPath: String($0)) })
        }
        if let name = defaults.string(forKey: "step"), let step = ViewStep.allCases.first(where: { $0.label.lowercased() == name }) {
            show(step)
        }
        if defaults.integer(forKey: "frame") > 0 {
            showFrame(defaults.integer(forKey: "frame") - 1)
        }
    }
}
#endif
