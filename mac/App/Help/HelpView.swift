import SwiftUI

/// The Help window (docs/design.md, "Doku"): the page list with search at the left, the page at the right. No toolbar or step bar;
/// Escape closes it.
struct HelpView: View {
    @State private var model = HelpModel.bundled()
    @Environment(\.dismissWindow) private var dismissWindow

    var body: some View {
        HStack(spacing: 0) {
            PageList(model: model)
                .frame(width: SPMetrics.sideWidth)
                .background(SP.panel)
                .overlay(alignment: .trailing) { SP.line.frame(width: 1) }
            Group {
                if let error = model.loadError {
                    Text(error).foregroundStyle(SP.muted).frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    DocWebView(html: model.html) { model.open($0) }
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(SP.bg)
        }
        .frame(minWidth: 640, minHeight: 400)
        .tint(SP.accent)
        .onExitCommand { dismissWindow(id: HelpWindow.id) }
    }
}

/// The search field and the pages, grouped.
private struct PageList: View {
    @Bindable var model: HelpModel

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 2) {
                TextField("Search the documentation", text: $model.query)
                    .textFieldStyle(.plain)
                    .font(.sp)
                    .padding(.horizontal, 8)
                    .frame(height: 28)
                    .background(SP.field, in: RoundedRectangle(cornerRadius: SPMetrics.radiusSmall))
                    .overlay(RoundedRectangle(cornerRadius: SPMetrics.radiusSmall).stroke(SP.lineStrong, lineWidth: 1))
                    .padding(.bottom, 12)
                    .accessibilityLabel("Search the documentation")
                if model.isSearching {
                    if model.results.isEmpty {
                        Text("No page matches.").font(.sp).foregroundStyle(SP.muted).padding(.horizontal, 8)
                    }
                    ForEach(model.results) { PageRow(model: model, page: $0) }
                } else {
                    ForEach(model.groups) { group in
                        Text(group.name)
                            .font(.spSmall)
                            .foregroundStyle(SP.muted)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .accessibilityAddTraits(.isHeader)
                        ForEach(group.pages) { PageRow(model: model, page: $0) }
                        Spacer().frame(height: 10)
                    }
                }
            }
            .padding(12)
        }
    }
}

private struct PageRow: View {
    let model: HelpModel
    let page: DocPage
    @State private var isHovering = false

    var body: some View {
        let current = page.slug == model.slug
        Button { model.open(page.slug) } label: {
            Text(page.title)
                .font(.sp(SPMetrics.fontSize, current ? .semibold : .regular))
                .foregroundStyle(current ? SP.accentInk : SP.text)
                .padding(.horizontal, 8)
                .frame(maxWidth: .infinity, minHeight: 30, alignment: .leading)
                .background(
                    current ? SP.accentSoft : (isHovering ? SP.raised : .clear),
                    in: RoundedRectangle(cornerRadius: SPMetrics.radius)
                )
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .onHover { isHovering = $0 }
        .accessibilityAddTraits(current ? .isSelected : [])
    }
}

/// The Window scene of the Help window: one window, 1000 x 720 pt, at least 640 x 400, centred on the screen.
enum HelpWindow {
    static let id = "help"
}

struct HelpScene: Scene {
    var body: some Scene {
        Window("Siderea Planets Help", id: HelpWindow.id) {
            HelpView()
        }
        .defaultSize(width: 1000, height: 720)
        .defaultPosition(.center)
        .windowResizability(.contentMinSize)
    }
}

