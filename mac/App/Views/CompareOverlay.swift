import AppKit
import SwiftUI

/// The divider and the labels of View Compare, over the image area (the two images themselves are drawn by `ImageCanvas`). The divider is
/// a line with a grip in the middle that is dragged sideways; VoiceOver's adjust gestures move it too. Only the strip around the line
/// takes the mouse; the rest of the area keeps panning the image.
struct CompareOverlay: View {
    @Bindable var model: AppModel

    private let gripWidth = 22.0
    /// What a label needs between the divider and the edge: its own width and the margin.
    private let labelRoom = 84.0

    var body: some View {
        GeometryReader { geometry in
            let width = geometry.size.width
            let share = model.wavelet.compareDivider
            ZStack {
                if compareLabelFits(share: share, width: width, needed: labelRoom, side: .before) {
                    label("Before").frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomLeading)
                }
                if compareLabelFits(share: share, width: width, needed: labelRoom, side: .after) {
                    label("After").frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
                }
                line(height: geometry.size.height)
                    .allowsHitTesting(false)
                    .position(x: share * width, y: geometry.size.height / 2)
                Color.clear
                    .frame(width: gripWidth + 8, height: geometry.size.height)
                    .contentShape(Rectangle())
                    .position(x: share * width, y: geometry.size.height / 2)
                    .gesture(
                        DragGesture(minimumDistance: 0, coordinateSpace: .named("compare"))
                            .onChanged { model.setCompareDivider(dividerShare(x: $0.location.x, width: width)) }
                    )
                    .onHover { inside in
                        if inside { NSCursor.resizeLeftRight.set() } else { NSCursor.openHand.set() }
                    }
                    .help("Drag to compare the stack with the result")
                    .accessibilityElement()
                    .accessibilityLabel("Compare divider")
                    .accessibilityValue("\(Int((share * 100).rounded())) percent from the left")
                    .accessibilityAdjustableAction { direction in
                        model.nudgeCompareDivider(by: direction == .increment ? 0.05 : -0.05)
                    }
            }
            .coordinateSpace(name: "compare")
        }
    }

    private func label(_ text: String) -> some View {
        Text(text)
            .font(.rk(RKMetrics.fontSizeSmall, .medium))
            .foregroundStyle(RK.onCanvas)
            .padding(.horizontal, 10)
            .frame(height: 24)
            .background(RK.canvasChrome, in: RoundedRectangle(cornerRadius: RKMetrics.radiusLarge))
            .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusLarge).strokeBorder(RK.canvasLine, lineWidth: 1) }
            .padding(12)
            .allowsHitTesting(false)
    }

    private func line(height: Double) -> some View {
        ZStack {
            Rectangle().fill(RK.onCanvas.opacity(0.9)).frame(width: 2, height: height)
                .shadow(color: .black.opacity(0.5), radius: 2)
            HStack(spacing: 1) {
                Image(systemName: "chevron.left")
                Image(systemName: "chevron.right")
            }
            .font(.system(size: 9, weight: .bold))
            // The image area stays dark in both appearances, so the grip uses the colors of the other canvas controls.
            .foregroundStyle(RK.accent)
            .frame(width: gripWidth, height: 28)
            .background(Color.black.opacity(0.75), in: Capsule())
            .overlay { Capsule().strokeBorder(RK.accent, lineWidth: 1) }
        }
        .frame(width: gripWidth)
    }
}
