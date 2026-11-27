import SidereaPlanetsKit
import SwiftUI

// The cards Flip and Rotate and Resize Image (MAC-017, docs/design.md, "Karten hinter den Functions-Schaltflächen"). Flip and Rotate changes
// the picture in the image area; Resize Image only tells the size that Save Image writes. The calculation is in the Rust core.

/// Mirrors the picture, turns it by 90° or by any angle, and chooses how the pixels in between are calculated.
struct FlipRotateCard: View {
    @Bindable var model: AppModel

    private var values: OrientationValues { model.geometry.orientation }

    var body: some View {
        let canEdit = model.canEditOrientation
        FloatCard(title: flipRotateFunction, close: { model.closeFunction(flipRotateFunction) }, actions: {
            LinkButton("Reset", isEnabled: canEdit && values != OrientationValues(), help: "No flip, no turn.") { model.resetOrientation() }
        }, content: {
            VStack(alignment: .leading, spacing: 8) {
                Text("Flip").font(.spSmall).foregroundStyle(SP.muted)
                SegmentBar(items: [
                    SegmentItem(
                        title: "Horizontal", isOn: values.flipHorizontal, isEnabled: canEdit, help: "Mirrors the picture from left to right."
                    ) { model.flipHorizontally() },
                    SegmentItem(
                        title: "Vertical", isOn: values.flipVertical, isEnabled: canEdit, help: "Mirrors the picture from top to bottom."
                    ) { model.flipVertically() },
                ])
                Text("Rotate").font(.spSmall).foregroundStyle(SP.muted)
                SegmentBar(items: [
                    SegmentItem(title: "Left 90°", isEnabled: canEdit, help: "Turns the picture 90° counterclockwise.") { model.rotate(by: -90) },
                    SegmentItem(title: "Right 90°", isEnabled: canEdit, help: "Turns the picture 90° clockwise.") { model.rotate(by: 90) },
                    SegmentItem(title: "180°", isEnabled: canEdit, help: "Turns the picture upside down.") { model.rotate(by: 180) },
                ])
                angle(canEdit: canEdit)
                HStack {
                    Text("Interpolation").font(.sp).foregroundStyle(SP.text)
                        .help("How the pixels are calculated when the angle is not a multiple of 90°. Bicubic is sharper, Bilinear is smoother.")
                    Spacer(minLength: 4)
                    SegmentBar(items: RotationMethod.allCases.map { method in
                        SegmentItem(title: method.title, isOn: values.method == method, isEnabled: canEdit) { model.setRotationMethod(method) }
                    })
                    .frame(width: 130)
                }
                PanelCheckbox(
                    title: "Crop to fit", isOn: Binding(get: { values.cropToFit }, set: { model.setCropToFit($0) }),
                    help: "Cuts away the black corners of a turned picture. Has no effect on turns by 90°, 180° or 270°."
                )
                .disabled(!canEdit)
                HStack(spacing: 6) {
                    Text("Picture").font(.spSmall).foregroundStyle(SP.muted)
                    Spacer(minLength: 0)
                    Text(model.orientedSizeText ?? "").font(.sp).foregroundStyle(SP.text).accessibilityLabel("Size of the picture")
                }
                if !canEdit {
                    Text("Finish or cancel the crop first.").font(.spSmall).foregroundStyle(SP.muted)
                }
            }
            .padding(10)
        })
    }

    private func angle(canEdit: Bool) -> some View {
        VStack(spacing: 2) {
            HStack(spacing: 6) {
                Text("Angle (°)").font(.sp).foregroundStyle(SP.text)
                    .help("Turns the picture clockwise; a negative angle turns it counterclockwise.")
                Spacer()
                DecimalField(
                    label: "Angle in degrees", value: Binding(get: { values.angle }, set: { model.setAngle($0) }), decimals: 1,
                    step: OrientationLimits.angleStep, range: OrientationLimits.angle, width: 52
                )
                .disabled(!canEdit)
            }
            Slider(
                value: Binding(get: { values.angle }, set: { model.setAngle($0) }), in: OrientationLimits.angle,
                step: OrientationLimits.angleStep
            )
            .controlSize(.small)
            .tint(SP.accent)
            .disabled(!canEdit)
            .accessibilityLabel("Angle")
        }
    }
}

/// Scales the picture by a percentage or to a size in pixels when it is saved, with a choice of how the pixels are calculated.
struct ResizeCard: View {
    @Bindable var model: AppModel

    private var values: ResizeValues { model.geometry.resize }

    var body: some View {
        FloatCard(title: resizeFunction, close: { model.closeFunction(resizeFunction) }, actions: {
            LinkButton("Reset", isEnabled: values != ResizeValues(), help: "The picture keeps its size.") { model.resetResize() }
        }, content: {
            VStack(alignment: .leading, spacing: 8) {
                SegmentBar(items: [
                    SegmentItem(title: "Percent", isOn: values.mode == .percent, help: "Scales by a share of the size.") {
                        model.setResizeMode(.percent)
                    },
                    SegmentItem(title: "Pixels", isOn: values.mode == .pixels, help: "Scales to a width and a height in pixels.") {
                        model.setResizeMode(.pixels)
                    },
                ])
                if values.mode == .percent { percent } else { pixels }
                methodMenu
                VStack(alignment: .leading, spacing: 3) {
                    HStack(spacing: 6) {
                        Text("Result").font(.spSmall).foregroundStyle(SP.muted)
                        Spacer(minLength: 0)
                        Text(model.resultSizeText ?? "").font(.sp).foregroundStyle(SP.text).accessibilityLabel("Size of the saved picture")
                    }
                    Text("The picture is scaled when it is saved, after flipping, turning and cropping.")
                        .font(.spSmall).foregroundStyle(SP.muted).fixedSize(horizontal: false, vertical: true)
                }
            }
            .padding(10)
        })
    }

    private var percent: some View {
        VStack(spacing: 2) {
            HStack(spacing: 6) {
                Text("Size (%)").font(.sp).foregroundStyle(SP.text)
                Spacer()
                NumberField(
                    label: "Size percent", value: Binding(get: { values.percent }, set: { model.setResizePercent($0) }),
                    range: ResizeLimits.percent, width: 44
                )
            }
            Slider(
                value: Binding(get: { Double(values.percent) }, set: { model.setResizePercent(Int($0.rounded())) }),
                in: Double(ResizeLimits.percent.lowerBound)...Double(ResizeLimits.percent.upperBound), step: 1
            )
            .controlSize(.small)
            .tint(SP.accent)
            .accessibilityLabel("Size")
        }
    }

    private var pixels: some View {
        VStack(alignment: .leading, spacing: 6) {
            sideRow(
                "Width (px)", label: "Width in pixels",
                value: Binding(get: { model.resultSize?.width ?? 0 }, set: { model.setResizeWidth($0) })
            )
            sideRow(
                "Height (px)", label: "Height in pixels",
                value: Binding(get: { model.resultSize?.height ?? 0 }, set: { model.setResizeHeight($0) })
            )
            PanelCheckbox(
                title: "Lock aspect ratio", isOn: Binding(get: { values.lockAspect }, set: { model.setResizeLock($0) }),
                help: "Keeps the shape of the picture: the other side follows the one you type."
            )
        }
    }

    private func sideRow(_ title: String, label: String, value: Binding<Int>) -> some View {
        HStack(spacing: 6) {
            Text(title).font(.sp).foregroundStyle(SP.text)
            Spacer()
            NumberField(label: label, value: value, range: ResizeLimits.side, width: 52)
        }
    }

    private var methodMenu: some View {
        HStack {
            Text("Method").font(.sp).foregroundStyle(SP.text)
                .help("How the new pixels are calculated. Lanczos3 keeps the most detail.")
            Spacer()
            Menu {
                ForEach(ResizeMethod.allCases, id: \.self) { method in
                    Button(method.title) { model.setResizeMethod(method) }
                }
            } label: {
                HStack(spacing: 5) {
                    Text(values.method.title).font(.sp)
                    Image(systemName: "chevron.down").font(.system(size: 9, weight: .semibold))
                }
                .foregroundStyle(SP.text)
                .padding(.horizontal, 10)
                .frame(height: SPMetrics.fieldHeight)
                .background(SP.raised, in: RoundedRectangle(cornerRadius: SPMetrics.radius))
                .overlay { RoundedRectangle(cornerRadius: SPMetrics.radius).strokeBorder(SP.lineStrong, lineWidth: 1) }
            }
            .menuStyle(.button)
            .buttonStyle(.plain)
            .menuIndicator(.hidden)
            .fixedSize()
            .help("How the new pixels are calculated")
        }
    }
}

/// The size group of the step Save: what Save Image writes, and where to change it.
struct SizeSection: View {
    let model: AppModel

    var body: some View {
        if let size = model.resultSizeText {
            VStack(alignment: .leading, spacing: 6) {
                GroupHeading(title: "Size")
                    .help("The size of the file that Save Image writes.")
                Text("The picture is saved at \(size).")
                    .font(.spSmall).foregroundStyle(SP.muted).fixedSize(horizontal: false, vertical: true)
                if let turned = model.geometry.orientation.summary {
                    Text("It is \(turned).").font(.spSmall).foregroundStyle(SP.muted).fixedSize(horizontal: false, vertical: true)
                }
                Text("Flip and Rotate and Resize Image are in the Adjustments of the step Wavelets.")
                    .font(.spSmall).foregroundStyle(SP.muted).fixedSize(horizontal: false, vertical: true)
            }
        }
    }
}
