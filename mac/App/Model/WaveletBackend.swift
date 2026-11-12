import SidereaPlanetsKit

/// What the Wavelets step and the Functions cards need from the Rust core. `SessionBackend` includes it; `PlanetsSession` is the real
/// one (its methods have exactly these signatures), the tests bring a fake. Defaults, Load Scheme and Save Scheme are plain
/// functions of the core (`waveletDefaultSettings`, `loadWaveletScheme`, `saveWaveletScheme`) and need no session.
protocol WaveletBackend: Sendable {
    /// The pixels of the last stack: the image the step sharpens (the core took it over when Stack finished).
    func stackedImage() async throws -> RgbaImage
    /// One area of the preview; `settings` of `nil` gives the unprocessed image.
    func waveletPreview(_ settings: WaveletSettings?, area: Region, mode: ColourMode) async throws -> WaveletPreview
    /// Do All: the whole image. Cancel and Pause reach the run through `cancel`/`pause` with `.wavelet`.
    func waveletDoAll(_ settings: WaveletSettings, mode: ColourMode, progress: ProgressHandler?) async throws -> RgbaImage
    /// Makes `image` the one the colour and tone cards work on.
    func setColorSource(_ image: RgbaImage) async throws -> SourceInfo
    func estimateRgbAlign(maxShift: UInt32) async throws -> ChannelShifts
    func applyColors(_ settings: ColorSettings) async throws -> RgbaImage
    /// Show mask of the card Masking: the colour source with the part the sharpening applies to tinted blue.
    func refineMaskOverlay(_ settings: RefineSettings) async throws -> RgbaImage
    /// Flip and Rotate: the picture flipped and rotated (a free angle makes the canvas bigger), for the image area.
    func orientPreview(_ image: RgbaImage, _ orientation: OrientationSettings) async throws -> RgbaImage
    func colorHistogram(_ settings: ColorSettings, after: Bool) async throws -> HistogramData
    func autoHistogramPoints(_ settings: ColorSettings, clip: Float) async throws -> HistogramPoints
}

extension PlanetsSession: WaveletBackend {}
