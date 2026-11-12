import Foundation

// The documentation pages of the Help window: the Markdown files of app/src/docs/pages/ (shared with the Windows app, which
// reads them in app/src/docs/docs.ts). Same rules there: a front matter with title, group and order, search over title and text.

/// One page of the documentation.
nonisolated struct DocPage: Equatable, Identifiable, Sendable {
    /// The file name without `.md`; links between pages use it (`[Welcome](welcome)`).
    let slug: String
    let title: String
    let group: String
    /// Position in the whole list, over all groups.
    let order: Double
    /// Markdown without the front matter.
    let body: String

    var id: String { slug }
}

nonisolated struct DocGroup: Equatable, Identifiable, Sendable {
    let name: String
    let pages: [DocPage]

    var id: String { name }
}

nonisolated struct DocPageError: Error, Equatable, LocalizedError {
    let message: String
    var errorDescription: String? { message }
}

nonisolated enum DocLibrary {
    /// Parses one file. The front matter is a block of `key: value` lines between two `---` lines at the very start.
    static func parse(slug: String, raw: String) throws -> DocPage {
        let text = raw.replacingOccurrences(of: "\r\n", with: "\n")
        guard text.hasPrefix("---\n"),
            let end = text.range(of: "\n---\n", range: text.index(text.startIndex, offsetBy: 3)..<text.endIndex)
        else { throw DocPageError(message: "Doc page \"\(slug)\" has no front matter") }
        var fields: [String: String] = [:]
        for line in text[text.index(text.startIndex, offsetBy: 4)..<end.lowerBound].split(separator: "\n") {
            guard let colon = line.firstIndex(of: ":"), colon != line.startIndex else { continue }
            let key = line[..<colon].trimmingCharacters(in: .whitespaces)
            fields[key] = line[line.index(after: colon)...].trimmingCharacters(in: .whitespaces)
        }
        guard let title = fields["title"], !title.isEmpty else {
            throw DocPageError(message: "Doc page \"\(slug)\" has no title")
        }
        return DocPage(
            slug: slug, title: title, group: fields["group"] ?? "", order: Double(fields["order"] ?? "") ?? 0,
            body: String(text[end.upperBound...])
        )
    }

    /// All `.md` files of a folder, as pages. A broken page is an error: the pages are part of the app and tested.
    static func load(from folder: URL) throws -> [DocPage] {
        let files = try FileManager.default.contentsOfDirectory(at: folder, includingPropertiesForKeys: nil)
            .filter { $0.pathExtension == "md" }
        let pages = try files.map {
            try parse(slug: $0.deletingPathExtension().lastPathComponent, raw: String(contentsOf: $0, encoding: .utf8))
        }
        return pages.sorted { ($0.order, $0.slug) < ($1.order, $1.slug) }
    }

    /// The groups in the order of their first page; the pages of a group by `order`.
    static func group(_ pages: [DocPage]) -> [DocGroup] {
        var names: [String] = []
        var members: [String: [DocPage]] = [:]
        for page in pages.sorted(by: { $0.order < $1.order }) {
            if members[page.group] == nil { names.append(page.group) }
            members[page.group, default: []].append(page)
        }
        return names.map { DocGroup(name: $0, pages: members[$0] ?? []) }
    }

    /// Every word must appear in the title or the text, ignoring case; pages with a word in the title come first.
    static func search(_ pages: [DocPage], query: String) -> [DocPage] {
        let words = query.lowercased().split(whereSeparator: \.isWhitespace).map(String.init)
        guard !words.isEmpty else { return pages }
        let hits = pages.filter { page in
            let text = "\(page.title)\n\(page.body)".lowercased()
            return words.allSatisfy(text.contains)
        }
        let inTitle = { (page: DocPage) in words.contains { page.title.lowercased().contains($0) } }
        return hits.filter(inTitle) + hits.filter { !inTitle($0) }
    }
}
