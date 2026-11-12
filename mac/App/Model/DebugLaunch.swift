#if DEBUG
import AppKit
import Foundation
import SidereaPlanetsKit

extension AppModel {
    /// Debug builds only, for screenshots and hand checks. Arguments (AppKit reads `-key value` pairs as defaults):
    ///
    ///     open -a "Siderea Planets.app" --args -open /path/to/JUPITER.AVI -appearance light
    ///
    /// `-open` loads a recording at launch (several files separated by a colon), `-appearance light|dark` overrides macOS,
    /// `-run alignpoints,align,limit,stack` presses those buttons one after the other (`roidraw` presses R.o.Interest, `bestframe` presses Go to best frame, `frame5` shows frame 5 at that point, `alignpaused` starts Align and pauses it after
    /// the first frame, for a picture of a running Align, `stackpaused` the same for Stack), `-step open|alignpoints|…` shows that step and `-frame 5` that frame
    /// (1-based) once the recording is open. Settings: `-show framelist,registrationgraph,alignpoints,roi`,
    /// `-roi x,y,width,height` and `-limit lowestQuality|bestPercent|framesPerPoint|bestCount`.
    /// For the Wavelets step: `-run …,process` and `…,doall` press Process and Do All (`doallpaused` pauses Do All after the first layer),
    /// `-layers 32,22,14,10,10,10` sets the sliders, `-functions Histogram,Gamma` opens cards, `-colour colour|lrgb|gray`,
    /// `-tone black=10,white=240,gamma=1.3,red=110,contrast=300,brightness=20` sets card values, `-before 1` shows Before; `-show` also knows
    /// `processingarea` and `wavealignpoints`. `-advanced wavelets,stack` (or `all`) opens those Advanced sections.
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
            wavelet.showProcessingArea = names.contains("processingarea")
            wavelet.showAlignpoints = names.contains("wavealignpoints")
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
            case "bestframe": goToBestFrame()
            case _ where token.hasPrefix("frame"):
                // "frame5": shows frame 5 (1-based) at that point of the sequence, e.g. before Go to best frame.
                if let number = Int(token.dropFirst(5)) { showFrame(number - 1) }
            case "stack": await runStack()
            case "stackpaused":
                Task { await runStack() }
                try? await Task.sleep(for: .milliseconds(1100))
                togglePause()
            case "process":
                await runProcess()
                await settleColours()
            case "doall":
                await doAll()
                await settleColours()
            case "doallpaused":
                Task { await doAll() }
                try? await Task.sleep(for: .milliseconds(150))
                togglePause()
            case "roidraw": toggleRegionOfInterest()
            default: break
            }
        }
        applyWaveletArguments(defaults)
        if let list = defaults.string(forKey: "advanced") {
            // `-advanced wavelets,stack` (or `all`) opens those Advanced sections, for pictures of the long panels.
            let names = list.split(separator: ",").map { $0.lowercased() }
            openAdvancedForDebug(Set(ViewStep.allCases.filter { names.contains("all") || names.contains($0.label.lowercased()) }))
        }
        if let name = defaults.string(forKey: "step"), let step = ViewStep.allCases.first(where: { $0.label.lowercased() == name }) {
            show(step)
        }
        if defaults.integer(forKey: "frame") > 0 {
            showFrame(defaults.integer(forKey: "frame") - 1)
        }
        waveletData.before = defaults.integer(forKey: "before") > 0
        await settleWaveletPreview()
    }

    /// The Wavelets arguments: values of the layers and cards, applied once the stack is there.
    private func applyWaveletArguments(_ defaults: UserDefaults) {
        let levels = (defaults.string(forKey: "layers") ?? "").split(separator: ",").compactMap { Int($0) }
        for (index, level) in levels.prefix(WaveletLimits.layerCount).enumerated() { wavelet.values.layers[index].level = level }
        if let list = defaults.string(forKey: "functions") { wavelet.functions = list.split(separator: ",").map(String.init) }
        switch defaults.string(forKey: "colour") {
        case "lrgb": wavelet.colour = true; wavelet.lrgb = true
        case "gray": wavelet.colour = false
        case "colour": wavelet.colour = true; wavelet.lrgb = false
        default: break
        }
        for pair in (defaults.string(forKey: "tone") ?? "").split(separator: ",") {
            let parts = pair.split(separator: "=")
            guard parts.count == 2, let value = Double(parts[1]) else { continue }
            switch parts[0] {
            case "black": tone.black = Int(value)
            case "white": tone.white = Int(value)
            case "gamma": tone.gamma = value
            case "red": tone.red = Int(value)
            case "green": tone.green = Int(value)
            case "blue": tone.blue = Int(value)
            case "contrast": tone.contrast = Int(value)
            case "brightness": tone.brightness = Int(value)
            default: break
            }
        }
    }
}
#endif
