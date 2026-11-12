import SwiftUI

// The crop tool (MAC-016, docs/design.md, "Zuschneiden"): the card over the image that sets the format and keeps or drops the rectangle,
// and the Crop group of the step Save that opens it. The rectangle itself is drawn by `CropOverlay` and dragged in `ImageCanvas`.

/// The card of the crop tool, top right over the image like the cards of the Adjustments buttons: the Format, the size of the rectangle,
/// Reset, Cancel and Done. Closing the card is Cancel.
struct CropCard: View {
    @Bindable var model: AppModel

    /// The rectangle is the whole image: nothing is cropped yet, and the next drag on the image draws one.
    private var isWholeImage: Bool {
        guard let draft = model.crop.draft, let size = model.cropImageSize else { return true }
        return draft.isFull(imageWidth: size.width, imageHeight: size.height)
    }

    var body: some View {
        FloatCard(title: "Crop", close: { model.cancelCrop() }, actions: {
            LinkButton("Reset", help: "Back to the whole image.") { model.resetCrop() }
        }, content: {
            VStack(alignment: .leading, spacing: 8) {
                Text("Format").font(.rkSmall).foregroundStyle(RK.muted)
                SegmentBar(items: CropFormat.allCases.map { format in
                    SegmentItem(
                        title: format.title, isOn: model.crop.format == format,
                        help: format == .free ? "Any shape." : "Keeps the shape \(format.title) while the rectangle is dragged."
                    ) { model.setCropFormat(format) }
                })
                HStack(spacing: 6) {
                    Text("Size").font(.rkSmall).foregroundStyle(RK.muted)
                    Spacer(minLength: 0)
                    Text(model.cropSizeText ?? "").font(.rk).foregroundStyle(RK.text)
                        .accessibilityLabel("Size of the crop")
                }
                Text(isWholeImage
                    ? "Drag on the image to draw the part to keep."
                    : "Drag the rectangle to move it and its handles to resize it. Save Image writes only what is inside.")
                    .font(.rkSmall)
                    .foregroundStyle(RK.muted)
                    .fixedSize(horizontal: false, vertical: true)
                HStack(spacing: 8) {
                    Button("Cancel") { model.cancelCrop() }
                        .buttonStyle(.rk)
                        .help("Closes the crop tool and drops the changes.")
                    Button("Done") { model.finishCrop() }
                        .buttonStyle(.rkFilled)
                        .help("Keeps the rectangle: Save Image writes only what is inside.")
                }
            }
            .padding(10)
        })
    }
}

/// The Crop group of the step Save: what the rectangle is, Crop… to set it and Remove Crop to take it away.
struct CropSection: View {
    let model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            GroupHeading(title: "Crop")
                .help("Cuts the picture to a rectangle when it is saved.")
            Text(summary)
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
                .fixedSize(horizontal: false, vertical: true)
            HStack(spacing: 8) {
                Button("Crop…") { model.startCrop() }
                    .buttonStyle(.rk)
                    .disabled(!model.canCrop || model.crop.isEditing)
                    .help("Draws the part of the picture to keep, as Free, 1:1, 3:2 or 4:3.")
                if model.crop.applied != nil {
                    LinkButton("Remove Crop", isEnabled: !model.isBusy, help: "Saves the whole picture again.") { model.removeCrop() }
                }
            }
        }
    }

    private var summary: String {
        guard let applied = model.crop.applied else { return "The whole picture is saved." }
        return "Only \(applied.sizeText) is saved."
    }
}
