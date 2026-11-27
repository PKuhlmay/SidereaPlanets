import Foundation

/// A pixel of the shown image, counted from its top left corner.
nonisolated struct ImagePixel: Equatable, Sendable {
    let x: Int
    let y: Int
}

/// The status bar field for the pointer in the Wavelets and Save steps: "X=200 Y=201 Stack=10", empty outside the image. Same text
/// as `onHover` in app/src/App.vue.
nonisolated func pointerPositionText(_ pixel: ImagePixel?, stackFrames: Int?) -> String {
    guard let pixel else { return "" }
    return "X=\(pixel.x) Y=\(pixel.y)" + (stackFrames.map { " Stack=\($0)" } ?? "")
}

/// "0.1.0 · a4e826b · 2026-11-13 13:38": version, commit and build time in local time to the minute, leaving out what is not known.
/// Same text as `buildLabel` in app/src/logic/buildInfo.ts.
nonisolated func buildLabel(version: String, commit: String?, builtAt: Date?, timeZone: TimeZone = .current) -> String {
    var parts = [version]
    if let commit, !commit.isEmpty { parts.append(commit) }
    if let builtAt {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.timeZone = timeZone
        formatter.dateFormat = "yyyy-MM-dd HH:mm"
        parts.append(formatter.string(from: builtAt))
    }
    return parts.filter { !$0.isEmpty }.joined(separator: " · ")
}

/// The build of the running app for the right end of the status bar, like Siderea's BuildInfo. The version is `MARKETING_VERSION`,
/// the commit `SP_BUILD_COMMIT` (filled in by scripts/deploy-mac.sh and scripts/build-mac-release.sh, empty in a build from Xcode), the
/// build time the modification date of the executable, which the linker and the code signature write when the app is built.
enum BuildInfo {
    static let label = buildLabel(
        version: Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "",
        commit: Bundle.main.object(forInfoDictionaryKey: "SPBuildCommit") as? String,
        builtAt: Bundle.main.executableURL.flatMap {
            try? $0.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate
        }
    )
}
