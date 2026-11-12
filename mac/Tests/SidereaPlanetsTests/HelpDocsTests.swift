import Foundation
import Testing

@testable import SidereaPlanets

/// The pages of the Help window: reading, grouping, searching, rendering (same rules as app/src/docs/docs.ts).
struct HelpDocsTests {
    private func page(_ slug: String, _ title: String, group: String = "A", order: Double = 1, body: String = "text") -> DocPage {
        DocPage(slug: slug, title: title, group: group, order: order, body: body)
    }

    // MARK: Front matter

    @Test("A page has title, group and order in its front matter, the rest is the body")
    func parsesFrontMatter() throws {
        let page = try DocLibrary.parse(slug: "welcome", raw: "---\ntitle: Welcome\ngroup: Getting started\norder: 1\n---\n# Welcome\n\nText.\n")
        #expect(page == DocPage(slug: "welcome", title: "Welcome", group: "Getting started", order: 1, body: "# Welcome\n\nText.\n"))
    }

    @Test("The order may have decimals (5.1 sorts between 5 and 6) and Windows line ends are fine")
    func decimalOrderAndCRLF() throws {
        let page = try DocLibrary.parse(slug: "a", raw: "---\r\ntitle: A: the first\r\norder: 5.1\r\n---\r\nBody\r\n")
        #expect(page.order == 5.1)
        #expect(page.title == "A: the first")
        #expect(page.body == "Body\n")
    }

    @Test("A page without front matter or without a title is an error")
    func rejectsBrokenPages() {
        #expect(throws: DocPageError.self) { try DocLibrary.parse(slug: "x", raw: "# No front matter") }
        #expect(throws: DocPageError.self) { try DocLibrary.parse(slug: "x", raw: "---\ngroup: G\n---\nBody") }
    }

    // MARK: Groups and search

    @Test("Groups follow the order of their first page, and pages inside a group follow `order`")
    func groupsAndOrder() {
        let pages = [
            page("c", "C", group: "Second", order: 3), page("a", "A", group: "First", order: 1),
            page("b", "B", group: "Second", order: 2), page("d", "D", group: "First", order: 4),
        ]
        let groups = DocLibrary.group(pages)
        #expect(groups.map(\.name) == ["First", "Second"])
        #expect(groups[0].pages.map(\.slug) == ["a", "d"])
        #expect(groups[1].pages.map(\.slug) == ["b", "c"])
    }

    @Test("Search needs every word in the title or the text, ignores case and puts title hits first")
    func search() {
        let pages = [
            page("text", "Other", body: "How to align the frames"), page("title", "Align your frames", body: "Nothing"),
            page("none", "Stack", body: "align only"),
        ]
        #expect(DocLibrary.search(pages, query: "ALIGN frames").map(\.slug) == ["title", "text"])
        #expect(DocLibrary.search(pages, query: "  ").count == 3)
        #expect(DocLibrary.search(pages, query: "wavelet").isEmpty)
    }

    // MARK: The real pages

    /// app/src/docs/pages/ of this checkout.
    private var pagesFolder: URL {
        URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().appendingPathComponent("app/src/docs/pages")
    }

    @Test("All pages of app/src/docs/pages/ read, and every link between pages leads to a page")
    func realPages() throws {
        let pages = try DocLibrary.load(from: pagesFolder)
        #expect(pages.count >= 10)
        let slugs = Set(pages.map(\.slug))
        for page in pages {
            #expect(!page.group.isEmpty, "\(page.slug) has no group")
            for match in page.body.matches(of: /\]\(([a-z0-9-]+)\)/) {
                #expect(slugs.contains(String(match.1)), "\(page.slug) links to \(match.1)")
            }
        }
        let groups = DocLibrary.group(pages)
        #expect(groups.first?.pages.first?.slug == "welcome")
    }

    @MainActor
    @Test("The app bundle carries the same pages")
    func bundledPages() throws {
        let model = HelpModel.bundled()
        #expect(model.loadError == nil)
        #expect(model.pages.count == (try DocLibrary.load(from: pagesFolder)).count)
    }

    // MARK: Model

    @MainActor
    @Test("Opening a page shows it and leaves the search; a broken link changes nothing")
    func opensPages() {
        let model = HelpModel(pages: [page("a", "A", order: 1), page("b", "B", order: 2), page("c", "C", order: 3)])
        #expect(model.slug == "a")
        #expect(model.previous == nil)
        #expect(model.next?.slug == "b")
        model.query = "c"
        #expect(model.isSearching)
        #expect(model.results.map(\.slug) == ["c"])
        model.open("c")
        #expect(model.slug == "c")
        #expect(model.query.isEmpty)
        #expect(model.next == nil)
        #expect(model.previous?.slug == "b")
        model.open("nowhere")
        #expect(model.slug == "c")
    }

    // MARK: Rendering

    @Test("Headings, paragraphs and the inline marks")
    func rendersText() {
        let html = MarkdownHTML.render("# Title\n\nSome **bold**, *italic* and `a < b` text.\nSecond line.\n\n## Part\n")
        #expect(html.contains("<h1>Title</h1>"))
        #expect(html.contains("<p>Some <strong>bold</strong>, <em>italic</em> and <code>a &lt; b</code> text. Second line.</p>"))
        #expect(html.contains("<h2>Part</h2>"))
    }

    @Test("Links between pages become rkdoc: links, other links stay")
    func rendersLinks() {
        let html = MarkdownHTML.render("See [Welcome](welcome) and [the site](https://example.com/a).")
        #expect(html.contains("<a href=\"rkdoc:welcome\">Welcome</a>"))
        #expect(html.contains("<a href=\"https://example.com/a\">the site</a>"))
    }

    @Test("Lists, quotes and tables, rows that are not available yet are marked")
    func rendersBlocks() {
        let html = MarkdownHTML.render(
            "1. One\n2. Two\n\n- Dot\n\n> Note **here**\n\n| Format | Works |\n|---|---|\n| SER | yes |\n| Still frames | Not available yet |\n"
        )
        #expect(html.contains("<ol>\n<li>One</li>\n<li>Two</li>\n</ol>"))
        #expect(html.contains("<ul>\n<li>Dot</li>\n</ul>"))
        #expect(html.contains("<blockquote><p>Note <strong>here</strong></p></blockquote>"))
        #expect(html.contains("<th>Format</th><th>Works</th>"))
        #expect(html.contains("<tr><td>SER</td><td>yes</td></tr>"))
        #expect(html.contains("<tr class=\"off\"><td>Still frames</td><td>Not available yet</td></tr>"))
    }

    @Test("Every real page renders without leftover Markdown marks")
    func rendersRealPages() throws {
        for page in try DocLibrary.load(from: pagesFolder) {
            let html = MarkdownHTML.render(page.body)
            #expect(!html.contains("**"), "\(page.slug): bold mark left")
            #expect(!html.contains("](") , "\(page.slug): link left")
            #expect(!html.contains("<p>|"), "\(page.slug): table left")
            #expect(!html.contains("<p>#"), "\(page.slug): heading left")
        }
    }
}
