import SidereaPlanetsKit
import SwiftUI

// The settings of the steps 2 to 4 in the left panel (docs/design.md, "2 Alignpoints", "3 Align", "4 Limit"): the key fields
// first, the rest under Advanced. Values, defaults and locked options are those of the Windows app (AlignpointsStep.vue,
// AlignStep.vue, LimitStep.vue).

/// Step 2: the Threshold slider, the two distances, R.o.Interest; under Advanced the Intensity selection with Lo and Hi.
struct AlignpointsStepView: View {
    @Bindable var model: AppModel

    private var hasPoints: Bool { model.progress.step >= .points }

    private var threshold: Binding<Double> {
        Binding(get: { Double(model.align.threshold) }, set: { model.setThreshold(Int($0.rounded())) })
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            VStack(spacing: 2) {
                HStack {
                    Text("weakest").foregroundStyle(RK.muted)
                    Spacer()
                    Text(hasPoints ? "Alignpoints: \(model.align.points.count)" : "Threshold")
                        .font(.rk(RKMetrics.fontSizeSmall, .semibold))
                        .foregroundStyle(RK.text)
                    Spacer()
                    Text("strongest").foregroundStyle(RK.muted)
                }
                .font(.rkSmall)
                // No range before Set Alignpoints: the slider stays at 0 and looks locked.
                Slider(value: threshold, in: 0...Double(max(model.align.total, 1)))
                    .controlSize(.small)
                    .tint(RK.accent)
                    .disabled(!model.canMoveThreshold)
                    .accessibilityLabel("Threshold")
            }
            FieldRow(title: "Minimum distance between", value: $model.options.minDistance).padding(.top, 6)
            FieldRow(title: "Min distance from edge", value: $model.options.edgeDistance)
            Button("R.o.Interest") { model.toggleRegionOfInterest() }
                .buttonStyle(.rkWide(isOn: model.isDrawingRegion))
                .disabled(!model.canDrawRegion)
                .help("Draw the region that alignment is based on; press again to drop it")
                .accessibilityAddTraits(model.isDrawingRegion ? .isSelected : [])
                .padding(.top, 6)

            AdvancedSection(model: model, step: .alignpoints, count: 5) {
                VStack(alignment: .leading, spacing: 6) {
                    GroupHeading(title: "Set Alignpoint parameters")
                    Text("Intensity selection").font(.rkSmall).foregroundStyle(RK.muted)
                    HStack(alignment: .top, spacing: 12) {
                        VStack(alignment: .leading, spacing: 0) {
                            RadioRow(title: "Default", isSelected: model.options.intensity == .default) { model.options.intensity = .default }
                            RadioRow(title: "3x3 area", isSelected: model.options.intensity == .area3x3) { model.options.intensity = .area3x3 }
                            RadioRow(title: "Lowest pixelvalue", isSelected: model.options.intensity == .lowest) { model.options.intensity = .lowest }
                        }
                        Spacer(minLength: 0)
                        VStack(spacing: 6) {
                            HStack(spacing: 8) {
                                Text("Lo").font(.rk)
                                NumberField(label: "Lo", value: $model.options.lo)
                            }
                            HStack(spacing: 8) {
                                Text("Hi").font(.rk)
                                NumberField(label: "Hi", value: $model.options.hi)
                            }
                        }
                    }
                    HStack(spacing: 8) {
                        Text("Number of Alignpoints").font(.rk).lineLimit(1)
                        Spacer(minLength: 4)
                        ReadOnlyField(label: "Number of Alignpoints", value: model.align.points.count)
                    }
                    PanelCheckbox(title: "Keep Alignpoints inside ScanFrame", isOn: .constant(false), isLocked: true)
                }
            }
        }
    }
}

/// Step 3: No Align and Scan Frames (locked), the two sizes; under Advanced the rest of Alignment setup.
struct AlignStepView: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            LockedSegments(first: "No Align", second: "Scan Frames")
            Text("Not available yet").font(.rkSmall).foregroundStyle(RK.muted)
            FieldRow(title: "Alignmentbox size", value: $model.options.boxSize, range: 1...1_000_000).padding(.top, 6)
            FieldRow(title: "Max Alignpoint movement", value: $model.options.maxMovement)

            AdvancedSection(model: model, step: .align, count: 4) {
                VStack(alignment: .leading, spacing: 6) {
                    GroupHeading(title: "Alignment setup")
                    PanelCheckbox(title: "Show Alignment", isOn: $model.options.showAlignment)
                    // Locked as observed in the original; what unlocks them is not known (docs/ui.md).
                    PanelCheckbox(title: "Align by Centre of gravity", isOn: .constant(false), isLocked: true)
                    HStack(spacing: 8) {
                        Text("Lum. Threshold").font(.rk).foregroundStyle(RK.off)
                        Spacer(minLength: 4)
                        NumberField(label: "Lum. Threshold", value: .constant(0))
                            .disabled(true)
                            .opacity(0.45)
                    }
                    .padding(.leading, 20)
                    .help("Not available yet")
                    PanelCheckbox(title: "Estimate Rotation", isOn: .constant(false), isLocked: true)
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
        }
    }

    /// A radio button; the field of its kind sits in its row, locked while the other kind of mode is chosen.
    private func limitRow(_ mode: LimitMode, value: Binding<Int>?, label: String) -> some View {
        HStack(spacing: 8) {
            RadioRow(title: mode.label, isSelected: model.options.limitMode == mode) { model.options.limitMode = mode }
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
                Text("best frame")
                Spacer()
                Text("weakest")
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
