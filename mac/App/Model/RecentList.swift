import Foundation

/// MRU: the recordings opened last, newest first. An entry is the list of files given to Open (several for a still-frame
/// sequence), so it is kept here and not in NSDocumentController, which only knows single documents.
/// Same rules as app/src/logic/recent.ts.
nonisolated struct RecentList: Equatable, Sendable {
    static let maxEntries = 10
    static let defaultsKey = "sp-recent"

    private(set) var entries: [[String]]

    init(entries: [[String]] = []) {
        self.entries = Array(entries.prefix(Self.maxEntries))
    }

    /// Moves `paths` to the front; the same selection is not listed twice.
    mutating func push(_ paths: [String]) {
        guard !paths.isEmpty else { return }
        entries = Array(([paths] + entries.filter { $0 != paths }).prefix(Self.maxEntries))
    }

    mutating func clear() {
        entries = []
    }

    /// Menu text: the file name, plus the number of further files for a still-frame sequence.
    static func label(for paths: [String]) -> String {
        guard let first = paths.first else { return "" }
        let name = URL(fileURLWithPath: first).lastPathComponent
        return paths.count > 1 ? "\(name) (+\(paths.count - 1))" : name
    }

    /// The folder of the first file, with the home folder as "~".
    static func folder(for paths: [String]) -> String {
        guard let first = paths.first else { return "" }
        return (URL(fileURLWithPath: first).deletingLastPathComponent().path as NSString).abbreviatingWithTildeInPath
    }

    // MARK: Persistence

    static func load(from defaults: UserDefaults) -> RecentList {
        guard let data = defaults.data(forKey: defaultsKey),
              let entries = try? JSONDecoder().decode([[String]].self, from: data)
        else { return RecentList() }
        return RecentList(entries: entries.filter { !$0.isEmpty })
    }

    func save(to defaults: UserDefaults) {
        defaults.set(try? JSONEncoder().encode(entries), forKey: Self.defaultsKey)
    }
}
