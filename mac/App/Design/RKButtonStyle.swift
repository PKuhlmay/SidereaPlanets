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

    func makeBody(configuration: Configuration) -> some View {
        StyledLabel(kind: kind, fullWidth: fullWidth || kind == .filled, configuration: configuration)
    }

    private struct StyledLabel: View {
        let kind: Kind
        let fullWidth: Bool
        let configuration: Configuration
        @Environment(\.isEnabled) private var isEnabled
        @State private var isHovering = false

        var body: some View {
            configuration.label
                .font(.rk(RKMetrics.fontSize, kind == .filled && isEnabled ? .semibold : .medium))
                .foregroundStyle(foreground)
                .padding(.horizontal, 12)
                .frame(minHeight: RKMetrics.buttonHeight)
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
            return kind == .filled ? RK.onAccent : RK.text
        }

        private var background: Color {
            guard isEnabled else { return .clear }
            let pressed = configuration.isPressed
            switch kind {
            case .filled: return isHovering || pressed ? RK.accentHover : RK.accent
            case .standard: return isHovering || pressed ? RK.raisedHover : RK.raised
            case .borderless: return isHovering || pressed ? RK.raised : .clear
            }
        }

        private var outline: Color {
            guard isEnabled else { return RK.line }
            return kind == .filled ? .clear : RK.lineStrong
        }
    }
}

extension ButtonStyle where Self == RKButtonStyle {
    static var rk: RKButtonStyle { RKButtonStyle() }
    static var rkWide: RKButtonStyle { RKButtonStyle(fullWidth: true) }
    static var rkFilled: RKButtonStyle { RKButtonStyle(kind: .filled) }
    static var rkBorderless: RKButtonStyle { RKButtonStyle(kind: .borderless) }
}
