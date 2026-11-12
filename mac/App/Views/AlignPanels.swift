import SidereaPlanetsKit
import SwiftUI

// The settings of the steps 2 to 4 in the left panel (docs/design.md, "2 Alignpoints", "3 Align", "4 Limit"): the key fields
// first, the rest under Advanced. Values and defaults are those of the Windows app (AlignpointsStep.vue, AlignStep.vue,
// LimitStep.vue); the texts follow MAC-014 (docs/design.md, "Texte und Schreibweise").

/// Step 2: the Threshold slider, the two distances, Draw Region; under Advanced the Point Placement with the brightness range.
struct AlignpointsStepView: View {
    @Bindable var model: AppModel

    private var hasPoints: Bool { model.progress.step >= .points }

    private var threshold: Binding<Double> {
        Binding(get: { Double(model.align.threshold) }, set: { model.setThreshold(Int($0.rounded())) })
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if let best = model.align.bestFrameHint {
                HintNote(text: bestFrameHint(frame: best))
            }
            VStack(spacing: 2) {
                Text(hasPoints ? "Threshold · \(model.align.points.count) points" : "Threshold")
                    .font(.rk(RKMetrics.fontSizeSmall, .semibold))
                    .foregroundStyle(RK.text)
                // No range before Set Alignpoints: the slider stays at 0 and looks locked.
                Slider(value: threshold, in: 0...Double(max(model.align.total, 1)))
                    .controlSize(.small)
                    .tint(RK.accent)
                    .disabled(!model.canMoveThreshold)
                    .accessibilityLabel("Threshold")
                HStack {
                    Text("more points")
                    Spacer()
                    Text("fewer points")
                }
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
            }
            .help("How strong a detail must be to get an alignpoint; left keeps more points, right fewer.")
            FieldRow(
                title: "Min. distance between points (px)", value: $model.options.minDistance,
                help: "Smallest distance between two alignpoints, in pixels."
            )
            .padding(.top, 6)
            FieldRow(
                title: "Min. distance from image edge (px)", value: $model.options.edgeDistance,
                help: "Alignpoints keep at least this distance from the edge of the image or the region, in pixels."
            )
            Button("Draw Region") { model.toggleRegionOfInterest() }
                .buttonStyle(.rkWide(isOn: model.isDrawingRegion))
                .disabled(!model.canDrawRegion)
                .help("Region of interest (ROI): drag a rectangle on the image so that only this area is used for placing alignpoints and aligning; press again to remove it.")
                .accessibilityAddTraits(model.isDrawingRegion ? .isSelected : [])
                .padding(.top, 6)

            AdvancedSection(model: model, step: .alignpoints, count: 5) {
                VStack(alignment: .leading, spacing: 6) {
                    GroupHeading(title: "Point Placement")
                    Text("Brightness measure")
                        .font(.rkSmall)
                        .foregroundStyle(RK.muted)
                        .help("How the brightness at a point is measured when alignpoints are placed.")
                    VStack(alignment: .leading, spacing: 0) {
                        RadioRow(title: "Default", isSelected: model.options.intensity == .default, help: "The centre pixel.") {
                            model.options.intensity = .default
                        }
                        RadioRow(title: "3×3 area", isSelected: model.options.intensity == .area3x3, help: "The mean of the 3×3 pixels around the centre.") {
                            model.options.intensity = .area3x3
                        }
                        RadioRow(title: "Lowest pixel value", isSelected: model.options.intensity == .lowest, help: "The darkest of the 3×3 pixels around the centre.") {
                            model.options.intensity = .lowest
                        }
                    }
                    FieldRow(
                        title: "Min. brightness (0–255)", value: $model.options.lo,
                        help: "Only pixels between Min. and Max. brightness (0–255) can get an alignpoint."
                    )
                    FieldRow(
                        title: "Max. brightness (0–255)", value: $model.options.hi,
                        help: "Only pixels between Min. and Max. brightness (0–255) can get an alignpoint."
                    )
                    HStack(spacing: 8) {
                        Text("Number of alignpoints").font(.rk).lineLimit(1)
                        Spacer(minLength: 4)
                        ReadOnlyField(label: "Number of alignpoints", value: model.align.points.count)
                    }
                    PanelCheckbox(title: "Keep points inside the scan frame", isOn: .constant(false), isLocked: true)
                }
            }
        }
    }
}

/// Step 3: the two sizes; under Advanced the locked alignment modes and the rest of Alignment Setup.
struct AlignStepView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            FieldRow(
                title: "Box size per point (px)", value: $model.options.boxSize, range: 1...1_000_000,
                help: "Edge length of the square around each alignpoint that is searched for in the other frames, in pixels."
            )
            FieldRow(
                title: "Max. point shift (px)", value: $model.options.maxMovement,
                help: "How far an alignpoint may move from one frame to the next, in pixels."
            )

            AdvancedSection(model: model, step: .align, count: 5) {
                VStack(alignment: .leading, spacing: 6) {
                    GroupHeading(title: "Alignment Setup")
                    Text("Alignment mode").font(.rkSmall).foregroundStyle(RK.muted)
                    LockedSegments(first: "No Align", second: "Scan Frames")
                    Text("Not available yet").font(.rkSmall).foregroundStyle(RK.muted)
                    PanelCheckbox(title: "Show alignment", isOn: .constant(false), isLocked: true)
                    PanelCheckbox(
                        title: "Align by center of gravity", isOn: $model.options.centreOfGravity,
                        help: "Follows the planet by the centre of its brightness, however far it jumps between frames. For a whole planet against a dark sky; leave it off for the Moon or the Sun in close-up."
                    )
                    let gravityOn = model.options.centreOfGravity
                    HStack(spacing: 8) {
                        Text("Brightness threshold").font(.rk).foregroundStyle(gravityOn ? RK.text : RK.off)
                        Spacer(minLength: 4)
                        NumberField(label: "Brightness threshold", value: $model.options.brightnessThreshold, range: 0...255)
                            .disabled(!gravityOn)
                            .opacity(gravityOn ? 1 : 0.45)
                    }
                    .padding(.leading, 20)
                    .help(gravityOn
                        ? "Pixels up to this brightness (0–255) do not count towards the centre: the sky and its noise. Raise it if a moon or a bright edge pulls the planet off."
                        : "Turn on Align by center of gravity to use it.")
                    // Locked because the way it works is not known (docs/ui.md).
                    PanelCheckbox(title: "Estimate rotation", isOn: .constant(false), isLocked: true)
                }
            }
        }
    }
}

/// Two joined buttons that do nothing yet (docs/design.md, "Segmente"); the first looks like the selected one.
private struct LockedSegments: View {
    let first: String
    let second: String

    var body: some View {
        HStack(spacing: 0) {
            Text(first)
                .font(.rk(RKMetrics.fontSize, .medium))
                .foregroundStyle(RK.text)
                .frame(maxWidth: .infinity, minHeight: RKMetrics.buttonHeight)
                .background(RK.raised)
            RK.line.frame(width: 1)
            Text(second)
                .font(.rk)
                .foregroundStyle(RK.off)
                .frame(maxWidth: .infinity, minHeight: RKMetrics.buttonHeight)
        }
        .clipShape(RoundedRectangle(cornerRadius: RKMetrics.radius))
        .overlay { RoundedRectangle(cornerRadius: RKMetrics.radius).strokeBorder(RK.lineStrong, lineWidth: 1) }
        .help("Not available yet")
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(first), \(second)")
        .accessibilityValue("not available yet")
    }
}

/// Step 4: the whole Limit Setup, and under it the quality of the frames.
struct LimitStepView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // The one group of the step, Limit Setup, without its heading (docs/design.md, "Schrittpanel").
            limitRow(.lowestQuality, value: $model.options.limitPercent, label: "Limit percent")
            limitRow(.bestPercent, value: nil, label: "")
            limitRow(.framesPerPoint, value: $model.options.limitCount, label: "Limit frame count")
            limitRow(.bestCount, value: nil, label: "")
            QualityChartView(frames: model.align.frames, options: model.options)
                .padding(.top, 4)
            GoToBestFrameButton(model: model)
        }
    }

    /// A radio button; the field of its kind sits in its row, locked while the other kind of mode is chosen.
    private func limitRow(_ mode: LimitMode, value: Binding<Int>?, label: String) -> some View {
        HStack(spacing: 8) {
            RadioRow(title: mode.label, isSelected: model.options.limitMode == mode, help: mode.explanation) { model.options.limitMode = mode }
            Spacer(minLength: 4)
            if let value {
                let locked = model.options.limitMode.countsFrames != mode.countsFrames
                NumberField(label: label, value: value, range: mode.countsFrames ? 1...1_000_000 : 0...1_000_000)
                    .disabled(locked)
                    .opacity(locked ? 0.45 : 1)
            }
        }
        .frame(minHeight: value == nil ? 0 : RKMetrics.fieldHeight)
    }
}

/// A note in a step panel (docs/design.md, "Hinweis"): `--accent-soft`, a 3 pt strip in `--accent` at the left.
private struct HintNote: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.rkSmall)
            .foregroundStyle(RK.text)
            .lineSpacing(2)
            .fixedSize(horizontal: false, vertical: true)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.vertical, 8)
            .padding(.leading, 13)
            .padding(.trailing, 10)
            .background(RK.accentSoft, in: RoundedRectangle(cornerRadius: RKMetrics.radius))
            .overlay(alignment: .leading) {
                UnevenRoundedRectangle(topLeadingRadius: RKMetrics.radius, bottomLeadingRadius: RKMetrics.radius)
                    .fill(RK.accent)
                    .frame(width: 3)
            }
            .accessibilityElement(children: .combine)
    }
}

/// Go to Best Frame: under the quality chart. Locked until Align has run.
private struct GoToBestFrameButton: View {
    let model: AppModel

    var body: some View {
        Button("Go to Best Frame") { model.goToBestFrame() }
            .buttonStyle(.rkWide)
            .disabled(!model.canGoToBestFrame)
            .help(model.progress.step >= .aligned ? goToBestFrameHelp : "Run Align first")
    }
}

/// The quality of the frames in the Limit step (docs/design.md, "4 Limit"): one bar per frame, best first, the kept frames in the
/// accent colour, the limit as a dashed line. Follows every change of the Limit Setup at once.
struct QualityChartView: View {
    let frames: [FrameEntry]
    let options: AlignOptions

    private let height: CGFloat = 76
    private let padding = (left: 3.0, right: 3.0, top: 12.0, bottom: 3.0)

    var body: some View {
        VStack(spacing: 2) {
            Canvas { context, size in draw(&context, size: size) }
                .frame(height: height)
                .background(RK.field, in: RoundedRectangle(cornerRadius: RKMetrics.radiusSmall))
                .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusSmall).strokeBorder(RK.line, lineWidth: 1) }
                .accessibilityElement()
                .accessibilityLabel(qualityChart(frames: frames, options: options, maxBars: 240).label)
            HStack {
                Text("sharpest")
                Spacer()
                Text("blurriest")
            }
            .font(.rk(10))
            .foregroundStyle(RK.muted)
        }
    }

    private func draw(_ context: inout GraphicsContext, size: CGSize) {
        let plotWidth = size.width - padding.left - padding.right
        let plotHeight = size.height - padding.top - padding.bottom
        let chart = qualityChart(frames: frames, options: options, maxBars: Int(plotWidth))
        guard !chart.bars.isEmpty else { return }

        let step = plotWidth / Double(chart.bars.count)
        let gap = step >= 3 ? 1.0 : 0.0
        for (i, bar) in chart.bars.enumerated() {
            let h = max(1, bar.height * plotHeight)
            let rect = CGRect(x: padding.left + Double(i) * step, y: padding.top + plotHeight - h, width: max(0.5, step - gap), height: h)
            context.fill(Path(rect), with: .color(bar.kept ? RK.accent : RK.lineStrong))
        }

        guard let line = chart.line else { return }
        var path = Path()
        let anchor: Alignment
        let at: CGPoint
        switch line {
        case .level(let share):
            let y = padding.top + plotHeight * (1 - share)
            path.move(to: CGPoint(x: padding.left, y: y))
            path.addLine(to: CGPoint(x: size.width - padding.right, y: y))
            anchor = .trailing
            at = CGPoint(x: size.width - padding.right, y: y - 7)
        case .after(let share):
            let x = padding.left + plotWidth * share
            path.move(to: CGPoint(x: x, y: padding.top - 2))
            path.addLine(to: CGPoint(x: x, y: size.height - padding.bottom))
            let right = x < size.width - 40
            anchor = right ? .leading : .trailing
            at = CGPoint(x: right ? x + 4 : x - 4, y: padding.top - 8)
        }
        context.stroke(path, with: .color(RK.text), style: StrokeStyle(lineWidth: 1, dash: [3, 3]))
        let value = context.resolve(Text(chart.value).font(.rk(10)).foregroundStyle(RK.muted))
        context.draw(value, at: at, anchor: anchor == .trailing ? .trailing : .leading)
    }
}
