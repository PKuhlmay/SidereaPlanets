import CoreGraphics
import SidereaPlanetsKit
import Testing

@testable import SidereaPlanets

/// The pure parts of Alignpoints, Align and Limit: what is sent to the core and what the chart and the Framelist make of its
/// answers. The cases follow app/src/logic/align.test.ts, qualityChart.test.ts and chart.test.ts.
@Suite struct AlignLogicTests {
    private static let quality = [92.1, 88.4, 97.3, 85.0, 90.6, 99.2, 83.7, 95.8, 87.2, 100.0]

    private func frames(_ qualities: [Double], checked: Bool = true) -> [FrameEntry] {
        qualities.enumerated().map { FrameEntry(index: $0.offset, checked: checked, quality: $0.element) }
    }

    private func options(_ mode: LimitMode, percent: Int = 80, count: Int = 200) -> AlignOptions {
        var options = AlignOptions()
        options.limitMode = mode
        options.limitPercent = percent
        options.limitCount = count
        return options
    }

    // MARK: Defaults and what is sent

    @Test func defaultsAreTheOnesOfthe reference program() {
        let options = AlignOptions()
        #expect(options.minDistance == 30 && options.edgeDistance == 20)
        #expect(options.intensity == .default && options.lo == 30 && options.hi == 230)
        #expect(options.boxSize == 30 && options.maxMovement == 5)
        #expect(options.limitMode == .lowestQuality && options.limitPercent == 80 && options.limitCount == 200)
        #expect(ShowOptions().alignpoints && !ShowOptions().roi && !ShowOptions().framelist)
    }

    @Test func pointSettingsCarryTheDefaultsTheRegionAndChangedValues() {
        let defaults = pointSettings(AlignOptions(), roi: nil)
        #expect(defaults == PointSettings(
            minDistance: 30, edgeDistance: 20, maxMovement: 5, roi: nil, intensity: .default, lo: 30, hi: 230, boxSize: 30))
        var changed = AlignOptions()
        changed.minDistance = 45
        changed.intensity = .area3x3
        let roi = RoiSettings(x: 100, y: 50, width: 400, height: 300)
        let settings = pointSettings(changed, roi: roi)
        #expect(settings.roi == roi && settings.minDistance == 45 && settings.intensity == .area3x3)
    }

    @Test func alignSettingsSendBoxSizeMovementAndRegion() {
        let roi = RoiSettings(x: 1, y: 2, width: 30, height: 40)
        #expect(alignSettings(AlignOptions(), roi: roi) == AlignSettings(boxSize: 30, maxMovement: 5, roi: roi))
    }

    @Test func typedValuesAreHeldToWhatTheCoreCanTake() {
        #expect(wholeNumber(-4, minimum: 0) == 0)
        #expect(wholeNumber(0, minimum: 1) == 1)
        #expect(wholeNumber(12, minimum: 1) == 12)
        var options = AlignOptions()
        options.boxSize = 0
        options.minDistance = -3
        #expect(alignSettings(options, roi: nil).boxSize == 1)
        #expect(pointSettings(options, roi: nil).minDistance == 0)
    }

    @Test func limitTakesThePercentFieldForPercentModesAndTheCountFieldForCountModes() {
        #expect(limitSettings(options(.lowestQuality)) == .lowestQuality(value: 80))
        #expect(limitSettings(options(.bestPercent, percent: 30)) == .bestPercent(value: 30))
        #expect(limitSettings(options(.bestCount, count: 12)) == .bestCount(value: 12))
        #expect(limitSettings(options(.framesPerPoint, count: 0)) == .framesPerPoint(value: 1))
    }

    // MARK: Answers

    @Test func qualityIsShownInPercentOfTheBestFrame() {
        #expect(qualityPercent([50, 25, 100]) == [50, 25, 100])
        #expect(qualityPercent([0, 0]) == [0, 0])
        #expect(qualityPercent([]).isEmpty)
    }

    @Test func aRegionIsKeptInsideTheImageWithWholePixels() {
        let image = CGSize(width: 1024, height: 680)
        let rect = rectBetween(CGPoint(x: 500.4, y: 300.6), CGPoint(x: 100, y: 50), image: image)
        #expect(rect == RoiSettings(x: 100, y: 50, width: 400, height: 251))
        let outside = rectBetween(CGPoint(x: -30, y: -5), CGPoint(x: 2000, y: 900), image: image)
        #expect(outside == RoiSettings(x: 0, y: 0, width: 1024, height: 680))
    }

    @Test func aDragByAccidentIsNoRegion() {
        #expect(!isRegion(RoiSettings(x: 0, y: 0, width: 7, height: 100)))
        #expect(!isRegion(RoiSettings(x: 0, y: 0, width: 100, height: 3)))
        #expect(isRegion(RoiSettings(x: 0, y: 0, width: 8, height: 8)))
    }

    @Test func aRegionNeedsRoomForPointsInsideTheEdgeDistance() {
        #expect(roomForPoints(RoiSettings(x: 0, y: 0, width: 41, height: 41), edgeDistance: 20))
        #expect(!roomForPoints(RoiSettings(x: 0, y: 0, width: 40, height: 100), edgeDistance: 20))
        #expect(!roomForPoints(RoiSettings(x: 0, y: 0, width: 100, height: 40), edgeDistance: 20))
    }

    @Test func limitReportsTheStackSizeAndTheWeakestKeptQuality() {
        let both = limitMessages(kept: 8, of: 10, keptQualities: [100, 87.2, 99])
        #expect(both.action == "Keeping 8 of 10 frames")
        #expect(both.result == "8 frames will be stacked · weakest kept: 87%")
        #expect(limitMessages(kept: 3, of: 10, keptQualities: []).result == "3 frames will be stacked")
    }

    @Test func timeLeftFollowsElapsedTimeAndProgress() {
        #expect(abs(timeLeft(elapsed: 4, progress: 0.4) - 6) < 1e-9)
        #expect(timeLeft(elapsed: 4, progress: 0) == 0)
        #expect(timeLeft(elapsed: 10, progress: 1) == 0)
    }

    @Test func theTimeUsedDoesNotCountThePause() {
        let t0 = ContinuousClock.now
        var clock = JobClock(now: t0)
        #expect(clock.used(now: t0 + .seconds(4)) == 4)
        clock.pause(now: t0 + .seconds(4))
        #expect(clock.used(now: t0 + .seconds(10)) == 4)
        clock.resume(now: t0 + .seconds(10))
        #expect(clock.used(now: t0 + .seconds(13)) == 7)
    }

    // MARK: Framelist

    @Test func theFrameListNamesTheFramesAndPutsTheBestFirstAfterAlign() {
        var data = AlignData(frameCount: 3)
        #expect(data.listed.map(\.name) == ["Frame 1", "Frame 2", "Frame 3"])
        data.frames[0].quality = 90
        data.frames[1].quality = 100
        data.frames[2].quality = 90
        #expect(data.listed.map(\.index) == [1, 0, 2])  // equal qualities keep the order of the recording
        #expect(data.bestQuality == 100)
    }

    @Test func theBestFrameIsTheFirstOfTheRankingAndNoneBeforeAlign() {
        var data = AlignData(frameCount: 3)
        #expect(data.bestFrame == nil)
        data.frames[0].quality = 90
        data.frames[1].quality = 100
        data.frames[2].quality = 100
        #expect(data.bestFrame == 1)  // equal qualities: the earlier frame
        data.frames[0].quality = 100
        #expect(data.bestFrame == 0)
        data.frames[0].checked = false  // the check marks do not change the ranking
        #expect(data.bestFrame == 0)
    }

    @Test func theBestFrameHintCountsFromOneLikeTheFrameBar() {
        #expect(bestFrameHint(frame: 294) == "Best frame: 295. Run Set Alignpoints and Align again to line all frames up on this one.")
        #expect(bestFrameHint(frame: 0).hasPrefix("Best frame: 1. "))
    }

    @Test func discardingTheRatingTakesBackTheBestFrameHint() {
        var data = AlignData(frameCount: 2)
        data.bestFrameHint = 1
        data.discardRating()
        #expect(data.bestFrameHint == nil)
    }

    @Test func discardingTheRatingChecksAllFramesAgain() {
        var data = AlignData(frameCount: 2)
        data.frames[0].quality = 100
        data.frames[0].difference = 3
        data.frames[1].checked = false
        data.discardRating()
        #expect(data.frames.allSatisfy { $0.checked && $0.quality == nil && $0.difference == nil })
        #expect(!data.isRated)
    }

    // MARK: Quality chart (as limit_frames in crates/core)

    @Test func keptCountFollowsTheLimitMode() {
        let sorted = Self.quality.sorted(by: >)
        #expect(keptCount(sorted: sorted, options: options(.lowestQuality, percent: 90)) == 6)
        #expect(keptCount(sorted: sorted, options: options(.lowestQuality, percent: 0)) == 10)
        #expect(keptCount(sorted: [100, 90, 80], options: options(.lowestQuality, percent: 90)) == 2)  // sits exactly on the limit
        #expect(keptCount(sorted: [], options: options(.bestCount, count: 5)) == 0)
        #expect(keptCount(sorted: sorted, options: options(.bestPercent, percent: 25)) == 3)  // rounds up
        #expect(keptCount(sorted: sorted, options: options(.bestCount, count: 4)) == 4)
        #expect(keptCount(sorted: sorted, options: options(.bestCount, count: 200)) == 10)
        #expect(keptCount(sorted: sorted, options: options(.bestCount, count: 0)) == 1)
        #expect(keptCount(sorted: sorted, options: options(.framesPerPoint, count: 2)) == nil)
    }

    @Test func theChartSortsBarsBestFirstAndMarksTheKeptFrames() {
        let chart = qualityChart(frames: frames(Self.quality), options: options(.lowestQuality, percent: 90), maxBars: 240)
        #expect(chart.bars.count == 10)
        #expect(chart.bars[0] == QualityBar(height: 1, kept: true))
        #expect(chart.bars.map(\.kept) == [true, true, true, true, true, true, false, false, false, false])
        let heights = chart.bars.map(\.height)
        #expect(heights == heights.sorted(by: >))
    }

    @Test func theLimitLineIsHorizontalForLowestQualityAndVerticalAfterTheLastKeptFrameOtherwise() {
        let level = qualityChart(frames: frames(Self.quality), options: options(.lowestQuality, percent: 90), maxBars: 240)
        #expect(level.line == .level(0.9) && level.value == "90%")
        let after = qualityChart(frames: frames(Self.quality), options: options(.bestCount, count: 4), maxBars: 240)
        #expect(after.line == .after(0.4) && after.value == "4")
    }

    @Test func theChartFollowsAChangedValueAtOnce() {
        #expect(qualityChart(frames: frames(Self.quality), options: options(.bestPercent, percent: 50), maxBars: 240).kept == 5)
        #expect(qualityChart(frames: frames(Self.quality), options: options(.bestPercent, percent: 80), maxBars: 240).kept == 8)
    }

    @Test func framesPerPointShowsTheFramesTheLastLimitKept() {
        var list = frames(Self.quality)
        for i in list.indices { list[i].checked = i % 2 == 0 }
        #expect(qualityChart(frames: list, options: options(.framesPerPoint, count: 1), maxBars: 240).kept == 5)
    }

    @Test func moreFramesThanRoomShareABarThatCountsAsKeptWhenHalfOfItIs() {
        let many = frames((0..<1000).map { 100 - Double($0) * 0.05 })
        let chart = qualityChart(frames: many, options: options(.bestCount, count: 500), maxBars: 240)
        #expect(chart.bars.count == 200)
        #expect(chart.bars.first?.kept == true && chart.bars.last?.kept == false)
        #expect(chart.kept == 500)

        let joined = qualityChart(frames: frames([100, 50, 40, 20]), options: options(.bestCount, count: 3), maxBars: 2)
        #expect(joined.bars.map(\.height) == [0.75, 0.3])
        #expect(joined.bars.map(\.kept) == [true, true])
        let fewer = qualityChart(frames: frames([100, 50, 40, 20]), options: options(.bestCount, count: 1), maxBars: 2)
        #expect(fewer.bars.map(\.kept) == [true, false])
    }

    @Test func eachBarIsTheShareOfTheBestQuality() {
        let chart = qualityChart(frames: frames([50, 25]), options: options(.bestCount, count: 2), maxBars: 240)
        #expect(chart.bars.map(\.height) == [1, 0.5])
    }

    @Test func theChartDescribesItselfAndIsEmptyBeforeAlign() {
        let chart = qualityChart(frames: frames(Self.quality), options: options(.bestCount, count: 8), maxBars: 240)
        #expect(chart.label == "Quality of 10 frames, best first. 8 are kept.")
        let empty = qualityChart(frames: [FrameEntry(index: 0)], options: AlignOptions(), maxBars: 240)
        #expect(empty.bars.isEmpty && empty.line == nil)
    }

    // MARK: Registrationgraph

    @Test func aSeriesSpreadsFromLeftToRightWithItsMaximumAtTheTop() {
        let box = CGRect(x: 8, y: 10, width: 100, height: 80)
        #expect(seriesPoints([50, 100, 25], in: box) == [CGPoint(x: 8, y: 50), CGPoint(x: 58, y: 10), CGPoint(x: 108, y: 70)])
        #expect(seriesPoints([2, 4], in: box) == seriesPoints([20, 40], in: box))
        #expect(seriesPoints([], in: box).isEmpty)
        #expect(seriesPoints([0, 0], in: box).isEmpty)
        #expect(seriesPoints([5], in: box) == [CGPoint(x: 8, y: 10)])
    }
}
