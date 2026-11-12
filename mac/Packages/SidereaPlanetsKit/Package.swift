// swift-tools-version: 6.4
import PackageDescription

// SidereaPlanetsKit: the Swift side of the Rust core.
//   SidereaPlanetsFFI       the static Rust library and its C header (XCFramework, built by scripts/build-ffi.sh)
//   SidereaPlanetsBindings  the Swift file UniFFI generates from crates/ffi (also written by scripts/build-ffi.sh)
//   SidereaPlanetsKit       the thin layer the app uses: typed async calls, progress, cancel, CGImage
// The XCFramework and the generated Swift file are not in Git; run scripts/build-ffi.sh first.
let package = Package(
    name: "SidereaPlanetsKit",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "SidereaPlanetsKit", targets: ["SidereaPlanetsKit"]),
    ],
    targets: [
        .binaryTarget(name: "SidereaPlanetsFFI", path: "SidereaPlanetsFFI.xcframework"),
        .target(
            name: "SidereaPlanetsBindings",
            dependencies: ["SidereaPlanetsFFI"],
            // Generated code: it is not ours to bring to Swift 6 strictness.
            swiftSettings: [.swiftLanguageMode(.v5)]
        ),
        .target(
            name: "SidereaPlanetsKit",
            dependencies: ["SidereaPlanetsBindings"]
        ),
        .testTarget(
            name: "SidereaPlanetsKitTests",
            dependencies: ["SidereaPlanetsKit"]
        ),
    ]
)
