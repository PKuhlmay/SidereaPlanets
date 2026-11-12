#if DEBUG
import AppKit
import Foundation
import SidereaPlanetsKit

extension AppModel {
    /// Debug builds only, for screenshots and hand checks. Arguments (AppKit reads `-key value` pairs as defaults):
    ///
    ///     open -a SidereaPlanets.app --args -open /path/to/JUPITER.AVI -appearance light
    ///
    /// `-open` loads a recording at launch (several files separated by a colon), `-appearance light|dark` overrides macOS,
    /// `-run alignpoints,align,limit` presses those buttons one after the other (`roidraw` presses R.o.Interest, `alignpaused` starts Align and pauses it after
    /// the first frame, for a picture of a running Align), `-step open|alignpoints|…` shows that step and `-frame 5` that frame
    /// (1-based) once the recording is open. Settings: `-show framelist,registrationgraph,alignpoints,roi`,
    /// `-roi x,y,width,height` and `-limit lowestQuality|bestPercent|framesPerPoint|bestCount`.
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
        if let list = defaults.string(forKey: "show") {
            let names = list.split(separator: ",").map(String.init)
            show.framelist = names.contains("framelist")
            show.registrationgraph = names.contains("registrationgraph")
            if names.contains("noalignpoints") { show.alignpoints = false }
        }
        if let mode = defaults.string(forKey: "limit").flatMap(LimitMode.init(rawValue:)) {
            options.limitMode = mode
        }
        let box = (defaults.string(forKey: "roi") ?? "").split(separator: ",").compactMap { UInt32($0) }
        if box.count == 4 {
            selectRegion(RoiSettings(x: box[0], y: box[1], width: box[2], height: box[3]))
        }
        for token in (defaults.string(forKey: "run") ?? "").split(separator: ",") {
            switch token {
            case "alignpoints": await setAlignpoints()
            case "align": await runAlign()
            case "alignpaused":
                Task { await runAlign() }
                try? await Task.sleep(for: .milliseconds(1200))
                togglePause()
            case "limit": await runLimit()
            case "roidraw": toggleRegionOfInterest()
            default: break
            }
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
