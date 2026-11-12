import SidereaPlanetsKit
import SwiftUI

/// The image area (docs/design.md, "Bildbereich"): the frame on `--canvas`, the caption at the top left and the zoom bar at the
/// bottom. The area stays dark in both appearances.
struct ImageArea: View {
    @Bindable var model: AppModel
    /// The rectangle of the region that is being dragged, in image pixels.
    @State private var draft: RoiSettings?

    /// The frame steps show the alignpoints and the region; the later steps show other images.
    private var showsFrameMarks: Bool { [.open, .alignpoints, .align, .limit].contains(model.viewStep) }
    private var isStackStep: Bool { model.viewStep == .stack }
    /// Wavelets and Save show the sharpened stack; Before and the Functions cards belong to Wavelets.
    private var isWaveletView: Bool { model.viewStep == .wavelets || model.viewStep == .save }

    var body: some View {
        let viewport = model.viewport
        ZStack {
            RK.canvas
            ImageCanvas(
                image: model.shownImage, transform: viewport.transform, viewport: viewport,
                isSelecting: model.isDrawingRegion, onDraft: { draft = $0 }, onSelect: { model.selectRegion($0) }
            )
            if model.shownImage == nil {
                Text(model.hasRecording ? "No image yet" : "Press Select Recording to open a video or a series of images")
                    .font(.rk)
                    .foregroundStyle(RK.onCanvas.opacity(0.6))
            } else {
                if showsFrameMarks || isStackStep || isWaveletView {
                    ImageOverlays(
                        transform: viewport.transform,
                        points: showsFrameMarks && model.show.alignpoints && model.progress.step >= .points ? model.align.points
                            : (isWaveletView && model.wavelet.showAlignpoints ? model.align.points : []),
                        sections: isStackStep ? model.alignSections : [],
                        region: showsFrameMarks && model.show.roi ? model.align.roi : processingAreaMark,
                        draft: draft
                    )
                }
                caption
                if model.viewStep != .open && (showsFrameMarks || isStackStep || isWaveletView) { ShowMenu(model: model) }
                if (showsFrameMarks && model.progress.step != .empty) || (isStackStep && model.progress.step >= .limited) {
                    FloatingCards(model: model)
                }
                if model.viewStep == .wavelets { FunctionCards(model: model) }
                ZoomBar(viewport: viewport, before: isWaveletView ? $model.waveletData.before : nil)
            }
        }
        .onGeometryChange(for: CGSize.self, of: \.size) { viewport.setBoxSize($0) }
        .clipped()
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Image")
    }

    /// The Processing Area as a dashed rectangle, with Show Processing Area in the Wavelets and Save steps.
    private var processingAreaMark: RoiSettings? {
        guard isWaveletView, model.wavelet.showProcessingArea, let area = model.processingAreaRegion else { return nil }
        return RoiSettings(x: area.x, y: area.y, width: area.width, height: area.height)
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

/// The Show menu at the top right (docs/design.md, "Show am Bild"): the check marks of the Align tab of the original, in the Stack
/// step those of the Stack tab. Show ROI is locked until a region is drawn.
private struct ShowMenu: View {
    @Bindable var model: AppModel

    var body: some View {
        HStack(spacing: 8) {
            if model.viewStep == .wavelets { functionsToggle }
            menu
        }
        .padding(10)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topTrailing)
    }

    /// Functions » shows and hides the right panel (Wavelets step); pressed with an outline and text in `--accent`.
    private var functionsToggle: some View {
        let isOn = model.wavelet.functionsOpen
        return Button { model.wavelet.functionsOpen.toggle() } label: {
            Text("Functions »")
                .font(.rk(RKMetrics.fontSizeSmall, .medium))
                .foregroundStyle(isOn ? RK.accentInk : RK.onCanvas)
                .padding(.horizontal, 10)
                .frame(height: 28)
                .background(isOn ? RK.accentSoft : RK.canvasChrome, in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
                .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusLarge).strokeBorder(isOn ? RK.accent : RK.canvasLine, lineWidth: 1) }
        }
        .buttonStyle(.plain)
        .help("Show or hide the Functions panel")
        .accessibilityAddTraits(isOn ? .isSelected : [])
    }

    private var menu: some View {
        Menu {
            if model.viewStep == .wavelets || model.viewStep == .save {
                Toggle("Show Full Image", isOn: $model.wavelet.showFullImage)
                Toggle("Show AlignPoints", isOn: $model.wavelet.showAlignpoints)
                Toggle("Show Processing Area", isOn: $model.wavelet.showProcessingArea)
            } else if model.viewStep == .stack {
                Toggle("Show Full Image", isOn: $model.stackOptions.showFullImage)
                Toggle("Show AlignSections", isOn: $model.stackOptions.showAlignSections)
                Toggle("Show Stackgraph", isOn: $model.stackOptions.showStackgraph)
                Toggle("Show Stacklist", isOn: $model.stackOptions.showStacklist)
            } else {
                Toggle("Show Full Image", isOn: $model.show.fullImage)
                Toggle("Show ROI", isOn: $model.show.roi).disabled(model.align.roi == nil)
                Toggle("Show Alignpoints", isOn: $model.show.alignpoints)
                Toggle("Show Aligndata", isOn: $model.show.aligndata)
                Toggle("Show Framelist", isOn: $model.show.framelist)
                Toggle("Show Registrationgraph", isOn: $model.show.registrationgraph)
                Toggle("Show Prefilter", isOn: $model.show.prefilter)
            }
        } label: {
            HStack(spacing: 5) {
                Text("Show").font(.rk(RKMetrics.fontSizeSmall, .medium))
                Image(systemName: "chevron.down").font(.system(size: 9, weight: .semibold))
            }
            .foregroundStyle(RK.onCanvas)
            .padding(.horizontal, 10)
            .frame(height: 28)
            .background(RK.canvasChrome, in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
            .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusLarge).strokeBorder(RK.canvasLine, lineWidth: 1) }
        }
        .menuStyle(.button)
        .buttonStyle(.plain)
        .menuIndicator(.hidden)
        .fixedSize()
        .help("What the image and the cards show")
    }
}

/// − · scale (a click shows 100 %) · + · Fit, bottom centre.
private struct ZoomBar: View {
    let viewport: ImageViewport
    /// Before in the Wavelets and Save steps: the stack without anything applied; `nil` in the other steps.
    var before: Binding<Bool>?

    var body: some View {
        HStack(spacing: 2) {
            ZoomButton("−", help: "Zoom out (⌘-)", label: "Zoom out") { viewport.zoom(by: 1 / 1.25) }
            ZoomButton(viewport.transform.label, help: "Show at 100 % (⌘1)", label: "Zoom, show at 100 percent") { viewport.showActualSize() }
            ZoomButton("+", help: "Zoom in (⌘+)", label: "Zoom in") { viewport.zoom(by: 1.25) }
            RK.canvasLine.frame(width: 1, height: 16).padding(.horizontal, 3)
            ZoomButton("Fit", help: "Fit the image into the area (⌘0)", label: "Fit") { viewport.fit() }
            if let before {
                ZoomButton("Before", help: "Show the unprocessed stack", label: "Before", isOn: before.wrappedValue) { before.wrappedValue.toggle() }
            }
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
    var isOn = false
    let action: () -> Void
    @State private var isHovering = false

    init(_ title: String, help: String, label: String, isOn: Bool = false, action: @escaping () -> Void) {
        self.title = title
        self.help = help
        self.label = label
        self.isOn = isOn
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.rk)
                .foregroundStyle(isOn ? RK.accent : RK.onCanvas)
                .padding(.horizontal, 8)
                .frame(minWidth: 26, minHeight: 24)
                .background(isOn ? RK.accent.opacity(0.18) : (isHovering ? Color.white.opacity(0.1) : .clear), in: RoundedRectangle(cornerRadius: 5))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { isHovering = $0 }
        .help(help)
        .accessibilityLabel(label)
        .accessibilityAddTraits(isOn ? .isSelected : [])
    }
}
