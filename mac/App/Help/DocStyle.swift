import Foundation

/// The page around a rendered documentation page: the text styles of docs/design.md ("Doku"), the colour tokens of
/// docs/design/tokens.css (dark by default, light with the macOS appearance), 720 px of text in a wide window.
nonisolated enum DocStyle {
    static let css = """
        :root { color-scheme: dark;
          --bg:#141518; --panel:#1c1d21; --raised:#26282d; --line:#2e3036; --text:#e7e8eb; --muted:#9b9fa9; --off:#5f636d;
          --accent:#f2a93b; --accent-ink:#f2a93b; --accent-soft:rgba(242,169,59,.14); }
        @media (prefers-color-scheme: light) { :root { color-scheme: light;          --bg:#e9ebef; --panel:#f6f7f9; --raised:#ffffff; --line:#d8dce3; --text:#1a1c21; --muted:#5a606c; --off:#a4a9b4;
          --accent-ink:#8f5300; --accent-soft:rgba(242,169,59,.22); } }
        html, body { margin:0; background:var(--bg); color:var(--text); }
        body { font:14px/20px -apple-system, system-ui, "Helvetica Neue", sans-serif; -webkit-font-smoothing:antialiased; }
        .page { box-sizing:border-box; max-width:816px; margin:0 auto; padding:36px clamp(16px,5vw,48px) 48px; }
        .crumb { margin:0 0 14px; font-size:12px; color:var(--muted); }
        h1 { margin:0 0 14px; font-size:28px; font-weight:600; line-height:36px; }
        h1 + p { font-size:15px; line-height:22px; color:var(--muted); }
        h2 { margin:28px 0 14px; font-size:18px; font-weight:600; line-height:26px; }
        h3 { margin:20px 0 10px; font-size:15px; font-weight:600; }
        p { margin:0 0 14px; }
        ul { margin:0 0 14px; padding-left:22px; }
        li { margin-bottom:8px; }
        a { color:var(--accent-ink); }
        ol { list-style:none; counter-reset:step; margin:0 0 14px; padding:0; }
        ol > li { counter-increment:step; position:relative; margin-bottom:8px; padding-left:30px; }
        ol > li::before { content:counter(step); position:absolute; left:0; top:0; width:20px; height:20px; border-radius:50%;
          background:var(--raised); color:var(--muted); font-size:11px; line-height:20px; text-align:center; }
        table { width:100%; margin:0 0 14px; border-collapse:separate; border-spacing:0; overflow:hidden;
          background:var(--panel); border:1px solid var(--line); border-radius:8px; }
        th { height:32px; padding:0 12px; font-size:12px; font-weight:600; text-align:left; color:var(--muted); background:var(--raised); }
        td { padding:9px 12px; border-top:1px solid var(--line); vertical-align:top; }
        td:first-child { font-weight:600; }
        tr.off td { color:var(--off); }
        blockquote { margin:0 0 14px; padding:12px 14px; background:var(--accent-soft); border-left:3px solid var(--accent); border-radius:6px; }
        blockquote p { margin:0; }
        code { font:12px ui-monospace, "SF Mono", Menlo, monospace; }
        pre { margin:0 0 14px; padding:12px 14px; overflow-x:auto; background:var(--panel); border:1px solid var(--line); border-radius:6px; }
        hr { border:0; border-top:1px solid var(--line); margin:20px 0; }
        .pager { display:flex; justify-content:space-between; padding-top:16px; }
        .pager a { text-decoration:none; }
        .pager a:hover { text-decoration:underline; }
        """

    /// The whole HTML document for one page: breadcrumb (the group), the text and the links to the previous and next page.
    static func document(for page: DocPage, previous: DocPage?, next: DocPage?) -> String {
        func link(_ page: DocPage?, _ text: (String) -> String) -> String {
            page.map { "<a href=\"rkdoc:\($0.slug)\">\(text(MarkdownHTML.escape($0.title)))</a>" } ?? "<span></span>"
        }
        return """
            <!doctype html><html><head><meta charset="utf-8"><style>\(css)</style></head><body>
            <article class="page"><p class="crumb">\(MarkdownHTML.escape(page.group))</p>
            \(MarkdownHTML.render(page.body))
            <footer class="pager">\(link(previous) { "‹ \($0)" })\(link(next) { "\($0) ›" })</footer></article></body></html>
            """
    }
}
