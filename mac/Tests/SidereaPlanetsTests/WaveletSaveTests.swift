import Foundation
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// Where the Wavelets step meets the step Save: the colour and tone values Save image writes, the Do All it runs first, and the sections
/// `wavelet` and `colour` of the project file in the layout of the Windows app.
@MainActor
@Suite struct WaveletSaveTests {
    let backend = FakeBackend()
    let model: AppModel
    let jupiter: URL
    let folder: URL

    init() throws {
        model = AppModel(backend: backend, defaults: try #require(UserDefaults(suiteName: "WaveletSaveTests-\(UUID().uuidString)")))
        folder = FileManager.default.temporaryDirectory.appendingPathComponent("planets-wavelet-save-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        jupiter = folder.appendingPathComponent("JUPITER.AVI")
        try Data().write(to: jupiter)
    }

    private func eventually(_ condition: () async -> Bool) async -> Bool {
        for _ in 0..<400 {
            if await condition() { return true }
            try? await Task.sleep(for: .milliseconds(5))
        }
        return false
    }

    private func stack(_ model: AppModel) async {
        await model.open([jupiter])
        await model.setAlignpoints()
        await model.runAlign()
        await model.runLimit()
        await model.runStack()
        await model.settleWaveletPreview()
    }

    private func format(_ id: ExportFormat) throws -> FormatInfo {
        try #require(model.formats(for: .wavelet).first { $0.id == id })
    }

    // MARK: Save image

    @Test func theSharpenedImageIsWrittenWithTheColourAndToneValues() async throws {
        await stack(model)
        model.tone.gamma = 1.4
        model.tone.red = 120
        model.tone.black = 10
        model.tone.contrast = 300
        model.tone.brightness = -5
        await model.doAll()
        let dialogs = Dialogs(answer: folder.appendingPathComponent("a.tif"))
        await model.saveImage(try format(.tiff16), choose: dialogs.save)

        let saved = try #require(await backend.imageSaves.first)
        #expect(saved.source == .wavelet)
        let adjustments = try #require(saved.adjustments)
        #expect(adjustments.gamma == 1.4 && adjustments.red == 120 && adjustments.black == 10)
        #expect(adjustments.contrast == 120 && adjustments.brightness == -5)
        #expect(model.status.result == "Saved a.tif")
    }

    @Test func neutralValuesAndTheStackStepWriteTheImageAsItIs() async throws {
        await stack(model)
        await model.doAll()
        #expect(model.saveAdjustments == nil)
        await model.saveImage(try format(.tiff16), choose: Dialogs(answer: folder.appendingPathComponent("a.tif")).save)
        #expect(await backend.imageSaves.last?.adjustments == nil)

        // The stack is saved as it is, whatever the cards say.
        model.tone.gamma = 2
        model.show(.stack)
        #expect(model.saveAdjustments == nil)
        let stackFormat = try #require(model.formats(for: .stack).first { $0.id == .tiff16 })
        await model.saveImage(stackFormat, choose: Dialogs(answer: folder.appendingPathComponent("b.tif")).save)
        #expect(await backend.imageSaves.last?.source == .stack)
        #expect(await backend.imageSaves.last?.adjustments == nil)
        await model.settleColours()
    }

    // MARK: Do All before saving

    @Test func doAllRunsOnlyWhenTheValuesChangedSinceTheLastOne() async throws {
        await stack(model)
        await model.doAll()
        #expect(backend.wavelets.doAlls.count == 1)
        model.show(.save)
        #expect(await model.ensureProcessedImage())
        #expect(backend.wavelets.doAlls.count == 1, "the result is current")

        model.moveLevel(2, to: 40)
        #expect(!model.doAllIsCurrent)
        #expect(await model.ensureProcessedImage())
        #expect(backend.wavelets.doAlls.count == 2)
        #expect(backend.wavelets.doAlls.last?.settings.layers[2].gain == 4)
        #expect(model.doAllIsCurrent)
    }

    @Test func aCancelledDoAllSavesNothingAndOpensNoDialog() async throws {
        await stack(model)
        backend.wavelets.holdsDoAll = true
        let dialogs = Dialogs(answer: folder.appendingPathComponent("a.tif"))
        let tiff = try format(.tiff16)
        let running = Task { await model.saveImage(tiff, choose: dialogs.save) }
        #expect(await eventually { model.canCancel })
        model.cancelRun()
        await running.value

        #expect(model.status.result == "Cancelled")
        #expect(dialogs.saveRequests.isEmpty && backend.log.cancelledOperations == [.wavelet])
        #expect(await backend.imageSaves.isEmpty && !model.progress.saved)
    }

    // MARK: Project file

    @Test func theProjectFileHasTheSectionsWaveletAndColourOfTheWindowsApp() async throws {
        model.wavelet.colour = false
        model.wavelet.lrgb = true
        model.wavelet.values.scheme = .dyadic
        model.wavelet.values.layers[1].denoise = 1.25
        model.moveLevel(1, to: 32)
        model.tone.shifts.blue = ChannelShift(x: -1.5, y: 0.5)
        model.tone.histogramView = .result
        let sections = model.projectSections

        #expect(sections["colour"] == .object(["colour": .bool(false), "lrgb": .bool(true)]))
        guard case .object(let wavelet)? = sections["wavelet"] else { Issue.record("no wavelet section"); return }
        #expect(Set(wavelet.keys) == [
            "showFullImage", "showAlignpoints", "showProcessingArea", "automatic", "hold", "scheme", "initialLayer", "stepIncrement", "filter",
            "linked", "layers", "functionsOpen", "functions", "contrast", "brightness", "holdContrast", "tone",
        ])
        #expect(wavelet["scheme"] == .string("dyadic") && wavelet["filter"] == .string("gaussian"))
        guard case .array(let layers)? = wavelet["layers"], layers.count == 6 else { Issue.record("no six layers"); return }
        #expect(layers[1] == .object(["enabled": .bool(true), "denoise": .number(1.25), "sharpen": .number(0.1), "level": .number(32)]))
        #expect(wavelet["contrast"] == .number(250) && wavelet["brightness"] == .number(0))
        guard case .object(let tone)? = wavelet["tone"] else { Issue.record("no tone"); return }
        #expect(Set(tone.keys) == ["shifts", "maxShift", "mix", "red", "green", "blue", "black", "white", "clip", "histogramView", "gamma"])
        #expect(tone["histogramView"] == .string("result") && tone["gamma"] == .number(1) && tone["clip"] == .number(0.1))
        #expect(tone["shifts"] == .object([
            "red": .object(["x": .number(0), "y": .number(0)]), "green": .object(["x": .number(0), "y": .number(0)]),
            "blue": .object(["x": .number(-1.5), "y": .number(0.5)]),
        ]))
    }

    @Test func aSavedProjectBringsTheValuesBackAndTheFirstStackKeepsThem() async throws {
        await model.open([jupiter])
        model.wavelet.colour = false
        model.wavelet.hold = true
        model.wavelet.linked = true
        model.wavelet.functions = ["Gamma", "Histogram"]
        model.wavelet.values.filter = .default
        model.wavelet.values.initialLayer = 3
        model.wavelet.values.layers[0].enabled = false
        model.wavelet.values.layers[3].sharpen = 0.35
        model.wavelet.values.layers[3].level = -20
        model.tone.gamma = 1.25
        model.tone.black = 8
        model.tone.shifts.red = ChannelShift(x: 0.5, y: -0.25)
        model.tone.contrast = 400
        model.tone.holdContrast = true
        let file = folder.appendingPathComponent("w.rkproj")
        await model.saveProject(choose: Dialogs(answer: file).save)

        let other = AppModel(backend: FakeBackend(), defaults: try #require(UserDefaults(suiteName: "WaveletSaveTests-\(UUID().uuidString)")))
        await other.openProject(choose: Dialogs(answer: file).open)
        #expect(!other.wavelet.colour && other.wavelet.hold && other.wavelet.linked)
        #expect(other.wavelet.functions == ["Gamma", "Histogram"])
        #expect(other.wavelet.values.filter == .default && other.wavelet.values.initialLayer == 3)
        #expect(other.wavelet.values.layers[0].enabled == false)
        #expect(other.wavelet.values.layers[3].level == -20 && abs(other.wavelet.values.layers[3].sharpen - 0.35) < 1e-9)
        #expect(other.tone.gamma == 1.25 && other.tone.black == 8 && other.tone.contrast == 400 && other.tone.holdContrast)
        #expect(other.tone.shifts.red == ChannelShift(x: 0.5, y: -0.25))

        // The Wavelets step is locked until Stack: the values of the file survive the first stack, Hold or not.
        other.wavelet.hold = false
        await stack(other)
        #expect(other.wavelet.values.layers[3].level == -20 && other.wavelet.values.layers[0].enabled == false)
        other.show(.stack)
        await other.runStack()
        #expect(other.wavelet.values.layers[3].level == 10, "the second stack starts the layers over")
        await other.settleWaveletPreview()
        await other.settleColours()
    }

    @Test func aWindowsProjectFileIsReadKeyByKeyAndWrongTypesAreIgnored() async throws {
        let file = folder.appendingPathComponent("windows.rkproj")
        let json = """
            {"formatVersion": 1, "recording": ["\(jupiter.path)"], "sections": {
              "wavelet": {"automatic": true, "hold": "yes", "scheme": "dyadic", "filter": "nonsense", "initialLayer": 2, "stepIncrement": 1,
                "layers": [{"enabled": false, "denoise": 0.5, "sharpen": 0.2, "level": 32}, {"level": 55}, {"level": "high"}],
                "functions": ["RGB Align", 5], "contrast": 300, "brightness": -40,
                "tone": {"shifts": {"red": {"x": 1.5, "y": -2}}, "maxShift": 7, "red": 90, "black": 12, "white": 240, "clip": 0.5,
                  "histogramView": "result", "gamma": 1.5, "green": "x"}},
              "colour": {"colour": false, "lrgb": true}}}
            """
        try json.write(to: file, atomically: true, encoding: .utf8)
        await model.openProject(choose: Dialogs(answer: file).open)

        #expect(model.wavelet.automatic && !model.wavelet.hold, "a string is not a boolean")
        #expect(model.wavelet.values.scheme == .dyadic && model.wavelet.values.filter == .gaussian, "an unknown name keeps the value")
        #expect(model.wavelet.values.initialLayer == 2 && model.wavelet.values.stepIncrement == 1)
        let layers = model.wavelet.values.layers
        #expect(layers[0] == WaveletLayer(enabled: false, denoise: 0.5, sharpen: 0.2, level: 32))
        #expect(layers[1].level == 55 && layers[2].level == 10 && layers[3] == WaveletValues.defaults.layers[3])
        #expect(model.wavelet.functions == ["RGB Align"])
        #expect(model.tone.contrast == 300 && model.tone.brightness == -40)
        #expect(model.tone.shifts.red == ChannelShift(x: 1.5, y: -2) && model.tone.shifts.green == ToneValues.noShifts.green)
        #expect(model.tone.maxShift == 7 && model.tone.red == 90 && model.tone.green == 100 && model.tone.black == 12 && model.tone.white == 240)
        #expect(model.tone.clip == 0.5 && model.tone.histogramView == .result && model.tone.gamma == 1.5)
        #expect(!model.wavelet.colour && model.wavelet.lrgb)
        #expect(model.status.result == "Opened windows.rkproj")
    }
}
