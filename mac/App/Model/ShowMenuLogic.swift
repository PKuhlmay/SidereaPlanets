// The entries of the Show menu at the top right of the image, per step, and which of them do something yet. The ones without an effect
// are dimmed under "Not available yet" (MAC-014).

/// One check mark of the Show menu.
nonisolated enum ShowItem: Hashable, Sendable {
    case fullImage, region, alignpoints, alignData, frameList, qualityGraph, prefilter, stackGraph, stackList, previewArea

    var title: String {
        switch self {
        case .fullImage: "Show Full Image"
        case .region: "Show Region"
        case .alignpoints: "Show Alignpoints"
        case .alignData: "Show Align Data"
        case .frameList: "Show Frame List"
        case .qualityGraph: "Show Quality Graph"
        case .prefilter: "Show Prefilter"
        case .stackGraph: "Show Stack Graph"
        case .stackList: "Show Stack List"
        case .previewArea: "Show Preview Area"
        }
    }

    /// Whether the check mark changes anything on screen.
    var isAvailable: Bool {
        switch self {
        case .fullImage, .alignData, .prefilter, .stackGraph, .stackList: false
        case .region, .alignpoints, .frameList, .qualityGraph, .previewArea: true
        }
    }
}

/// The entries of the Show menu in a step, in menu order.
nonisolated func showItems(for step: ViewStep) -> [ShowItem] {
    switch step {
    case .wavelets, .save: [.fullImage, .alignpoints, .previewArea]
    case .stack: [.fullImage, .alignpoints, .stackGraph, .stackList]
    default: [.fullImage, .region, .alignpoints, .alignData, .frameList, .qualityGraph, .prefilter]
    }
}
