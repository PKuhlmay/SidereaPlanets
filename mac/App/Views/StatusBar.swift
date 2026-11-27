import SwiftUI

/// The status bar (docs/design.md, "Statusleiste"): five fields separated by lines: progress, current action, result,
/// position under the pointer (Wavelets and Save steps) and the time.
struct StatusBar: View {
    let status: StatusTexts

    var body: some View {
        HStack(spacing: 0) {
            ResourceField()
            HStack(spacing: 8) {
                ZStack(alignment: .leading) {
                    Capsule().fill(SP.raised)
                    GeometryReader { proxy in
                        Capsule().fill(SP.accent).frame(width: proxy.size.width * min(max(status.progress, 0), 1))
                    }
                }
                .frame(height: 6)
                Text("\(Int((status.progress * 100).rounded()))%")
                    .foregroundStyle(SP.text)
                    .frame(width: 34, alignment: .trailing)
            }
            .padding(.horizontal, 12)
            .frame(width: 210)

            field(status.action, expands: true)
            field(status.result, expands: true)
            field(status.position, expands: false)
            field("Time used \(formatDuration(status.timeUsed)) · Time left \(formatDuration(status.timeLeft))", expands: false)
        }
        .font(.spSmall)
        .foregroundStyle(SP.muted)
        .frame(height: SPMetrics.statusBarHeight)
        .background(SP.panel)
        .overlay(alignment: .top) { SP.line.frame(height: 1) }
    }

    private func field(_ text: String, expands: Bool) -> some View {
        Text(text)
            .foregroundStyle(expands ? SP.text : SP.muted)
            .lineLimit(1)
            .truncationMode(.tail)
            .padding(.horizontal, 12)
            .frame(maxWidth: expands ? .infinity : nil, maxHeight: .infinity, alignment: .leading)
            .fixedSize(horizontal: !expands, vertical: false)
            .overlay(alignment: .leading) { SP.line.frame(width: 1) }
            .help(text)
    }
}
