import SwiftUI

/// The image area (docs/design.md, "Bildbereich"): the frame on `--canvas`, the caption at the top left and the zoom bar at the
/// bottom. The area stays dark in both appearances.
struct ImageArea: View {
    let model: AppModel

    var body: some View {
        let viewport = model.viewport
        ZStack {
            RK.canvas
            ImageCanvas(image: model.image, transform: viewport.transform, viewport: viewport)
            if model.image == nil {
                Text(model.hasRecording ? "No image yet" : "Press Select to open a recording")
                    .font(.rk)
                    .foregroundStyle(RK.onCanvas.opacity(0.6))
            } else {
                caption
                ZoomBar(viewport: viewport)
            }
        }
        .onGeometryChange(for: CGSize.self, of: \.size) { viewport.setBoxSize($0) }
        .clipped()
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Image")
    }

    @ViewBuilder
    private var caption: some View {
        if let text = model.imageCaption {
            Text(text)
                .font(.rk(RKMetrics.fontSizeSmall, .medium))
                .foregroundStyle(RK.onCanvas)
                .padding(.horizontal, 10)
                .frame(height: 28)
                .background(RK.canvasChrome, in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
                .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusLarge).strokeBorder(RK.canvasLine, lineWidth: 1) }
                .padding(10)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .allowsHitTesting(false)
        }
    }
}

/// − · scale (a click shows 100 %) · + · Fit, bottom centre.
private struct ZoomBar: View {
    let viewport: ImageViewport

    var body: some View {
        HStack(spacing: 2) {
            ZoomButton("−", help: "Zoom out (⌘-)", label: "Zoom out") { viewport.zoom(by: 1 / 1.25) }
            ZoomButton(viewport.transform.label, help: "Show at 100 % (⌘1)", label: "Zoom, show at 100 percent") { viewport.showActualSize() }
            ZoomButton("+", help: "Zoom in (⌘+)", label: "Zoom in") { viewport.zoom(by: 1.25) }
            RK.canvasLine.frame(width: 1, height: 16).padding(.horizontal, 3)
            ZoomButton("Fit", help: "Fit the image into the area (⌘0)", label: "Fit") { viewport.fit() }
        }
        .padding(3)
        .background(RK.canvasChrome, in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
        .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusLarge).strokeBorder(RK.canvasLine, lineWidth: 1) }
        .padding(12)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
    }
}

private struct ZoomButton: View {
    let title: String
    let help: String
    let label: String
    let action: () -> Void
    @State private var isHovering = false

    init(_ title: String, help: String, label: String, action: @escaping () -> Void) {
        self.title = title
        self.help = help
        self.label = label
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.rk)
                .foregroundStyle(RK.onCanvas)
                .padding(.horizontal, 8)
                .frame(minWidth: 26, minHeight: 24)
                .background(isHovering ? Color.white.opacity(0.1) : .clear, in: RoundedRectangle(cornerRadius: 5))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { isHovering = $0 }
        .help(help)
        .accessibilityLabel(label)
    }
}
