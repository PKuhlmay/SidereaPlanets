import AppKit
import SwiftUI
import WebKit

/// Shows one rendered documentation page. Links to other pages (`rkdoc:<slug>`) go to `onOpenPage`, web links open in the browser.
struct DocWebView: NSViewRepresentable {
    let html: String
    let onOpenPage: (String) -> Void

    func makeCoordinator() -> Coordinator { Coordinator(onOpenPage: onOpenPage) }

    func makeNSView(context: Context) -> WKWebView {
        let view = WKWebView(frame: .zero, configuration: WKWebViewConfiguration())
        view.navigationDelegate = context.coordinator
        view.underPageBackgroundColor = NSColor(SP.bg)
        view.setAccessibilityLabel("Documentation page")
        return view
    }

    func updateNSView(_ view: WKWebView, context: Context) {
        context.coordinator.onOpenPage = onOpenPage
        // Only a different page is loaded again: SwiftUI updates this view for many other reasons, and a reload would jump to the top.
        guard context.coordinator.loadedHTML != html else { return }
        context.coordinator.loadedHTML = html
        view.loadHTMLString(html, baseURL: nil)
    }

    @MainActor
    final class Coordinator: NSObject, WKNavigationDelegate {
        var onOpenPage: (String) -> Void
        var loadedHTML = ""

        init(onOpenPage: @escaping (String) -> Void) {
            self.onOpenPage = onOpenPage
        }

        func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction) async -> WKNavigationActionPolicy {
            guard navigationAction.navigationType == .linkActivated, let url = navigationAction.request.url else { return .allow }
            if url.scheme == "rkdoc" {
                onOpenPage(url.absoluteString.replacingOccurrences(of: "rkdoc:", with: ""))
            } else {
                NSWorkspace.shared.open(url)
            }
            return .cancel
        }
    }
}
