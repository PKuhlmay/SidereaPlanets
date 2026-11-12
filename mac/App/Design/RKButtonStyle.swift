import SwiftUI

/// The buttons of docs/design.md ("Schaltflächen"): standard, filled (the one action of the open step) and borderless.
/// Disabled buttons lose their fill and show `--off`.
struct RKButtonStyle: ButtonStyle {
    enum Kind {
        case standard, filled, borderless
    }

    var kind: Kind = .standard
    /// Fills the width of the panel; the filled kind always does.
    var fullWidth = false
    /// A switch that is on (R.o.Interest while it waits for a drag): amber outline and text on `--accent-soft`.
    var isOn = false
    /// The small buttons of the floating cards: 24 pt high, 12 pt text.
    var compact = false

    func makeBody(configuration: Configuration) -> some View {
        StyledLabel(kind: kind, fullWidth: fullWidth || kind == .filled, isOn: isOn, compact: compact, configuration: configuration)
    }

    private struct StyledLabel: View {
        let kind: Kind
        let fullWidth: Bool
        let isOn: Bool
        let compact: Bool
        let configuration: Configuration
        @Environment(\.isEnabled) private var isEnabled
        @State private var isHovering = false

        var body: some View {
            configuration.label
                .font(.rk(compact ? RKMetrics.fontSizeSmall : RKMetrics.fontSize, kind == .filled && isEnabled ? .semibold : .medium))
                .foregroundStyle(foreground)
                .lineLimit(1)
                .minimumScaleFactor(compact ? 0.85 : 1)
                .padding(.horizontal, compact ? 4 : 12)
                .frame(minHeight: compact ? 24 : RKMetrics.buttonHeight)
                .frame(maxWidth: fullWidth ? .infinity : nil)
                .background(background, in: RoundedRectangle(cornerRadius: RKMetrics.radius))
                .overlay {
                    if kind != .borderless {
                        RoundedRectangle(cornerRadius: RKMetrics.radius).strokeBorder(outline, lineWidth: 1)
                    }
                }
                .contentShape(RoundedRectangle(cornerRadius: RKMetrics.radius))
                .onHover { isHovering = $0 }
        }

        private var foreground: Color {
            guard isEnabled else { return RK.off }
            if isOn { return RK.accentInk }
            return kind == .filled ? RK.onAccent : RK.text
        }

        private var background: Color {
            guard isEnabled else { return .clear }
            if isOn { return RK.accentSoft }
            let pressed = configuration.isPressed
            switch kind {
            case .filled: return isHovering || pressed ? RK.accentHover : RK.accent
            case .standard: return isHovering || pressed ? RK.raisedHover : RK.raised
            case .borderless: return isHovering || pressed ? RK.raised : .clear
            }
        }

        private var outline: Color {
            guard isEnabled else { return RK.line }
            if isOn { return RK.accent }
            return kind == .filled ? .clear : RK.lineStrong
        }
    }
}

extension ButtonStyle where Self == RKButtonStyle {
    static var rk: RKButtonStyle { RKButtonStyle() }
    static var rkWide: RKButtonStyle { RKButtonStyle(fullWidth: true) }
    static var rkFilled: RKButtonStyle { RKButtonStyle(kind: .filled) }
    static var rkBorderless: RKButtonStyle { RKButtonStyle(kind: .borderless) }
    static var rkCompact: RKButtonStyle { RKButtonStyle(fullWidth: true, compact: true) }
    static func rkWide(isOn: Bool) -> RKButtonStyle { RKButtonStyle(fullWidth: true, isOn: isOn) }
}
