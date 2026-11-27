import SwiftUI

/// The buttons of docs/design.md ("Schaltflächen"): standard, filled (the one action of the open step) and borderless.
/// Disabled buttons lose their fill and show `--off`.
struct SPButtonStyle: ButtonStyle {
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
                .font(.sp(compact ? SPMetrics.fontSizeSmall : SPMetrics.fontSize, kind == .filled && isEnabled ? .semibold : .medium))
                .foregroundStyle(foreground)
                .lineLimit(1)
                .minimumScaleFactor(compact ? 0.85 : 1)
                .padding(.horizontal, compact ? 4 : 12)
                .frame(minHeight: compact ? 24 : SPMetrics.buttonHeight)
                .frame(maxWidth: fullWidth ? .infinity : nil)
                .background(background, in: RoundedRectangle(cornerRadius: SPMetrics.radius))
                .overlay {
                    if kind != .borderless {
                        RoundedRectangle(cornerRadius: SPMetrics.radius).strokeBorder(outline, lineWidth: 1)
                    }
                }
                .contentShape(RoundedRectangle(cornerRadius: SPMetrics.radius))
                .onHover { isHovering = $0 }
        }

        private var foreground: Color {
            guard isEnabled else { return SP.off }
            if isOn { return SP.accentInk }
            return kind == .filled ? SP.onAccent : SP.text
        }

        private var background: Color {
            guard isEnabled else { return .clear }
            if isOn { return SP.accentSoft }
            let pressed = configuration.isPressed
            switch kind {
            case .filled: return isHovering || pressed ? SP.accentHover : SP.accent
            case .standard: return isHovering || pressed ? SP.raisedHover : SP.raised
            case .borderless: return isHovering || pressed ? SP.raised : .clear
            }
        }

        private var outline: Color {
            guard isEnabled else { return SP.line }
            if isOn { return SP.accent }
            return kind == .filled ? .clear : SP.lineStrong
        }
    }
}

extension ButtonStyle where Self == SPButtonStyle {
    static var sp: SPButtonStyle { SPButtonStyle() }
    static var spWide: SPButtonStyle { SPButtonStyle(fullWidth: true) }
    static var spFilled: SPButtonStyle { SPButtonStyle(kind: .filled) }
    static var spBorderless: SPButtonStyle { SPButtonStyle(kind: .borderless) }
    static var spCompact: SPButtonStyle { SPButtonStyle(fullWidth: true, compact: true) }
    static func spWide(isOn: Bool) -> SPButtonStyle { SPButtonStyle(fullWidth: true, isOn: isOn) }
}
