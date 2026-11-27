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
            SP.canvas
            ImageCanvas(
                image: model.shownImage, beforeImage: model.compareBeforeImage, divider: model.wavelet.compareDivider,
                transform: viewport.transform, viewport: viewport,
                isSelecting: model.isDrawingRegion, onDraft: { draft = $0 }, onSelect: { model.selectRegion($0) },
                isCropping: model.crop.isEditing, crop: model.crop.draft, cropAspect: model.cropAspect,
                onCropChange: { model.setCropDraft($0) }
            )
            if model.shownImage == nil {
                Text(model.hasRecording ? "No image to show yet" : "Select a recording or drop a video or images here")
                    .font(.sp)
                    .foregroundStyle(SP.onCanvas.opacity(0.6))
            } else {
                if showsFrameMarks || isStackStep || isWaveletView {
                    ImageOverlays(
                        transform: viewport.transform,
                        points: showsFrameMarks && model.show.alignpoints && model.progress.step >= .points ? model.align.points
                            : (isWaveletView && model.wavelet.showAlignpoints && !model.orientationApplies ? model.align.points : []),
                        sections: isStackStep ? model.alignSections : [],
                        region: showsFrameMarks && model.show.roi ? model.align.roi : processingAreaMark,
                        draft: draft,
                        disc: isStackStep ? model.derotationDisc : nil
                    )
                }
                cropMark
                // The split view has its own labels for the two sides.
                if model.isComparing { CompareOverlay(model: model) } else { caption }
                if model.viewStep != .open && (showsFrameMarks || isStackStep || isWaveletView) { ShowMenu(model: model) }
                if (showsFrameMarks && model.progress.step != .empty) || (isStackStep && model.progress.step >= .limited) {
                    FloatingCards(model: model)
                }
                if model.viewStep == .wavelets || (model.viewStep == .save && model.crop.isEditing) { FunctionCards(model: model) }
                ZoomBar(
                    viewport: viewport,
                    before: isWaveletView ? Binding(get: { model.waveletData.before }, set: { model.setBefore($0) }) : nil
                )
            }
        }
        .onGeometryChange(for: CGSize.self, of: \.size) { viewport.setBoxSize($0) }
        .clipped()
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Image")
    }

    /// The crop rectangle: with its grips while it is edited, as the dimmed outside with the size once Done kept it (Wavelets and Save).
    @ViewBuilder
    private var cropMark: some View {
        if isWaveletView, let shown = model.crop.isEditing ? model.crop.draft : model.crop.applied,
           let size = model.cropImageSize,
           !shown.isFull(imageWidth: size.width, imageHeight: size.height) {
            CropOverlay(transform: model.viewport.transform, rectangle: shown, isEditing: model.crop.isEditing)
        }
    }

    /// The preview area as a dashed rectangle, with Show Preview Area in the Wavelets and Save steps.
    private var processingAreaMark: RoiSettings? {
        // The marks are in the pixels of the stack; a flipped or turned picture has no place for them.
        guard isWaveletView, model.wavelet.showProcessingArea, !model.orientationApplies, let area = model.processingAreaRegion else { return nil }
        return RoiSettings(x: area.x, y: area.y, width: area.width, height: area.height)
    }

    @ViewBuilder
    private var caption: some View {
        if let text = model.imageCaption {
            Text(text)
                .font(.sp(SPMetrics.fontSizeSmall, .medium))
                .foregroundStyle(SP.onCanvas)
                .padding(.horizontal, 10)
                .frame(height: 28)
                .background(SP.canvasChrome, in: RoundedRectangle(cornerRadius: SPMetrics.radiusLarge))
                .overlay { RoundedRectangle(cornerRadius: SPMetrics.radiusLarge).strokeBorder(SP.canvasLine, lineWidth: 1) }
                .padding(10)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .allowsHitTesting(false)
        }
    }
}

/// The Show menu at the top right (docs/design.md, "Show am Bild"): the check marks of the Align tab, in the Stack
/// step those of the Stack tab. Show Region is locked until a region is drawn; entries without an effect are dimmed.
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
            Text("Adjust ›")
                .font(.sp(SPMetrics.fontSizeSmall, .medium))
                .foregroundStyle(isOn ? SP.accentInk : SP.onCanvas)
                .padding(.horizontal, 10)
                .frame(height: 28)
                .background(isOn ? SP.accentSoft : SP.canvasChrome, in: RoundedRectangle(cornerRadius: SPMetrics.radiusLarge))
                .overlay { RoundedRectangle(cornerRadius: SPMetrics.radiusLarge).strokeBorder(isOn ? SP.accent : SP.canvasLine, lineWidth: 1) }
        }
        .buttonStyle(.plain)
        .help("Show or hide the panel with color, tone and other image adjustments")
        .accessibilityAddTraits(isOn ? .isSelected : [])
    }

    /// The check mark of an entry that works; which setting it is depends on the step.
    private func binding(_ item: ShowItem) -> Binding<Bool> {
        switch item {
        case .region: $model.show.roi
        case .alignpoints:
            switch model.viewStep {
            case .wavelets, .save: $model.wavelet.showAlignpoints
            case .stack: $model.stackOptions.showAlignSections
            default: $model.show.alignpoints
            }
        case .frameList: $model.show.framelist
        case .qualityGraph: $model.show.registrationgraph
        case .previewArea: $model.wavelet.showProcessingArea
        case .fullImage, .alignData, .prefilter, .stackGraph, .stackList: .constant(false)
        }
    }

    private var menu: some View {
        Menu {
            let items = showItems(for: model.viewStep)
            ForEach(items.filter(\.isAvailable), id: \.self) { item in
                Toggle(item.title, isOn: binding(item)).disabled(item == .region && model.align.roi == nil)
            }
            // The check marks that change nothing on screen are dimmed under their own heading.
            let locked = items.filter { !$0.isAvailable }
            if !locked.isEmpty {
                Divider()
                Section("Not available yet") {
                    ForEach(locked, id: \.self) { item in
                        Toggle(item.title, isOn: .constant(false)).disabled(true)
                    }
                }
            }
        } label: {
            HStack(spacing: 5) {
                Text("Show").font(.sp(SPMetrics.fontSizeSmall, .medium))
                Image(systemName: "chevron.down").font(.system(size: 9, weight: .semibold))
            }
            .foregroundStyle(SP.onCanvas)
            .padding(.horizontal, 10)
            .frame(height: 28)
            .background(SP.canvasChrome, in: RoundedRectangle(cornerRadius: SPMetrics.radiusLarge))
            .overlay { RoundedRectangle(cornerRadius: SPMetrics.radiusLarge).strokeBorder(SP.canvasLine, lineWidth: 1) }
        }
        .menuStyle(.button)
        .buttonStyle(.plain)
        .menuIndicator(.hidden)
        .fixedSize()
        .help("What the image and the cards show")
    }
}

/// − · scale (a click shows 100%) · + · Fit, bottom centre.
private struct ZoomBar: View {
    let viewport: ImageViewport
    /// Before in the Wavelets and Save steps: the stack without anything applied; `nil` in the other steps.
    var before: Binding<Bool>?

    var body: some View {
        HStack(spacing: 2) {
            ZoomButton("−", help: "Zoom out (⌘-)", label: "Zoom out") { viewport.zoom(by: 1 / 1.25) }
            ZoomButton(viewport.transform.label, help: "Show at 100% (⌘1)", label: "Zoom, show at 100 percent") { viewport.showActualSize() }
            ZoomButton("+", help: "Zoom in (⌘+)", label: "Zoom in") { viewport.zoom(by: 1.25) }
            SP.canvasLine.frame(width: 1, height: 16).padding(.horizontal, 3)
            ZoomButton("Fit", help: "Fit the image into the area (⌘0)", label: "Fit") { viewport.fit() }
            if let before {
                ZoomButton("Before/After", help: "Switch between the stack without any sharpening and the sharpened image. Turns off View Compare", label: "Before and after", isOn: before.wrappedValue) { before.wrappedValue.toggle() }
            }
        }
        .padding(3)
        .background(SP.canvasChrome, in: RoundedRectangle(cornerRadius: SPMetrics.radiusLarge))
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: SPMetrics.radiusLarge))
        .overlay { RoundedRectangle(cornerRadius: SPMetrics.radiusLarge).strokeBorder(SP.canvasLine, lineWidth: 1) }
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
                .font(.sp)
                .foregroundStyle(isOn ? SP.accent : SP.onCanvas)
                .padding(.horizontal, 8)
                .frame(minWidth: 26, minHeight: 24)
                .background(isOn ? SP.accent.opacity(0.18) : (isHovering ? Color.white.opacity(0.1) : .clear), in: RoundedRectangle(cornerRadius: 5))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { isHovering = $0 }
        .help(help)
        .accessibilityLabel(label)
        .accessibilityAddTraits(isOn ? .isSelected : [])
    }
}
