import SidereaPlanetsKit

// The crop tool (MAC-016): the shapes it offers, its state and the conversion to what the core writes. The rectangle itself and all
// the geometry of dragging are `CropRectangle`; the model methods are in `AppModel+Crop.swift`.

/// The Format choice of the crop tool: Free, or a fixed shape. A fixed shape keeps the orientation the rectangle has (see `CropAspect`).
nonisolated enum CropFormat: CaseIterable, Sendable {
    case free, square, threeByTwo, fourByThree

    var title: String {
        switch self {
        case .free: "Free"
        case .square: "1:1"
        case .threeByTwo: "3:2"
        case .fourByThree: "4:3"
        }
    }

    /// The fixed shape; `nil` is free.
    var aspect: CropAspect? {
        switch self {
        case .free: nil
        case .square: .square
        case .threeByTwo: .threeByTwo
        case .fourByThree: .threeByFour
        }
    }
}

/// What the crop tool holds: the rectangle that Save Image writes (`applied`), and while the tool is open the one being edited (`draft`).
/// Both are in pixels of the stack, which has the size of the frames; nothing is stored in the project file (the stack is not either).
nonisolated struct CropState: Equatable, Sendable {
    var format = CropFormat.free
    var isEditing = false
    var applied: CropRectangle?
    var draft: CropRectangle?
}

extension CropRectangle {
    /// The rectangle as the core takes it.
    var coreRect: CropRect {
        CropRect(x: UInt32(minX), y: UInt32(minY), width: UInt32(width), height: UInt32(height))
    }

    /// "1200 × 900 px": the size of the picture that is written.
    var sizeText: String { "\(width) × \(height) px" }
}
