import SwiftUI

/// The Frame List and the Quality Graph (steps up to Limit) and Alignpoints Used for Stacking (Stack step) as cards over the image (docs/design.md, "Schwebende Karten"): top right, one below the
/// other, 236 pt wide, with a title and a cross that takes the check mark out of the Show menu. Not movable.
struct FloatingCards: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(spacing: 10) {
            if model.viewStep == .stack {
                if model.stackOptions.showAlignSections { AlignmentpointsCard(model: model) }
            } else {
                if model.show.framelist { FramelistCard(model: model) }
                if model.show.registrationgraph { RegistrationgraphCard(model: model) }
            }
            Spacer(minLength: 0)
        }
        .frame(width: 236)
        .frame(maxHeight: .infinity, alignment: .top)
        .padding(.top, 48)
        .padding(.trailing, 12)
        .padding(.bottom, 56)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
    }
}

/// The frame of a card: title, actions such as Reset or Auto as links at the left of the cross, the close cross, content.
struct FloatCard<Actions: View, Content: View>: View {
    let title: String
    let close: () -> Void
    @ViewBuilder let actions: () -> Actions
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 2) {
                Text(title).font(.rk(RKMetrics.fontSize, .semibold)).foregroundStyle(RK.text)
                Spacer()
                actions()
                Button(action: close) { Image(systemName: "xmark").font(.system(size: 9, weight: .semibold)) }
                    .buttonStyle(.rkBorderless)
                    .foregroundStyle(RK.muted)
                    .help("Close")
                    .accessibilityLabel("Close \(title)")
            }
            .padding(.leading, 10)
            .padding(.trailing, 4)
            .frame(height: 34)
            .overlay(alignment: .bottom) { RK.line.frame(height: 1) }
            content()
        }
        .background(RK.panel.opacity(0.94), in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
        .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusLarge).strokeBorder(RK.lineStrong, lineWidth: 1) }
        .shadow(color: .black.opacity(0.45), radius: 14, y: 10)
        .accessibilityElement(children: .contain)
        .accessibilityLabel(title)
    }
}

extension FloatCard where Actions == EmptyView {
    init(title: String, close: @escaping () -> Void, @ViewBuilder content: @escaping () -> Content) {
        self.init(title: title, close: close, actions: { EmptyView() }, content: content)
    }
}

/// Check All · Check None · Clear List (locked), and every frame with a check mark; after Align the best frame first, its quality at the right.
private struct FramelistCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Frame List", close: { model.show.framelist = false }) {
            VStack(spacing: 4) {
                HStack(spacing: 4) {
                    Button("Check All") { model.checkAllFrames(true) }
                    Button("Check None") { model.checkAllFrames(false) }
                    Button("Clear List") {}
                        .disabled(true)
                        .help("Not available yet")
                }
                .buttonStyle(.rkCompact)
                .padding(.horizontal, 8)
                .padding(.top, 6)
                Button("Go to Best Frame") { model.goToBestFrame() }
                    .buttonStyle(.rkCompact)
                    .disabled(!model.canGoToBestFrame)
                    .help(model.progress.step >= .aligned ? goToBestFrameHelp : "Run Align first")
                    .padding(.horizontal, 8)
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(model.align.listed) { frame in
                            Toggle(isOn: Binding(get: { frame.checked }, set: { model.setFrameChecked(frame.index, $0) })) {
                                HStack {
                                    Text(frame.name).font(.rk).foregroundStyle(RK.text)
                                    Spacer()
                                    if let quality = frame.quality {
                                        Text(String(format: "%.1f%%", quality)).font(.rk).foregroundStyle(RK.muted)
                                    }
                                }
                            }
                            .toggleStyle(.checkbox)
                            .frame(minHeight: 23)
                        }
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 4)
                }
                // As tall as the list, or less and then it scrolls.
                .frame(maxHeight: CGFloat(model.align.frames.count) * 23 + 8)
            }
            .padding(.bottom, 4)
        }
    }
}

/// One check box per alignpoint of the Stack step: only the checked points go into the stack.
private struct AlignmentpointsCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Alignpoints Used for Stacking", close: { model.stackOptions.showAlignSections = false }) {
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(model.align.points.indices, id: \.self) { index in
                        Toggle(isOn: Binding(get: { model.align.pointsChecked[safe: index] ?? true }, set: { model.setPointChecked(index, $0) })) {
                            Text("Alignpoint \(index + 1)").font(.rk).foregroundStyle(RK.text)
                        }
                        .toggleStyle(.checkbox)
                        .frame(maxWidth: .infinity, minHeight: 23, alignment: .leading)
                    }
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
            }
            .frame(maxHeight: CGFloat(model.align.points.count) * 23 + 8)
        }
    }
}

private extension Array {
    subscript(safe index: Int) -> Element? { indices.contains(index) ? self[index] : nil }
}

/// Quality and Difference per frame as two lines; before Align a note instead of empty axes.
private struct RegistrationgraphCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Quality Graph", close: { model.show.registrationgraph = false }) {
            VStack(alignment: .leading, spacing: 4) {
                Canvas { context, size in draw(&context, size: size) }
                    .aspectRatio(214.0 / 96.0, contentMode: .fit)
                    .accessibilityElement()
                    .accessibilityLabel("Quality and difference per frame")
                HStack(spacing: 14) {
                    legend("Quality", color: RK.accent, help: "How sharp a frame is, in percent of the best frame.")
                    legend("Difference", color: RK.info, help: "How much a frame differs from the frame all the others are lined up on; higher is worse.")
                }
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
        }
    }

    private func legend(_ title: String, color: Color, help: String) -> some View {
        HStack(spacing: 6) {
            RoundedRectangle(cornerRadius: 1).fill(color).frame(width: 14, height: 2)
            Text(title)
        }
        .help(help)
    }

    private func draw(_ context: inout GraphicsContext, size: CGSize) {
        guard model.align.isRated else {
            let note = context.resolve(Text("Run Align to see the graph").font(.rk(11)).foregroundStyle(RK.text.opacity(0.6)))
            context.draw(note, at: CGPoint(x: size.width / 2, y: size.height / 2))
            return
        }
        // The graph is drawn in a box of 214 x 96 and scaled to the card.
        let k = size.width / 214
        var axes = Path()
        axes.move(to: CGPoint(x: 8 * k, y: 10 * k))
        axes.addLine(to: CGPoint(x: 8 * k, y: 88 * k))
        axes.addLine(to: CGPoint(x: 206 * k, y: 88 * k))
        context.stroke(axes, with: .color(RK.text.opacity(0.25)), lineWidth: 1)
        let box = CGRect(x: 8 * k, y: 10 * k, width: 198 * k, height: 78 * k)
        let listed = model.align.listed
        for (values, color) in [(listed.map { $0.difference ?? 0 }, RK.info), (listed.map { $0.quality ?? 0 }, RK.accent)] {
            let points = seriesPoints(values, in: box)
            guard let first = points.first else { continue }
            var line = Path()
            line.move(to: first)
            for point in points.dropFirst() { line.addLine(to: point) }
            context.stroke(line, with: .color(color), style: StrokeStyle(lineWidth: 2, lineJoin: .round))
        }
    }
}
