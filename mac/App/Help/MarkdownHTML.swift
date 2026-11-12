import Foundation

/// A small Markdown renderer for the documentation pages: headings, paragraphs, lists, tables, quotes, code blocks and the
/// inline marks the pages use (code, bold, italic, links). It is not a general Markdown engine; the pages in
/// app/src/docs/pages/ are written within this subset, and a test renders all of them.
///
/// A link without scheme and slash (`[Welcome](welcome)`) points to another page and becomes `rkdoc:welcome`, which the Help
/// view handles; every other link stays as written. A table row that names something as "Not available yet" gets the class `off`
/// (greyed out), like in the Windows app.
nonisolated enum MarkdownHTML {
    static func render(_ markdown: String) -> String {
        let lines = markdown.replacingOccurrences(of: "\r\n", with: "\n").components(separatedBy: "\n")
        var html = ""
        var index = 0
        while index < lines.count {
            let line = lines[index]
            if line.trimmingCharacters(in: .whitespaces).isEmpty {
                index += 1
            } else if line.hasPrefix("```") {
                var code: [String] = []
                index += 1
                while index < lines.count, !lines[index].hasPrefix("```") {
                    code.append(lines[index])
                    index += 1
                }
                index += 1
                html += "<pre><code>\(escape(code.joined(separator: "\n")))</code></pre>\n"
            } else if let (level, text) = heading(line) {
                html += "<h\(level)>\(inline(text))</h\(level)>\n"
                index += 1
            } else if isRule(line) {
                html += "<hr>\n"
                index += 1
            } else if line.hasPrefix(">") {
                var quote: [String] = []
                while index < lines.count, lines[index].hasPrefix(">") {
                    quote.append(String(lines[index].dropFirst()).trimmingCharacters(in: .whitespaces))
                    index += 1
                }
                html += "<blockquote><p>\(inline(quote.joined(separator: " ")))</p></blockquote>\n"
            } else if line.hasPrefix("|") {
                var rows: [String] = []
                while index < lines.count, lines[index].hasPrefix("|") {
                    rows.append(lines[index])
                    index += 1
                }
                html += table(rows)
            } else if let kind = listKind(line) {
                var items: [String] = []
                while index < lines.count, listKind(lines[index]) == kind {
                    items.append(inline(stripMarker(lines[index])))
                    index += 1
                }
                let tag = kind == .ordered ? "ol" : "ul"
                html += "<\(tag)>\n" + items.map { "<li>\($0)</li>\n" }.joined() + "</\(tag)>\n"
            } else {
                var paragraph: [String] = []
                while index < lines.count, !startsBlock(lines[index]) {
                    paragraph.append(lines[index].trimmingCharacters(in: .whitespaces))
                    index += 1
                }
                html += "<p>\(inline(paragraph.joined(separator: " ")))</p>\n"
            }
        }
        return html
    }

    // MARK: Blocks

    private enum ListKind { case ordered, bullet }

    private static func heading(_ line: String) -> (Int, String)? {
        let hashes = line.prefix { $0 == "#" }.count
        guard (1...6).contains(hashes), line.dropFirst(hashes).first == " " else { return nil }
        return (hashes, line.dropFirst(hashes + 1).trimmingCharacters(in: .whitespaces))
    }

    private static func isRule(_ line: String) -> Bool {
        let text = line.trimmingCharacters(in: .whitespaces)
        return text.count >= 3 && Set(text) == ["-"]
    }

    private static func listKind(_ line: String) -> ListKind? {
        if line.hasPrefix("- ") || line.hasPrefix("* ") { return .bullet }
        let digits = line.prefix { $0.isNumber }
        if !digits.isEmpty, line.dropFirst(digits.count).hasPrefix(". ") { return .ordered }
        return nil
    }

    private static func stripMarker(_ line: String) -> String {
        if let end = line.firstIndex(of: " ") { return String(line[line.index(after: end)...]).trimmingCharacters(in: .whitespaces) }
        return line
    }

    private static func startsBlock(_ line: String) -> Bool {
        line.trimmingCharacters(in: .whitespaces).isEmpty || line.hasPrefix("```") || line.hasPrefix(">") || line.hasPrefix("|")
            || heading(line) != nil || listKind(line) != nil || isRule(line)
    }

    private static func cells(_ row: String) -> [String] {
        var text = row.trimmingCharacters(in: .whitespaces)
        if text.hasPrefix("|") { text.removeFirst() }
        if text.hasSuffix("|") { text.removeLast() }
        return text.components(separatedBy: "|").map { $0.trimmingCharacters(in: .whitespaces) }
    }

    private static func table(_ rows: [String]) -> String {
        guard rows.count >= 2 else { return "" }
        var html = "<table>\n<thead><tr>" + cells(rows[0]).map { "<th>\(inline($0))</th>" }.joined() + "</tr></thead>\n<tbody>\n"
        for row in rows.dropFirst(2) {
            let off = row.contains("Not available yet") ? " class=\"off\"" : ""
            html += "<tr\(off)>" + cells(row).map { "<td>\(inline($0))</td>" }.joined() + "</tr>\n"
        }
        return html + "</tbody>\n</table>\n"
    }

    // MARK: Inline

    static func escape(_ text: String) -> String {
        text.replacingOccurrences(of: "&", with: "&amp;")
            .replacingOccurrences(of: "<", with: "&lt;")
            .replacingOccurrences(of: ">", with: "&gt;")
            .replacingOccurrences(of: "\"", with: "&quot;")
    }

    /// Code, links, bold and italic of one line of text. Code spans are cut out first so their content stays as it is.
    static func inline(_ text: String) -> String {
        var codes: [String] = []
        var work = ""
        var rest = Substring(text)
        while let open = rest.firstIndex(of: "`") {
            let after = rest.index(after: open)
            guard let close = rest[after...].firstIndex(of: "`") else { break }
            work += rest[..<open]
            codes.append(String(rest[after..<close]))
            work += "\u{1}\(codes.count - 1)\u{1}"
            rest = rest[rest.index(after: close)...]
        }
        work += rest
        var html = escape(work)
        html = replace(html, #"\[([^\]]+)\]\(([^)\s]+)\)"#) { match in
            let href = match[2]
            let target = href.range(of: #"^[a-z0-9-]+$"#, options: .regularExpression) != nil ? "rkdoc:\(href)" : href
            return "<a href=\"\(target)\">\(match[1])</a>"
        }
        html = replace(html, #"\*\*(.+?)\*\*"#) { "<strong>\($0[1])</strong>" }
        html = replace(html, #"(?<![*\w])\*(?!\s)(.+?)(?<!\s)\*(?![*\w])"#) { "<em>\($0[1])</em>" }
        html = replace(html, "\u{1}([0-9]+)\u{1}") { "<code>\(escape(codes[Int($0[1]) ?? 0]))</code>" }
        return html
    }

    /// Regex replacement with a closure that gets the whole match and the capture groups.
    private static func replace(_ text: String, _ pattern: String, _ transform: ([String]) -> String) -> String {
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return text }
        var result = ""
        var last = text.startIndex
        for match in regex.matches(in: text, range: NSRange(text.startIndex..., in: text)) {
            guard let whole = Range(match.range, in: text) else { continue }
            let groups = (0..<match.numberOfRanges).map { Range(match.range(at: $0), in: text).map { String(text[$0]) } ?? "" }
            result += text[last..<whole.lowerBound] + transform(groups)
            last = whole.upperBound
        }
        return result + text[last...]
    }
}
