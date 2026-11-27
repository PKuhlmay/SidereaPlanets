import AppKit
import SwiftUI

// Design tokens of docs/design/tokens.css as Swift values. tokens.css stays the single source: change a value there
// first, then here. Colours follow the macOS appearance (dark is the CSS default, light is `data-theme="light"`);
// the image area stays dark in both.

/// An sRGB colour with alpha, from a 0xRRGGBB literal.
nonisolated struct RGBA: Sendable {
    let hex: UInt32
    let alpha: Double

    init(_ hex: UInt32, alpha: Double = 1) {
        self.hex = hex
        self.alpha = alpha
    }

    var nsColor: NSColor {
        NSColor(
            srgbRed: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255,
            alpha: alpha
        )
    }
}

extension Color {
    /// A colour that switches with the appearance of the window it is drawn in.
    fileprivate init(dark: RGBA, light: RGBA) {
        self.init(nsColor: NSColor(name: nil) { appearance in
            let isDark = appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            return (isDark ? dark : light).nsColor
        })
    }

    /// The same colour in both appearances.
    fileprivate init(_ both: RGBA) {
        self.init(dark: both, light: both)
    }
}

/// Colours, named like the CSS variables (`--panel` is `SP.panel`).
enum SP {
    // surfaces: neutral greys without tint, so the UI does not shift the perceived image colours
    static let bg = Color(dark: RGBA(0x141518), light: RGBA(0xE9EBEF))
    static let panel = Color(dark: RGBA(0x1C1D21), light: RGBA(0xF6F7F9))
    static let raised = Color(dark: RGBA(0x26282D), light: RGBA(0xFFFFFF))
    static let raisedHover = Color(dark: RGBA(0x30333A), light: RGBA(0xEEF0F4))
    static let field = Color(dark: RGBA(0x101114), light: RGBA(0xFFFFFF))
    static let line = Color(dark: RGBA(0x2E3036), light: RGBA(0xD8DCE3))
    static let lineStrong = Color(dark: RGBA(0x40434B), light: RGBA(0xBCC2CC))

    // text
    static let text = Color(dark: RGBA(0xE7E8EB), light: RGBA(0x1A1C21))
    static let muted = Color(dark: RGBA(0x9B9FA9), light: RGBA(0x5A606C))
    static let off = Color(dark: RGBA(0x5F636D), light: RGBA(0xA4A9B4))

    // accent: amber, the only accent colour
    static let accent = Color(RGBA(0xF2A93B))
    static let accentHover = Color(dark: RGBA(0xF7B955), light: RGBA(0xEB9C22))
    static let onAccent = Color(RGBA(0x1D1300))
    static let accentInk = Color(dark: RGBA(0xF2A93B), light: RGBA(0x8F5300))
    static let accentSoft = Color(dark: RGBA(0xF2A93B, alpha: 0.14), light: RGBA(0xF2A93B, alpha: 0.22))

    // status
    static let ok = Color(dark: RGBA(0x4FC08D), light: RGBA(0x1E8A5A))
    static let info = Color(dark: RGBA(0x6FA8FF), light: RGBA(0x2F6FD6))

    // histogram curves: image data, not UI colours
    static let channelRed = Color(dark: RGBA(0xFF6B6B), light: RGBA(0xD93F3F))
    static let channelGreen = Color(dark: RGBA(0x4FC08D), light: RGBA(0x1E8A5A))
    static let channelBlue = Color(dark: RGBA(0x6FA8FF), light: RGBA(0x2F6FD6))

    // image area: stays dark in both appearances
    static let canvas = Color(dark: RGBA(0x08080A), light: RGBA(0x0D0D10))
    static let canvasChrome = Color(RGBA(0x18191D, alpha: 0.86))
    static let canvasLine = Color(RGBA(0x40434B))
    static let onCanvas = Color(RGBA(0xE7E8EB))
    static let alignpoint = Color(RGBA(0xFF4D4D))
    static let alignpointRing = Color(RGBA(0xFFD84A))
    static let alignpointShift = Color(RGBA(0x57E389))
}

/// Sizes and radii of tokens.css and the bar heights of docs/design.md ("Fensteraufbau").
enum SPMetrics {
    static let radius: CGFloat = 6  // --r: buttons, segments
    static let radiusSmall: CGFloat = 4  // --r-s: fields, menu items
    static let radiusLarge: CGFloat = 8  // --r-l: menus, floating cards, zoom bar
    static let buttonHeight: CGFloat = 28  // --h-btn
    static let fieldHeight: CGFloat = 26  // --h-field
    static let sideWidth: CGFloat = 272  // --side: left panel
    static let sideWidthRight: CGFloat = 248  // --side-right: Functions panel (Wavelets)

    static let stepBarHeight: CGFloat = 40
    static let frameBarHeight: CGFloat = 40
    static let statusBarHeight: CGFloat = 28
    static let panelPadding: CGFloat = 12
    static let imageMargin: CGFloat = 12  // fit margin around the image
    static let minWindowSize = CGSize(width: 1120, height: 600)

    static let fontSize: CGFloat = 13  // --fs
    static let fontSizeSmall: CGFloat = 12  // --fs-s
}

extension Font {
    /// The system font at the base size of the design (13 pt); the digits do not jump when a value changes.
    static let sp = Font.system(size: SPMetrics.fontSize).monospacedDigit()
    static let spSmall = Font.system(size: SPMetrics.fontSizeSmall).monospacedDigit()
    static func sp(_ size: CGFloat, _ weight: Font.Weight = .regular) -> Font {
        .system(size: size, weight: weight).monospacedDigit()
    }
}
