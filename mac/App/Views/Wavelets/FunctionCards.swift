import SidereaPlanetsKit
import SwiftUI

/// The cards behind the Functions buttons of step 6 (docs/design.md, "Karten hinter den Functions-Schaltflächen"): top right over the
/// image, in the order the buttons were pressed, 236 pt wide, not movable. They set values; the image follows every change.
struct FunctionCards: View {
    @Bindable var model: AppModel

    var body: some View {
        ScrollView(.vertical) {
            VStack(spacing: 10) {
                ForEach(model.openCards, id: \.self) { name in
                    switch name {
                    case "Histogram": HistogramCard(model: model)
                    case "Gamma": GammaCard(model: model)
                    case "Color Mixing": ColorMixingCard(model: model)
                    case "RGB Align": RgbAlignCard(model: model)
                    case "Denoise/Deringing": DenoiseCard(model: model)
                    case "Masking": MaskCard(model: model)
                    default: RgbBalanceCard(model: model)
                    }
                }
            }
            .padding(.bottom, 12)
        }
        .scrollIndicators(.never)
        .frame(width: 236)
        .padding(.top, 48)
        .padding(.trailing, 12)
        .padding(.bottom, 44)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
    }
}

// MARK: Histogram

/// The histogram of the image and the black and white point of the stretch (markers below the graph, draggable).
private struct HistogramCard: View {
    @Bindable var model: AppModel
    @State private var data: HistogramData?
    @State private var dragged: Marker?
    @State private var canvasWidth = 216.0

    private enum Marker { case black, white }

    private let width = 212.0
    private let height = 84.0
    private let markerBand = 12.0

    private var key: HistogramKey {
        HistogramKey(model.tone.core, after: model.tone.histogramView == .result, base: model.waveletData.baseVersion)
    }

    var body: some View {
        FloatCard(title: "Histogram", close: { model.closeFunction("Histogram") }, actions: {
            LinkButton("Reset") {
                model.tone.black = ToneLimits.level.lowerBound
                model.tone.white = ToneLimits.level.upperBound
            }
        }, content: {
            VStack(spacing: 6) {
                graph
                SegmentBar(items: [
                    SegmentItem(title: "Input", isOn: model.tone.histogramView == .input) { model.tone.histogramView = .input },
                    SegmentItem(title: "Result", isOn: model.tone.histogramView == .result) { model.tone.histogramView = .result },
                ])
                HStack(alignment: .top, spacing: 12) {
                    VStack(alignment: .leading, spacing: 3) {
                        Text("Black point").font(.rkSmall).foregroundStyle(RK.muted)
                        NumberField(label: "Black point", value: $model.tone.black, range: 0...max(model.tone.white - 1, 0), width: 62)
                    }
                    VStack(alignment: .leading, spacing: 3) {
                        Text("White point").font(.rkSmall).foregroundStyle(RK.muted)
                        NumberField(label: "White point", value: $model.tone.white, range: min(model.tone.black + 1, 255)...255, width: 62)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                HStack(spacing: 8) {
                    Button("Auto") { Task { await model.autoLevels() } }
                        .buttonStyle(.rk)
                    Text("Clip (%)").font(.rkSmall).foregroundStyle(RK.muted)
                        .help("Share of the darkest and brightest pixels that Auto may cut off.")
                    DecimalField(label: "Clip percent", value: $model.tone.clip, decimals: 1, step: 0.1, range: ToneLimits.clip, width: 40)
                    Spacer(minLength: 0)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding(10)
        })
        // One request at a time, the newest answer wins; moving a field or marker waits a moment before asking.
        .task(id: key) {
            try? await Task.sleep(for: .milliseconds(80))
            guard !Task.isCancelled else { return }
            data = await model.histogram(after: key.after)
        }
    }

    private var graph: some View {
        Canvas { context, size in
            let k = size.width / width
            context.scaleBy(x: k, y: k)
            let plot = CGRect(x: 0, y: 0, width: width, height: height)
            context.fill(Path(plot), with: .color(RK.field))
            if let data {
                let maximum = histogramMax(data)
                context.fill(
                    outline(data.luma, maximum: maximum, closed: true), with: .color(RK.lineStrong)
                )
                // A grayscale image has one channel; it is drawn like the luminance, in the muted text colour.
                let colours = [RK.channelRed, RK.channelGreen, RK.channelBlue]
                for (index, counts) in data.channels.enumerated() {
                    context.stroke(
                        outline(counts, maximum: maximum, closed: false),
                        with: .color(data.channels.count == 3 ? colours[index] : RK.muted), lineWidth: 1
                    )
                }
            }
            context.stroke(Path(plot), with: .color(RK.line), lineWidth: 1)
            for (level, marker) in [(model.tone.black, Marker.black), (model.tone.white, .white)] {
                let x = levelToX(level, width: width)
                var triangle = Path()
                triangle.move(to: CGPoint(x: x - 6, y: height + markerBand))
                triangle.addLine(to: CGPoint(x: x + 6, y: height + markerBand))
                triangle.addLine(to: CGPoint(x: x, y: height + 2))
                triangle.closeSubpath()
                context.fill(triangle, with: .color(dragged == marker ? RK.accent : RK.text))
            }
        }
        .aspectRatio(width / (height + markerBand), contentMode: .fit)
        .onGeometryChange(for: Double.self, of: { Double($0.size.width) }) { canvasWidth = max($0, 1) }
        .gesture(
            DragGesture(minimumDistance: 0, coordinateSpace: .local)
                .onChanged { drag in moveMarker(drag) }
                .onEnded { _ in dragged = nil }
        )
        .accessibilityElement()
        .accessibilityLabel("Histogram")
        .help("Drag the triangles to set the black and white point")
    }

    private func outline(_ counts: [UInt32], maximum: Double, closed: Bool) -> Path {
        let points = histogramPoints(counts, max: maximum, width: width, height: height)
        var path = Path()
        guard let first = points.first else { return path }
        path.move(to: first)
        for point in points.dropFirst() { path.addLine(to: point) }
        if closed {
            path.addLine(to: CGPoint(x: width, y: height))
            path.addLine(to: CGPoint(x: 0, y: height))
            path.closeSubpath()
        }
        return path
    }

    /// Dragging a marker: the pointer position on the graph gives the level. The marker nearest to where the drag started is taken.
    private func moveMarker(_ drag: DragGesture.Value) {
        if dragged == nil {
            let black = abs(drag.startLocation.x / cardScale - levelToX(model.tone.black, width: width))
            let white = abs(drag.startLocation.x / cardScale - levelToX(model.tone.white, width: width))
            dragged = black <= white ? .black : .white
        }
        let level = xToLevel(drag.location.x / cardScale, width: width)
        switch dragged {
        case .black: model.tone.black = moveBlack(level, white: model.tone.white)
        case .white: model.tone.white = moveWhite(level, black: model.tone.black)
        case nil: break
        }
    }

    /// The graph is drawn at a width of `width` and scaled to the card.
    private var cardScale: Double { canvasWidth / width }
}

// MARK: Gamma

/// One slider (0.01 steps) and one field. Above 1.00 the midtones get brighter.
private struct GammaCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Gamma", close: { model.closeFunction("Gamma") }, actions: {
            LinkButton("Reset") { model.tone.gamma = 1 }
        }, content: {
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text("darker")
                    Spacer()
                    Text("brighter")
                }
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
                Slider(
                    value: Binding(get: { (model.tone.gamma * 100).rounded() }, set: { model.tone.gamma = $0.rounded() / 100 }),
                    in: ToneLimits.gamma.lowerBound * 100...ToneLimits.gamma.upperBound * 100, step: 1
                )
                .controlSize(.small)
                .tint(RK.accent)
                .accessibilityLabel("Gamma")
                HStack {
                    Text("Gamma").font(.rk).foregroundStyle(RK.text)
                    Spacer()
                    DecimalField(label: "Gamma value", value: $model.tone.gamma, decimals: 2, step: 0.01, range: ToneLimits.gamma)
                }
                .padding(.top, 2)
            }
            .padding(10)
        })
    }
}

// MARK: RGB Align

/// The shift of each channel in pixels. Auto fills in red and blue so that they lie on green.
private struct RgbAlignCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "RGB Align", close: { model.closeFunction("RGB Align") }, actions: {
            LinkButton("Auto", isEnabled: !model.isMono) { Task { await model.autoAlign() } }
            LinkButton("Reset", isEnabled: !model.isMono) { model.tone.shifts = ToneValues.noShifts }
        }, content: {
            VStack(alignment: .leading, spacing: 4) {
                Grid(alignment: .trailing, horizontalSpacing: 6, verticalSpacing: 2) {
                    GridRow {
                        Text("")
                        Text("X (px)").frame(width: 70, alignment: .trailing)
                        Text("Y (px)").frame(width: 70, alignment: .trailing)
                    }
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                    channel("Red", x: shiftBinding(\.red, \.x), y: shiftBinding(\.red, \.y))
                    channel("Green", x: shiftBinding(\.green, \.x), y: shiftBinding(\.green, \.y))
                    channel("Blue", x: shiftBinding(\.blue, \.x), y: shiftBinding(\.blue, \.y))
                }
                HStack {
                    Text("Max shift (px)").font(.rk).foregroundStyle(RK.text)
                    Spacer()
                    NumberField(label: "Max shift in pixels", value: $model.tone.maxShift, range: ToneLimits.maxShift, width: 40)
                }
                .padding(.top, 4)
                Text("Auto moves Red and Blue onto Green").font(.rkSmall).foregroundStyle(RK.muted)
            }
            .padding(10)
            .disabled(model.isMono)
            .opacity(model.isMono ? 0.45 : 1)
            .help(model.isMono ? "Needs a color image" : "")
        })
    }

    private func channel(_ name: String, x: Binding<Double>, y: Binding<Double>) -> some View {
        GridRow {
            Text(name).font(.rk).foregroundStyle(RK.text).gridColumnAlignment(.leading)
            DecimalField(label: "\(name) X", value: x, decimals: 2, step: 0.1, range: -ToneLimits.shift...ToneLimits.shift, width: 40)
            DecimalField(label: "\(name) Y", value: y, decimals: 2, step: 0.1, range: -ToneLimits.shift...ToneLimits.shift, width: 40)
        }
    }

    private func shiftBinding(_ channel: WritableKeyPath<ChannelShifts, ChannelShift>, _ axis: WritableKeyPath<ChannelShift, Float>) -> Binding<Double> {
        Binding(
            get: { Double(model.tone.shifts[keyPath: channel][keyPath: axis]) },
            set: { model.tone.shifts[keyPath: channel][keyPath: axis] = Float($0) }
        )
    }
}

// MARK: RGB Balance

/// A slider and a field per channel, in percent. The channels are told apart by their labels.
private struct RgbBalanceCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "RGB Balance", close: { model.closeFunction("RGB Balance") }, actions: {
            LinkButton("Reset", isEnabled: !model.isMono) {
                model.tone.red = 100
                model.tone.green = 100
                model.tone.blue = 100
            }
        }, content: {
            VStack(spacing: 4) {
                channel("Red", value: $model.tone.red)
                channel("Green", value: $model.tone.green)
                channel("Blue", value: $model.tone.blue)
            }
            .padding(10)
            .disabled(model.isMono)
            .opacity(model.isMono ? 0.45 : 1)
            .help(model.isMono ? "Needs a color image" : "")
        })
    }

    private func channel(_ name: String, value: Binding<Int>) -> some View {
        HStack(spacing: 8) {
            Text(name).font(.rk).foregroundStyle(RK.text).frame(width: 40, alignment: .leading)
            Slider(
                value: Binding(get: { Double(value.wrappedValue) }, set: { value.wrappedValue = Int($0.rounded()) }),
                in: Double(ToneLimits.balance.lowerBound)...Double(ToneLimits.balance.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(RK.accent)
            .accessibilityLabel(name)
            NumberField(label: "\(name) percent", value: value, range: ToneLimits.balance, width: 36)
            Text("%").font(.rkSmall).foregroundStyle(RK.muted)
        }
        .frame(minHeight: 30)
    }
}
