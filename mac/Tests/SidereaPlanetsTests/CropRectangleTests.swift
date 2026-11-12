import SidereaPlanetsKit
import Testing
@testable import SidereaPlanets

@MainActor
struct CropRectangleTests {
    private let width = 1000, height = 800

    @Test func thirdsLinesDivideTheRectangleIntoNineEqualFields() {
        let rect = CropRectangle(minX: 100, minY: 50, maxX: 400, maxY: 350)
        let lines = rect.thirdsLines
        #expect(lines.xs == [200, 300])
        #expect(lines.ys == [150, 250])
    }

    @Test func thirdsLinesOfASideNotDivisibleByThreeAreNotRounded() {
        let rect = CropRectangle(minX: 10, minY: 20, maxX: 21, maxY: 30)
        let lines = rect.thirdsLines
        #expect(lines.xs.count == 2 && lines.ys.count == 2)
        #expect(abs(lines.xs[0] - (10 + 11.0 / 3)) < 1e-9 && abs(lines.xs[1] - (10 + 22.0 / 3)) < 1e-9)
        #expect(abs(lines.ys[0] - (20 + 10.0 / 3)) < 1e-9 && abs(lines.ys[1] - (20 + 20.0 / 3)) < 1e-9)
    }

    @Test func theCentreCrossKeepsItsFullArmOnALargeRectangle() {
        // 700 × 720: a quarter of the shorter side is 175, far above the 10 requested.
        let rect = CropRectangle(minX: 100, minY: 50, maxX: 800, maxY: 770)
        #expect(rect.centreCrossArm(10) == 10)
    }

    @Test func theCentreCrossArmIsCappedAtAQuarterOfTheShorterSide() {
        // 20 × 12: a quarter of 12 is 3.
        #expect(CropRectangle(minX: 0, minY: 0, maxX: 20, maxY: 12).centreCrossArm(10) == 3)
        // 13 × 9 (odd sides): a quarter of 9 is 2.25, not rounded.
        #expect(CropRectangle(minX: 5, minY: 7, maxX: 18, maxY: 16).centreCrossArm(10) == 2.25)
        // The 8 × 8 smallest rectangle: a quarter of 8 is 2.
        #expect(CropRectangle(minX: 0, minY: 0, maxX: 8, maxY: 8).centreCrossArm(10) == 2)
    }

    @Test func theCentreCrossArmEqualsTheCapExactlyAtTheBoundary() {
        // 40 × 40: the cap is 10, so 10 requested stays 10 and 11 becomes 10.
        let rect = CropRectangle(minX: 0, minY: 0, maxX: 40, maxY: 40)
        #expect(rect.centreCrossArm(10) == 10)
        #expect(rect.centreCrossArm(11) == 10)
    }

    @Test func theWholeImageIsFull() {
        let rect = CropRectangle.full(imageWidth: width, imageHeight: height)
        #expect(rect.isFull(imageWidth: width, imageHeight: height))
        #expect(!CropRectangle(minX: 1, minY: 0, maxX: width, maxY: height).isFull(imageWidth: width, imageHeight: height))
    }

    @Test func anEdgeMovesAlone() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400)
        let moved = rect.dragged(.left, byX: 30, y: 999, imageWidth: width, imageHeight: height)
        #expect(moved == CropRectangle(minX: 130, minY: 100, maxX: 500, maxY: 400))
    }

    @Test func aCornerMovesTwoEdges() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400)
        let moved = rect.dragged(.bottomRight, byX: 50, y: -20, imageWidth: width, imageHeight: height)
        #expect(moved == CropRectangle(minX: 100, minY: 100, maxX: 550, maxY: 380))
    }

    @Test func anEdgeStopsAtTheImageAndAtTheMinimumSize() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400)
        #expect(rect.dragged(.left, byX: -500, y: 0, imageWidth: width, imageHeight: height).minX == 0)
        #expect(rect.dragged(.right, byX: 9999, y: 0, imageWidth: width, imageHeight: height).maxX == width)
        // Dragging an edge past the opposite one stops short of it, so the rectangle never flips or vanishes.
        #expect(rect.dragged(.left, byX: 9999, y: 0, imageWidth: width, imageHeight: height).minX == 500 - CropRectangle.minimumSize)
        #expect(rect.dragged(.bottom, byX: 0, y: -9999, imageWidth: width, imageHeight: height).maxY == 100 + CropRectangle.minimumSize)
    }

    @Test func theBodyMovesAndStaysInsideTheImage() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400)
        #expect(rect.dragged(.body, byX: 40, y: 10, imageWidth: width, imageHeight: height) == CropRectangle(minX: 140, minY: 110, maxX: 540, maxY: 410))
        let pushed = rect.dragged(.body, byX: 9999, y: -9999, imageWidth: width, imageHeight: height)
        #expect(pushed == CropRectangle(minX: 600, minY: 0, maxX: 1000, maxY: 300))
    }

    @Test func fractionsOfAPixelRoundToWholePixels() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400)
        #expect(rect.dragged(.left, byX: 10.4, y: 0, imageWidth: width, imageHeight: height).minX == 110)
        #expect(rect.dragged(.left, byX: 10.6, y: 0, imageWidth: width, imageHeight: height).minX == 111)
    }

    @Test func aDrawnRectangleIsOrderedAndKeptInsideTheImage() throws {
        let rect = try #require(CropRectangle.drawn(fromX: 400, y: 300, toX: 100, y: 200, imageWidth: width, imageHeight: height))
        #expect(rect == CropRectangle(minX: 100, minY: 200, maxX: 400, maxY: 300))
        let clipped = try #require(CropRectangle.drawn(fromX: 900, y: 700, toX: 1500, y: 2000, imageWidth: width, imageHeight: height))
        #expect(clipped == CropRectangle(minX: 900, minY: 700, maxX: 1000, maxY: 800))
        #expect(CropRectangle.drawn(fromX: 10, y: 10, toX: 12, y: 300, imageWidth: width, imageHeight: height) == nil)
    }

    @Test func handlesAreFoundCornersFirstThenEdgesThenTheBody() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400)
        #expect(rect.handle(atX: 102, y: 98, tolerance: 6) == .topLeft)
        #expect(rect.handle(atX: 498, y: 403, tolerance: 6) == .bottomRight)
        #expect(rect.handle(atX: 300, y: 103, tolerance: 6) == .top)
        #expect(rect.handle(atX: 496, y: 250, tolerance: 6) == .right)
        #expect(rect.handle(atX: 300, y: 250, tolerance: 6) == .body)
        #expect(rect.handle(atX: 700, y: 250, tolerance: 6) == nil)
        // An edge is only grabbed along its own length.
        #expect(rect.handle(atX: 700, y: 100, tolerance: 6) == nil)
    }

    @Test func theGrabReachIsCappedAtAThirdOfTheShorterSide() {
        // 8 × 8: a third of 8 is 8 / 3 whatever the requested reach.
        let small = CropRectangle(minX: 100, minY: 100, maxX: 108, maxY: 108)
        #expect(small.effectiveTolerance(6) == 8.0 / 3)
        // 300 × 200: a third of 200 is 66.7, so a reach of 7 stays as requested.
        let large = CropRectangle(minX: 0, minY: 0, maxX: 300, maxY: 200)
        #expect(large.effectiveTolerance(7) == 7)
        // 90 × 30: the shorter side decides, not the longer one.
        let wide = CropRectangle(minX: 0, minY: 0, maxX: 90, maxY: 30)
        #expect(wide.effectiveTolerance(20) == 10)
    }

    @Test func aRectangleSmallerThanTheToleranceStillHasABody() {
        // 8 × 8 with a tolerance of 6 would be all corner and edge: the reach is capped at a third of the shorter side (8 / 3 ≈ 2.67),
        // so the middle is still the body and a point next to a corner is still that corner.
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 108, maxY: 108)
        #expect(rect.handle(atX: 104, y: 104, tolerance: 6) == .body)
        #expect(rect.handle(atX: 101, y: 101, tolerance: 6) == .topLeft)
        #expect(rect.handle(atX: 107, y: 107, tolerance: 6) == .bottomRight)
        #expect(rect.handle(atX: 104, y: 101, tolerance: 6) == .top)
    }

    // MARK: Tiny images and the whole image

    @Test func anImageSmallerThanTheMinimumSizeIsTheWholeRectangleAndStaysInside() {
        // 5 × 5 is below minimumSize (8): the whole image is the rectangle, nothing can be drawn, and no drag leaves the image.
        let rect = CropRectangle.full(imageWidth: 5, imageHeight: 5)
        #expect(rect == CropRectangle(minX: 0, minY: 0, maxX: 5, maxY: 5))
        #expect(CropRectangle.drawn(fromX: 0, y: 0, toX: 5, y: 5, imageWidth: 5, imageHeight: 5) == nil)
        #expect(rect.dragged(.body, byX: 3, y: -3, imageWidth: 5, imageHeight: 5) == rect)
        #expect(rect.dragged(.right, byX: 9999, y: 0, imageWidth: 5, imageHeight: 5).maxX == 5)
        #expect(rect.dragged(.bottom, byX: 0, y: 9999, imageWidth: 5, imageHeight: 5).maxY == 5)
        #expect(rect.dragged(.topLeft, byX: -9999, y: -9999, imageWidth: 5, imageHeight: 5) == rect)
    }

    @Test func aOneByOneImageIsTheWholeRectangleAndStaysInside() {
        let rect = CropRectangle.full(imageWidth: 1, imageHeight: 1)
        #expect(rect == CropRectangle(minX: 0, minY: 0, maxX: 1, maxY: 1))
        #expect(CropRectangle.drawn(fromX: 0, y: 0, toX: 1, y: 1, imageWidth: 1, imageHeight: 1) == nil)
        #expect(rect.dragged(.body, byX: -5, y: 5, imageWidth: 1, imageHeight: 1) == rect)
        #expect(rect.dragged(.bottomRight, byX: 50, y: 50, imageWidth: 1, imageHeight: 1) == rect)
    }

    @Test func aRectangleCoveringTheWholeImageStaysPutWhenDraggedAsABody() {
        let whole = CropRectangle(minX: 0, minY: 0, maxX: width, maxY: height)
        #expect(whole.dragged(.body, byX: 50, y: -30, imageWidth: width, imageHeight: height) == whole)
        #expect(whole.dragged(.body, byX: -9999, y: 9999, imageWidth: width, imageHeight: height) == whole)
    }

    // MARK: Fixed aspect ratio

    @Test func anAspectKnowsItsRatioInBothOrientations() throws {
        let threeByTwo = try #require(CropAspect(long: 3, short: 2))
        #expect(threeByTwo.ratio(portrait: false) == 1.5)
        #expect(abs(threeByTwo.ratio(portrait: true) - 2.0 / 3.0) < 1e-12)
        #expect(CropAspect(long: 0, short: 2) == nil)
        #expect(CropAspect(long: 2, short: 3) == nil, "the long side comes first")
        #expect(CropAspect.square.ratio(portrait: true) == 1)
    }

    @Test func reshapingKeepsTheCentreAndTheAreaAsFarAsTheImageAllows() {
        // A portrait 400 × 700 (area 280 000) becomes 2:3 portrait of about the same area: 432 × 648 around the same centre.
        let portrait = CropRectangle(minX: 200, minY: 50, maxX: 600, maxY: 750).fitted(to: .threeByTwo, imageWidth: width, imageHeight: height)
        #expect(portrait == CropRectangle(minX: 184, minY: 76, maxX: 616, maxY: 724))
        // The whole 1000 × 800 image as 3:4: the area asks for more than the width, so it fills the width and leaves 25 rows each side.
        let whole = CropRectangle(minX: 0, minY: 0, maxX: width, maxY: height).fitted(to: .threeByFour, imageWidth: width, imageHeight: height)
        #expect(whole == CropRectangle(minX: 0, minY: 25, maxX: 1000, maxY: 775))
    }

    @Test func reshapingNearAnEdgeMovesTheRectangleInsteadOfCuttingIt() {
        // 300 × 300 at the left edge becomes 1:1 → 3:2 landscape of the same area (367 × 245): the centre would push it past the edge, so it slides in.
        let rect = CropRectangle(minX: 0, minY: 100, maxX: 300, maxY: 400)
        let reshaped = rect.fitted(to: .threeByTwo, imageWidth: width, imageHeight: height)
        #expect(reshaped.minX == 0 && reshaped.width == 367 && reshaped.height == 245)
        #expect(reshaped.minY == 128 && reshaped.maxY == 373, "still around the old vertical middle")
    }

    @Test func switchingFormatsBackAndForthDoesNotShrinkTheRectangle() {
        let start = CropRectangle(minX: 200, minY: 100, maxX: 800, maxY: 700)
        let once = start.fitted(to: .square, imageWidth: width, imageHeight: height)
        let twice = once.fitted(to: .threeByTwo, imageWidth: width, imageHeight: height)
        let area = Double(start.width * start.height)
        #expect(abs(Double(twice.width * twice.height) - area) / area < 0.01)
    }

    @Test func aCornerKeepsTheRatioAndTheOppositeCorner() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 300) // 300 × 200, landscape
        let moved = rect.dragged(.bottomRight, byX: 90, y: 10, imageWidth: width, imageHeight: height, aspect: .threeByTwo)
        #expect(moved.minX == 100 && moved.minY == 100)
        #expect(moved.width == 390 && moved.height == 260, "the width the pointer asked for, the height that follows from 3:2")
        // Top-left by (60, 0) on a square: the pointer sits at (160, 100), 240 left of and 200 above the fixed corner (400, 300). The wider
        // of the two wins, so the side is 240 and the corner lands at (400 - 240, 300 - 240).
        let shrunk = rect.dragged(.topLeft, byX: 60, y: 0, imageWidth: width, imageHeight: height, aspect: .square)
        #expect(shrunk == CropRectangle(minX: 160, minY: 60, maxX: 400, maxY: 300))
    }

    @Test func anEdgeKeepsTheRatioAroundTheMiddleOfTheOtherSide() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 300) // 300 × 200, centre y = 200
        let wider = rect.dragged(.right, byX: 150, y: 0, imageWidth: width, imageHeight: height, aspect: .threeByTwo)
        #expect(wider.minX == 100 && wider.width == 450 && wider.height == 300)
        #expect(wider.minY == 50 && wider.maxY == 350, "grows evenly up and down")
        let taller = rect.dragged(.bottom, byX: 0, y: 100, imageWidth: width, imageHeight: height, aspect: .square)
        #expect(taller.minY == 100 && taller.height == 300 && taller.width == 300)
        #expect(taller.minX == 100 && taller.maxX == 400, "the width already equals the new height, so the sides stay put")
    }

    @Test func aCornerWithAFixedRatioStopsAtTheImage() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 300)
        let moved = rect.dragged(.bottomRight, byX: 9999, y: 9999, imageWidth: width, imageHeight: height, aspect: .threeByTwo)
        // The image is 800 high: 700 rows below the fixed corner at most, so 1050 × 700 would not fit the width of 900; the width wins and
        // caps the width at 900, which gives a height of 900 ÷ 1.5 = 600.
        #expect(moved == CropRectangle(minX: 100, minY: 100, maxX: 1000, maxY: 700))
    }

    @Test func aFixedRatioNeverGoesBelowTheMinimumSize() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 300)
        let moved = rect.dragged(.bottomRight, byX: -9999, y: -9999, imageWidth: width, imageHeight: height, aspect: .threeByTwo)
        #expect(moved.width >= CropRectangle.minimumSize && moved.height >= CropRectangle.minimumSize)
    }

    @Test func aDrawnRectangleWithAFixedRatioFollowsTheDragsDirection() throws {
        // A wide drag makes a landscape square-of-sorts: 1:1 from (100, 100) over 300 across and 50 down is 300 × 300.
        let square = try #require(CropRectangle.drawn(fromX: 100, y: 100, toX: 400, y: 150, imageWidth: width, imageHeight: height, aspect: .square))
        #expect(square == CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 400))
        // 3:2 landscape: 300 across needs 200 down.
        let landscape = try #require(CropRectangle.drawn(fromX: 100, y: 100, toX: 400, y: 150, imageWidth: width, imageHeight: height, aspect: .threeByTwo))
        #expect(landscape == CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 300))
        // A tall drag makes it portrait: 300 down needs 200 across; dragging up and left grows toward the pointer.
        let portrait = try #require(CropRectangle.drawn(fromX: 500, y: 500, toX: 450, y: 200, imageWidth: width, imageHeight: height, aspect: .threeByTwo))
        #expect(portrait == CropRectangle(minX: 300, minY: 200, maxX: 500, maxY: 500))
    }

    @Test func aDrawnRectangleWithAFixedRatioStaysInsideTheImage() throws {
        let rect = try #require(CropRectangle.drawn(fromX: 900, y: 700, toX: 1500, y: 720, imageWidth: width, imageHeight: height, aspect: .square))
        #expect(rect == CropRectangle(minX: 900, minY: 700, maxX: 1000, maxY: 800))
        #expect(CropRectangle.drawn(fromX: 10, y: 10, toX: 12, y: 12, imageWidth: width, imageHeight: height, aspect: .square) == nil)
    }

    // MARK: Fixed aspect ratio: minimum size and edge cases

    @Test func reshapingTheSmallestRectangleKeepsBothSidesAtTheMinimum() {
        // 8 × 8 around the centre (104, 104). 2:3 lying needs a width of 8 × 1.5 = 12 for a height of 8, so it grows to 12 × 8 instead of
        // shrinking to about 10 × 7: x = 104 - 6 = 98, y = 104 - 4 = 100.
        let square = CropRectangle(minX: 100, minY: 100, maxX: 108, maxY: 108)
        #expect(square.fitted(to: .threeByTwo, imageWidth: width, imageHeight: height) == CropRectangle(minX: 98, minY: 100, maxX: 110, maxY: 108))
        // 3:4 lying needs a width of 8 × 4/3 = 10.67 for a height of 8: 11 × 8 (the height 10.67 ÷ 4/3 is exactly 8). The odd width puts the
        // left edge at 104 - 5.5 = 98.5, which rounds away from zero to 99.
        #expect(square.fitted(to: .threeByFour, imageWidth: width, imageHeight: height) == CropRectangle(minX: 99, minY: 100, maxX: 110, maxY: 108))
    }

    // MARK: Fixed aspect ratio: dragging edges

    @Test func anEdgeStopsWhereTheOtherSideReachesTheImage() {
        // 300 × 200 near the top, centre y = 150. Growing the right edge by 500 would ask for 800 × 533, but the height can grow only to
        // 2 × 150 = 300 around that middle before it meets the top edge: 300 × 1.5 = 450 wide, y from 0 to 300.
        let top = CropRectangle(minX: 100, minY: 50, maxX: 400, maxY: 250)
        #expect(top.dragged(.right, byX: 500, y: 0, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 100, minY: 0, maxX: 550, maxY: 300))
        // Near the bottom, centre y = 600, 200 rows to the bottom edge: at most 400 high, 600 wide, y from 400 to 800.
        let bottom = CropRectangle(minX: 100, minY: 500, maxX: 400, maxY: 700)
        #expect(bottom.dragged(.right, byX: 500, y: 0, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 100, minY: 400, maxX: 700, maxY: 800))
        // The same across: centre x = 180, so the width can grow only to 360 (the left edge) and the height to 360 ÷ 1.5 = 240.
        let left = CropRectangle(minX: 30, minY: 100, maxX: 330, maxY: 300)
        #expect(left.dragged(.bottom, byX: 0, y: 400, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 0, minY: 100, maxX: 360, maxY: 340))
    }

    @Test func theLeftAndTopEdgesKeepTheRatioToo() {
        let rect = CropRectangle(minX: 300, minY: 200, maxX: 600, maxY: 400) // 300 × 200, centre (450, 300)
        // Left by 150: 450 wide from the fixed right edge 600, so 300 high around y = 300.
        #expect(rect.dragged(.left, byX: -150, y: 0, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 150, minY: 150, maxX: 600, maxY: 450))
        // Top by 100: 300 high from the fixed bottom edge 400, so 450 wide around x = 450.
        #expect(rect.dragged(.top, byX: 0, y: -100, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 225, minY: 100, maxX: 675, maxY: 400))
    }

    @Test func anEdgeWithAFixedRatioNeverShrinksBelowTheMinimumSize() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 300) // centre (250, 200)
        // 2:3 lying: the smallest is 12 × 8. The width gives 12 and the height 8 around y = 200.
        #expect(rect.dragged(.right, byX: -9999, y: 0, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 100, minY: 196, maxX: 112, maxY: 204))
        // The height gives 8 and the width 12 around x = 250.
        #expect(rect.dragged(.bottom, byX: 0, y: -9999, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 244, minY: 100, maxX: 256, maxY: 108))
    }

    // MARK: Fixed aspect ratio: standing rectangles and 3:4

    @Test func aStandingRectangleKeepsStandingWhenACornerIsDragged() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 300, maxY: 400) // 200 × 300, 2:3 standing
        // Bottom-right by (50, 30): the pointer is 250 right of and 330 below the fixed corner (100, 100). The width asks for 250, the
        // height for 330 × 2/3 = 220: 250 wide wins, and 250 ÷ 2/3 = 375 high.
        #expect(rect.dragged(.bottomRight, byX: 50, y: 30, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 100, minY: 100, maxX: 350, maxY: 475))
        // Top-left by (0, -90) around the fixed corner (300, 400): the pointer is 200 left and 390 above. 390 × 2/3 = 260 beats 200,
        // so 260 wide and 390 high: the corner lands at (40, 10).
        #expect(rect.dragged(.topLeft, byX: 0, y: -90, imageWidth: width, imageHeight: height, aspect: .threeByTwo) == CropRectangle(minX: 40, minY: 10, maxX: 300, maxY: 400))
    }

    @Test func aThreeByFourRectangleIsDrawnLyingOrStanding() throws {
        // Lying: 400 across and 100 down from (100, 100) needs 400 ÷ 4/3 = 300 down.
        let lying = try #require(CropRectangle.drawn(fromX: 100, y: 100, toX: 500, y: 200, imageWidth: width, imageHeight: height, aspect: .threeByFour))
        #expect(lying == CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400))
        // Standing, dragged up and left from (500, 500): 300 up is the larger move, so 300 × 3/4 = 225 across; it grows toward the pointer.
        let standing = try #require(CropRectangle.drawn(fromX: 500, y: 500, toX: 450, y: 200, imageWidth: width, imageHeight: height, aspect: .threeByFour))
        #expect(standing == CropRectangle(minX: 275, minY: 200, maxX: 500, maxY: 500))
    }

    @Test func aThreeByFourRectangleIsDraggedByCornerAndEdge() {
        let rect = CropRectangle(minX: 100, minY: 100, maxX: 400, maxY: 300) // 300 × 200, centre (250, 200)
        // Bottom-right by (100, 0): 400 wide, 400 ÷ 4/3 = 300 high from the fixed corner (100, 100).
        #expect(rect.dragged(.bottomRight, byX: 100, y: 0, imageWidth: width, imageHeight: height, aspect: .threeByFour) == CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400))
        // Right edge by 100: the same 400 × 300, centred on y = 200 (from 50 to 350).
        #expect(rect.dragged(.right, byX: 100, y: 0, imageWidth: width, imageHeight: height, aspect: .threeByFour) == CropRectangle(minX: 100, minY: 50, maxX: 500, maxY: 350))
        // Bottom edge by 100: 300 high from the fixed top edge 100, 300 × 4/3 = 400 wide centred on x = 250 (from 50 to 450).
        #expect(rect.dragged(.bottom, byX: 0, y: 100, imageWidth: width, imageHeight: height, aspect: .threeByFour) == CropRectangle(minX: 50, minY: 100, maxX: 450, maxY: 400))
    }

    @Test func aRectangleIsHeldInsideTheImageAtAWholeSizeRange() {
        // Bigger than the image: the whole image.
        #expect(CropRectangle(minX: -50, minY: -10, maxX: 2000, maxY: 900).clamped(imageWidth: width, imageHeight: height) == CropRectangle.full(imageWidth: width, imageHeight: height))
        // Past the right and bottom edge: slid back, the size kept.
        #expect(CropRectangle(minX: 900, minY: 700, maxX: 1100, maxY: 900).clamped(imageWidth: width, imageHeight: height) == CropRectangle(minX: 800, minY: 600, maxX: 1000, maxY: 800))
        // Smaller than the minimum: grown to it.
        #expect(CropRectangle(minX: 10, minY: 10, maxX: 12, maxY: 12).clamped(imageWidth: width, imageHeight: height) == CropRectangle(minX: 10, minY: 10, maxX: 18, maxY: 18))
        // Inside already: unchanged.
        let inside = CropRectangle(minX: 100, minY: 100, maxX: 500, maxY: 400)
        #expect(inside.clamped(imageWidth: width, imageHeight: height) == inside)
    }

    @Test func aCentredRectangleHasTheShareOfEachSideAndTheShapeOfTheFormat() {
        let free = CropRectangle.centred(share: 0.5, imageWidth: width, imageHeight: height)
        #expect(free == CropRectangle(minX: 250, minY: 200, maxX: 750, maxY: 600))
        let square = CropRectangle.centred(share: 0.5, imageWidth: width, imageHeight: height, aspect: .square)
        #expect(square.width == square.height && square.minX >= 0 && square.maxX <= width && square.minY >= 0 && square.maxY <= height)
        let fourThree = CropRectangle.centred(share: 0.8, imageWidth: width, imageHeight: height, aspect: .threeByFour)
        #expect(abs(Double(fourThree.width) / Double(fourThree.height) - 4.0 / 3.0) < 0.01)
    }

    @Test func theSizeReadsWidthTimesHeightInPixels() {
        #expect(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650).sizeText == "800 × 600 px")
        #expect(CropRectangle(minX: 100, minY: 50, maxX: 900, maxY: 650).coreRect == CropRect(x: 100, y: 50, width: 800, height: 600))
    }
}
