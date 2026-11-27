import SwiftUI

// The small controls the step panels are built from (docs/design.md, "Häkchen und Auswahl", "Zahlenfeld", "Panels und
// Gruppen"): native text field, stepper and check box, with the field and line colours of the design.

/// A whole-number field with a stepper (docs/design.md, "Zahlenfeld"): the value right-aligned in `--field`, the arrows at its
/// right. Typed values are only held to `range`; the design sets one only where the original has a known limit.
struct NumberField: View {
    let label: String
    @Binding var value: Int
    var range: ClosedRange<Int> = 0...1_000_000
    var width: CGFloat = 38

    private var clamped: Binding<Int> {
        Binding(get: { value }, set: { value = min(max($0, range.lowerBound), range.upperBound) })
    }

    var body: some View {
        HStack(spacing: 4) {
            TextField(label, value: clamped, format: .number.grouping(.never))
                .textFieldStyle(.plain)
                .multilineTextAlignment(.trailing)
                .font(.sp)
                .padding(.horizontal, 6)
                .frame(width: width, height: SPMetrics.fieldHeight)
                .background(SP.field, in: RoundedRectangle(cornerRadius: SPMetrics.radiusSmall))
                .overlay { RoundedRectangle(cornerRadius: SPMetrics.radiusSmall).strokeBorder(SP.lineStrong, lineWidth: 1) }
                .accessibilityLabel(label)
            Stepper(label, value: clamped, in: range).labelsHidden().controlSize(.small)
        }
        .fixedSize()
    }
}

/// A read-only value in the look of a field, without arrows (Number of Alignpoints).
struct ReadOnlyField: View {
    let label: String
    let value: Int

    var body: some View {
        Text("\(value)")
            .font(.sp)
            .foregroundStyle(SP.text)
            .padding(.horizontal, 6)
            .frame(width: 38, height: SPMetrics.fieldHeight, alignment: .trailing)
            .background(SP.field, in: RoundedRectangle(cornerRadius: SPMetrics.radiusSmall))
            .overlay { RoundedRectangle(cornerRadius: SPMetrics.radiusSmall).strokeBorder(SP.lineStrong, lineWidth: 1) }
            .padding(.trailing, 20)
            .accessibilityElement()
            .accessibilityLabel(label)
            .accessibilityValue("\(value)")
    }
}

/// The label at the left, the field at the right: one row of a panel. Locked, the field is dimmed to 45% (docs/design.md). A long
/// label wraps to a second line instead of being cut off; `help` is the tooltip of the row.
struct FieldRow: View {
    let title: String
    @Binding var value: Int
    var range: ClosedRange<Int> = 0...1_000_000
    var isLocked = false
    var help = ""

    var body: some View {
        HStack(spacing: 4) {
            Text(title)
                .font(.sp)
                .foregroundStyle(isLocked ? SP.off : SP.text)
                .lineLimit(2)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
            NumberField(label: title, value: $value, range: range)
                .disabled(isLocked)
                .opacity(isLocked ? 0.45 : 1)
        }
        .help(isLocked ? "Not available yet" : help)
    }
}

/// One radio button of a group that stands one below the other (Intensity selection, Limit Setup). SwiftUI has no radio button
/// of its own on the Mac outside a `Picker`, and the fields of Limit Setup have to sit in the rows of their buttons.
struct RadioRow: View {
    let title: String
    let isSelected: Bool
    var isEnabled = true
    var help = ""
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                Image(systemName: isSelected ? "circle.inset.filled" : "circle")
                    .font(.system(size: 13))
                    .foregroundStyle(isSelected ? SP.accent : SP.muted)
                Text(title).font(.sp).foregroundStyle(isEnabled ? SP.text : SP.off)
            }
            .frame(minHeight: 23, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
        .help(help)
        .accessibilityAddTraits(isSelected ? .isSelected : [])
    }
}

/// A check box in the style of the panels; locked ones carry the tooltip "Not available yet". `help` explains an active one.
struct PanelCheckbox: View {
    let title: String
    @Binding var isOn: Bool
    var isLocked = false
    var help = ""

    var body: some View {
        Toggle(isOn: $isOn) {
            Text(title).font(.sp).foregroundStyle(isLocked ? SP.off : SP.text)
        }
        .toggleStyle(.checkbox)
        .disabled(isLocked)
        .help(isLocked ? "Not available yet" : help)
    }
}

/// The heading of a group in the Advanced section and the lines between groups (docs/design.md, "Panels und Gruppen").
struct GroupHeading: View {
    let title: String

    var body: some View {
        Text(title).font(.sp(SPMetrics.fontSize, .semibold)).foregroundStyle(SP.text)
    }
}

/// "▸ Advanced · 5 more settings": the rest of a step's settings behind a line, open or closed per step, kept across launches.
struct AdvancedSection<Content: View>: View {
    let model: AppModel
    let step: ViewStep
    let count: Int
    @ViewBuilder let content: () -> Content

    private var isOpen: Bool { model.advancedOpen.contains(step) }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Button { model.toggleAdvanced(step) } label: {
                HStack {
                    Text("\(isOpen ? "▾" : "▸") Advanced").font(.sp(SPMetrics.fontSize, .medium)).foregroundStyle(SP.text)
                    Spacer()
                    Text("\(count) more settings").font(.spSmall).foregroundStyle(SP.muted)
                }
                .frame(minHeight: 29)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Advanced, \(count) more settings")
            .accessibilityValue(isOpen ? "expanded" : "collapsed")
            if isOpen { content() }
        }
        .padding(.top, 2)
        .overlay(alignment: .top) { SP.line.frame(height: 1) }
    }
}
