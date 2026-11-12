import SwiftUI

/// The Framelist and the Registrationgraph as cards over the image (docs/design.md, "Schwebende Karten"): top right, one below the
/// other, 236 pt wide, with a title and a cross that takes the check mark out of the Show menu. Not movable.
struct FloatingCards: View {
    @Bindable var model: AppModel

    var body: some View {
        VStack(spacing: 10) {
            if model.show.framelist { FramelistCard(model: model) }
            if model.show.registrationgraph { RegistrationgraphCard(model: model) }
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

/// The frame of a card: title, close cross, content.
private struct FloatCard<Content: View>: View {
    let title: String
    let close: () -> Void
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text(title).font(.rk(RKMetrics.fontSize, .semibold)).foregroundStyle(RK.text)
                Spacer()
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

/// Check all · Check none · Empty List, and every frame with a check mark; after Align the best frame first, its quality at the right.
private struct FramelistCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Framelist", close: { model.show.framelist = false }) {
            VStack(spacing: 4) {
                HStack(spacing: 4) {
                    Button("Check all") { model.checkAllFrames(true) }
                    Button("Check none") { model.checkAllFrames(false) }
                    Button("Empty List") { model.emptyList() }
                }
                .buttonStyle(.rkCompact)
                .padding(.horizontal, 8)
                .padding(.top, 6)
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

/// Quality and Difference per frame as two lines (the reference program calls it the registration graph); before Align a note instead of empty axes.
private struct RegistrationgraphCard: View {
    @Bindable var model: AppModel

    var body: some View {
        FloatCard(title: "Registrationgraph", close: { model.show.registrationgraph = false }) {
            VStack(alignment: .leading, spacing: 4) {
                Canvas { context, size in draw(&context, size: size) }
                    .aspectRatio(214.0 / 96.0, contentMode: .fit)
                    .accessibilityElement()
                    .accessibilityLabel("Quality and difference per frame")
                HStack(spacing: 14) {
                    legend("Quality", color: RK.accent)
                    legend("Difference", color: RK.info)
                }
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
        }
    }

    private func legend(_ title: String, color: Color) -> some View {
        HStack(spacing: 6) {
            RoundedRectangle(cornerRadius: 1).fill(color).frame(width: 14, height: 2)
            Text(title)
        }
    }

    private func draw(_ context: inout GraphicsContext, size: CGSize) {
        guard model.align.isRated else {
            let note = context.resolve(Text("Run Align to see the graph").font(.rk(11)).foregroundStyle(RK.text.opacity(0.6)))
            context.draw(note, at: CGPoint(x: size.width / 2, y: size.height / 2))
            return
        }
        // The viewBox of the original, 214 x 96, scaled to the card.
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
