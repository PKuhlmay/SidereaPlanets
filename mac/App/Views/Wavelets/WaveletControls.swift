import SwiftUI

// The controls of the Wavelets step and the Functions cards that the other steps do not need: a field with decimals, segments, and the
// text links of group headings and cards (docs/design.md, "Zahlenfeld", "Segmente", "Panels und Gruppen").

/// A field with a fixed number of decimals and a stepper; the decimal separator is always a point (docs/design.md, "Zahlenfeld").
/// A typed value is rounded to the decimals and held to `range`.
struct DecimalField: View {
    let label: String
    @Binding var value: Double
    var decimals = 2
    var step = 0.01
    var range: ClosedRange<Double> = -1_000_000...1_000_000
    var width: CGFloat = 50

    private var clamped: Binding<Double> {
        Binding(
            get: { value },
            set: {
                let scale = pow(10, Double(decimals))
                value = min(max(($0 * scale).rounded() / scale, range.lowerBound), range.upperBound)
            }
        )
    }

    var body: some View {
        HStack(spacing: 4) {
            TextField(label, value: clamped, format: .number.locale(Locale(identifier: "en_US")).precision(.fractionLength(decimals)).grouping(.never))
                .textFieldStyle(.plain)
                .multilineTextAlignment(.trailing)
                .font(.rk)
                .padding(.horizontal, 6)
                .frame(width: width, height: RKMetrics.fieldHeight)
                .background(RK.field, in: RoundedRectangle(cornerRadius: RKMetrics.radiusSmall))
                .overlay { RoundedRectangle(cornerRadius: RKMetrics.radiusSmall).strokeBorder(RK.lineStrong, lineWidth: 1) }
                .accessibilityLabel(label)
            Stepper(label, value: clamped, in: range, step: step).labelsHidden().controlSize(.small)
        }
        .fixedSize()
    }
}

/// One segment of a `SegmentBar`.
struct SegmentItem: Identifiable {
    let title: String
    var isOn = false
    var isEnabled = true
    let action: () -> Void

    var id: String { title }
}

/// Connected buttons in a shared outline: a choice of two or a group of related actions (docs/design.md, "Segmente"). A segment that is on
/// shows `--accent-soft` with text in `--accent-ink`.
struct SegmentBar: View {
    let items: [SegmentItem]

    var body: some View {
        HStack(spacing: 0) {
            ForEach(Array(items.enumerated()), id: \.element.id) { index, item in
                if index > 0 { RK.lineStrong.frame(width: 1) }
                Button(action: item.action) {
                    Text(item.title)
                        .font(.rk(RKMetrics.fontSize, .medium))
                        .foregroundStyle(item.isEnabled ? (item.isOn ? RK.accentInk : RK.text) : RK.off)
                        .lineLimit(1)
                        .minimumScaleFactor(0.85)
                        .frame(maxWidth: .infinity, minHeight: RKMetrics.buttonHeight)
                        .background(item.isOn && item.isEnabled ? RK.accentSoft : RK.raised)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .disabled(!item.isEnabled)
                .accessibilityAddTraits(item.isOn ? .isSelected : [])
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: RKMetrics.radius))
        .overlay { RoundedRectangle(cornerRadius: RKMetrics.radius).strokeBorder(RK.lineStrong, lineWidth: 1) }
    }
}

/// A text link in a group heading or a card (Reset Wavelets, Hold, Reset, Auto): `--accent-ink`, no frame. `isOn` for a switch that is held.
struct LinkButton: View {
    let title: String
    var isOn = false
    var isEnabled = true
    let action: () -> Void
    @State private var isHovering = false

    init(_ title: String, isOn: Bool = false, isEnabled: Bool = true, action: @escaping () -> Void) {
        self.title = title
        self.isOn = isOn
        self.isEnabled = isEnabled
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.rk(RKMetrics.fontSize, isOn ? .semibold : .regular))
                .foregroundStyle(isEnabled ? RK.accentInk : RK.off)
                .padding(.horizontal, 4)
                .padding(.vertical, 2)
                .background(isHovering && isEnabled || isOn ? RK.accentSoft : .clear, in: RoundedRectangle(cornerRadius: RKMetrics.radiusSmall))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
        .onHover { isHovering = $0 }
        .accessibilityAddTraits(isOn ? .isSelected : [])
    }
}
