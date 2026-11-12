import SwiftUI

/// The left panel (docs/design.md, "Schrittpanel"): result line of the previous step, header, the settings of the open step
/// and the foot with its action. Steps that are not built yet show the frame and a note.
struct StepPanel: View {
    let model: AppModel
    let select: () -> Void

    private var step: ViewStep { model.viewStep }
    private var state: StepState { model.progress.state(of: step) }

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    if let line = previousResult {
                        Label(line, systemImage: "checkmark")
                            .font(.rkSmall)
                            .foregroundStyle(RK.ok)
                    }
                    header
                    content
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(RKMetrics.panelPadding)
            }
            .scrollIndicators(.automatic)
            foot
        }
        .frame(width: RKMetrics.sideWidth)
        .background(RK.panel)
        .overlay(alignment: .trailing) { RK.line.frame(width: 1) }
    }

    /// "Open · JUPITER.AVI · 10 frames": only above the step that opens next.
    private var previousResult: String? {
        guard step == model.progress.nextOpen, let previous = step.previous else { return nil }
        return "\(previous.label) · \(model.result(of: previous))"
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("Step \(step.number) of 7").font(.rkSmall).foregroundStyle(RK.muted)
            Text(step.label).font(.rk(15, .semibold)).foregroundStyle(RK.text)
            Text(step.sentence)
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
                .lineSpacing(3)
                .fixedSize(horizontal: false, vertical: true)
                .padding(.top, 2)
        }
    }

    @ViewBuilder
    private var content: some View {
        switch step {
        case .open: OpenStepView(model: model)
        case .alignpoints: AlignpointsStepView(model: model)
        case .align: AlignStepView(model: model)
        case .limit: LimitStepView(model: model)
        default:
            Text("Comes in a later version")
                .font(.rkSmall)
                .foregroundStyle(RK.muted)
        }
    }

    // MARK: Foot

    private var foot: some View {
        VStack(spacing: 8) {
            if state.status == .done {
                Text(model.result(of: step))
                    .font(.rkSmall)
                    .foregroundStyle(RK.ok)
                    .lineLimit(1)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Button("\(step.action) again", action: act)
                    .buttonStyle(.rkWide)
                    .disabled(!step.isBuilt || model.isBusy)
                if model.progress.againDiscards(step) {
                    Text("Running it again discards the steps after it.")
                        .font(.rkSmall)
                        .foregroundStyle(RK.muted)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                Button("Continue: \(model.progress.nextOpen.label) ›") { model.show(model.progress.nextOpen) }
                    .buttonStyle(.rkFilled)
                    .disabled(model.progress.nextOpen == step || model.isBusy)
            } else {
                Button(step.action, action: act)
                    .buttonStyle(.rkFilled)
                    .disabled(!step.isBuilt || model.isBusy)
                    .help(step.isBuilt ? "" : "Comes in a later version")
                if let next = step.next {
                    Text("Then: \(next.label)").font(.rkSmall).foregroundStyle(RK.muted)
                }
            }
        }
        .padding(RKMetrics.panelPadding)
        .background(RK.panel)
        .overlay(alignment: .top) { RK.line.frame(height: 1) }
    }

    /// The action of the shown step; Stack, Do All and Save image come with their tickets.
    private func act() {
        switch step {
        case .open: select()
        case .alignpoints: Task { await model.setAlignpoints() }
        case .align: Task { await model.runAlign() }
        case .limit: Task { await model.runLimit() }
        case .stack, .wavelets, .save: break
        }
    }
}

/// Step 1 Open: the loaded recording with its file info, and the files opened last.
private struct OpenStepView: View {
    let model: AppModel

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            if let details = model.details {
                fileInfo(details)
            }
            recent
        }
    }

    private func fileInfo(_ details: RecordingDetails) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(details.fileName)
                .font(.rk(RKMetrics.fontSize, .semibold))
                .lineLimit(1)
                .truncationMode(.middle)
            Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 10, verticalSpacing: 3) {
                ForEach(Array(details.rows.enumerated()), id: \.offset) { _, row in
                    GridRow {
                        Text(row.label).foregroundStyle(RK.muted)
                        Text(row.value).foregroundStyle(RK.text).textSelection(.enabled)
                    }
                    .font(.rkSmall)
                }
            }
            RK.line.frame(height: 1).padding(.top, 6)
        }
    }

    private var recent: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("Recent").font(.rkSmall).foregroundStyle(RK.muted).padding(.bottom, 2)
            if model.recent.entries.isEmpty {
                Text("No recent files").font(.rkSmall).foregroundStyle(RK.muted)
            }
            ForEach(model.recent.entries, id: \.self) { paths in
                RecentRow(paths: paths, isEnabled: !model.isBusy) {
                    Task { await model.open(paths.map { URL(fileURLWithPath: $0) }) }
                }
            }
        }
    }
}

private struct RecentRow: View {
    let paths: [String]
    let isEnabled: Bool
    let action: () -> Void
    @State private var isHovering = false

    var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 0) {
                Text(RecentList.label(for: paths)).font(.rk(RKMetrics.fontSize)).foregroundStyle(isEnabled ? RK.text : RK.off)
                Text(RecentList.folder(for: paths)).font(.rkSmall).foregroundStyle(RK.muted)
            }
            .lineLimit(1)
            .truncationMode(.middle)
            .padding(.horizontal, 6)
            .padding(.vertical, 5)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(isHovering && isEnabled ? RK.raisedHover : .clear, in: RoundedRectangle(cornerRadius: RKMetrics.radiusSmall))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
        .onHover { isHovering = $0 }
        .help(paths.joined(separator: "\n"))
    }
}
