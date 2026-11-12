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
                .padding(RKMetrics.panelPadding)
                .modifier(KeepPanelWidth())
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
        case .stack: StackStepView(model: model)
        case .wavelets: WaveletsStepView(model: model)
        case .save: SaveStepView(model: model)
        }
    }

    // MARK: Foot

    /// After Save image everything is done and Save stays open with its result.
    private var isFinished: Bool { step == .save && state.status == .done && model.progress.nextOpen == .save }

    private var foot: some View {
        VStack(spacing: 8) {
            if isFinished {
                Text(model.result(of: step))
                    .font(.rkSmall)
                    .foregroundStyle(RK.ok)
                    .lineLimit(1)
                    .frame(maxWidth: .infinity, alignment: .leading)
                Button("Select Another Recording…", action: select)
                    .buttonStyle(.rkFilled)
                    .disabled(model.isBusy)
            } else if state.status == .done {
                Text(model.result(of: step))
                    .font(.rkSmall)
                    .foregroundStyle(RK.ok)
                    .lineLimit(1)
                    .frame(maxWidth: .infinity, alignment: .leading)
                actionLine {
                    Button(step.againLabel, action: act)
                        .buttonStyle(.rkWide)
                        .disabled(!step.isBuilt || model.isBusy)
                        .help(step.actionExplanation)
                }
                if model.progress.againDiscards(step) {
                    Text("Running it again discards the results of the later steps.")
                        .font(.rkSmall)
                        .foregroundStyle(RK.muted)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                if step == .stack { SaveImageMenu(model: model, source: .stack) }
                Button("Continue: \(model.progress.nextOpen.label) ›") { model.show(model.progress.nextOpen) }
                    .buttonStyle(.rkFilled)
                    .disabled(model.progress.nextOpen == step || model.isBusy)
            } else {
                if step == .save {
                    SaveImageMenu(model: model, source: .wavelet, filled: true)
                } else {
                    actionLine {
                        Button(step.action, action: act)
                            .buttonStyle(.rkFilled)
                            .disabled(!step.isBuilt || model.isBusy)
                            .help(step.isBuilt ? step.actionExplanation : "Not available yet")
                    }
                }
                if let next = step.next {
                    Text("Then: \(next.label)").font(.rkSmall).foregroundStyle(RK.muted)
                }
            }
        }
        .padding(RKMetrics.panelPadding)
        .background(RK.panel)
        .overlay(alignment: .top) { RK.line.frame(height: 1) }
    }

    /// The action button; in the Wavelets step Update Preview stands at its left.
    private func actionLine<Action: View>(@ViewBuilder _ action: () -> Action) -> some View {
        // Update Preview and the action side by side; when the labels do not fit, one below the other.
        ViewThatFits(in: .horizontal) {
            HStack(spacing: 6) {
                updatePreview
                action()
            }
            VStack(spacing: 6) {
                updatePreview
                action()
            }
        }
    }

    /// Update Preview of the Wavelets step; nothing in the other steps.
    @ViewBuilder
    private var updatePreview: some View {
        if step == .wavelets {
            Button("Update Preview") { Task { await model.runProcess() } }
                .buttonStyle(.rk)
                .fixedSize(horizontal: true, vertical: false)
                .disabled(model.isBusy || model.stacked == nil)
                .help("Calculates the preview area with the current settings.")
        }
    }

    /// The action of the shown step; Save image comes with its ticket.
    private func act() {
        switch step {
        case .open: select()
        case .alignpoints: Task { await model.setAlignpoints() }
        case .align: Task { await model.runAlign() }
        case .limit: Task { await model.runLimit() }
        case .stack: Task { await model.runStack() }
        case .wavelets: Task { await model.doAll() }
        case .save: break
        }
    }
}

/// Gives the content of the panel exactly the width the scroll view offers (the panel without a scroll bar) and anchors it at the
/// left. A row that is too wide for the panel is then cut off at the right; without this the scroll view takes the widest row as the
/// width of the content, centres it and cuts off the left edge of every row.
private struct KeepPanelWidth: ViewModifier {
    func body(content: Content) -> some View {
        KeepWidthLayout { content }
    }
}

private struct KeepWidthLayout: Layout {
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = proposal.width ?? subviews.first?.sizeThatFits(.unspecified).width ?? 0
        let height = subviews.first?.sizeThatFits(ProposedViewSize(width: width, height: nil)).height ?? 0
        return CGSize(width: width, height: height)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        subviews.first?.place(at: bounds.origin, anchor: .topLeading, proposal: ProposedViewSize(width: bounds.width, height: bounds.height))
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
                        Text(row.label).foregroundStyle(RK.muted).help(RecordingDetails.explanation(of: row.label) ?? "")
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
            Text("Recent Recordings").font(.rkSmall).foregroundStyle(RK.muted).padding(.bottom, 2)
            if model.recent.entries.isEmpty {
                Text("No recent recordings").font(.rkSmall).foregroundStyle(RK.muted)
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
