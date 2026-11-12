import Foundation
import Observation

/// The state of the Help window: the pages, the one shown and the search.
@MainActor @Observable
final class HelpModel {
    let pages: [DocPage]
    let groups: [DocGroup]
    /// Set when the pages could not be read (the folder is part of the app bundle, so this should not happen).
    let loadError: String?
    var query = ""
    private(set) var slug: String

    init(pages: [DocPage], loadError: String? = nil) {
        let groups = DocLibrary.group(pages)
        self.pages = groups.flatMap(\.pages)
        self.groups = groups
        self.loadError = loadError
        slug = self.pages.first?.slug ?? ""
    }

    /// The pages bundled with the app: the folder `pages` among the resources (a copy of app/src/docs/pages/, see `project.yml`).
    static func bundled(_ bundle: Bundle = .main) -> HelpModel {
        guard let folder = bundle.resourceURL?.appendingPathComponent("pages") else {
            return HelpModel(pages: [], loadError: "The documentation is not part of this build.")
        }
        do {
            return HelpModel(pages: try DocLibrary.load(from: folder))
        } catch {
            return HelpModel(pages: [], loadError: "The documentation could not be read: \(error.localizedDescription)")
        }
    }

    var isSearching: Bool { !query.trimmingCharacters(in: .whitespaces).isEmpty }
    var results: [DocPage] { DocLibrary.search(pages, query: query) }

    private var index: Int? { pages.firstIndex { $0.slug == slug } }
    var page: DocPage? { index.map { pages[$0] } }
    var previous: DocPage? { index.flatMap { $0 > 0 ? pages[$0 - 1] : nil } }
    var next: DocPage? { index.flatMap { $0 + 1 < pages.count ? pages[$0 + 1] : nil } }

    /// The HTML of the shown page.
    var html: String { page.map { DocStyle.document(for: $0, previous: previous, next: next) } ?? "" }

    /// Shows a page and leaves the search. An unknown slug (a broken link) changes nothing.
    func open(_ target: String) {
        guard pages.contains(where: { $0.slug == target }) else { return }
        slug = target
        query = ""
    }
}
