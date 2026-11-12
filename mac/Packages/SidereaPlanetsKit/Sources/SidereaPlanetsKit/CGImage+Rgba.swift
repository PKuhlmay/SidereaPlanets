import CoreGraphics
import Foundation

/// An RGBA8 buffer as a `CGImage` for display (alpha is ignored). `nil` if the buffer does not fit `width` x `height`.
private func makeCGImage(width: UInt32, height: UInt32, rgba: Data) -> CGImage? {
    let (w, h) = (Int(width), Int(height))
    guard w > 0, h > 0, rgba.count == w * h * 4,
          let provider = CGDataProvider(data: rgba as CFData),
          let space = CGColorSpace(name: CGColorSpace.sRGB)
    else { return nil }
    return CGImage(
        width: w, height: h, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: w * 4, space: space,
        bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipLast.rawValue),
        provider: provider, decode: nil, shouldInterpolate: false, intent: .defaultIntent
    )
}

extension RgbaImage {
    /// The image for display, `nil` if the buffer size does not match.
    public func makeCGImage() -> CGImage? {
        SidereaPlanetsKit.makeCGImage(width: width, height: height, rgba: rgba)
    }
}

extension WaveletPreview {
    /// The rendered area for display, `nil` if the buffer size does not match its region.
    public func makeCGImage() -> CGImage? {
        SidereaPlanetsKit.makeCGImage(width: region.width, height: region.height, rgba: rgba)
    }
}
