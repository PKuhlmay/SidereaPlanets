import Foundation
import Testing

@testable import SidereaPlanets

@Suite struct RecentListTests {
    @Test func newestComesFirstAndTheSameSelectionIsListedOnce() {
        var list = RecentList()
        list.push(["/a/one.ser"])
        list.push(["/a/two.avi"])
        list.push(["/a/one.ser"])
        #expect(list.entries == [["/a/one.ser"], ["/a/two.avi"]])
    }

    @Test func aSequenceIsOneEntryAndOrderMatters() {
        var list = RecentList()
        list.push(["/s/1.tif", "/s/2.tif"])
        list.push(["/s/2.tif", "/s/1.tif"])
        #expect(list.entries.count == 2)
        #expect(list.entries.first == ["/s/2.tif", "/s/1.tif"])
    }

    @Test func keepsTenEntriesAndDropsTheOldest() {
        var list = RecentList()
        for number in 1...12 { list.push(["/f/\(number).ser"]) }
        #expect(list.entries.count == 10)
        #expect(list.entries.first == ["/f/12.ser"])
        #expect(list.entries.last == ["/f/3.ser"])
    }

    @Test func labelsAndFolders() {
        #expect(RecentList.label(for: ["/a/b/JUPITER.AVI"]) == "JUPITER.AVI")
        #expect(RecentList.label(for: ["/s/1.tif", "/s/2.tif", "/s/3.tif"]) == "1.tif (+2)")
        #expect(RecentList.folder(for: ["/a/b/JUPITER.AVI"]) == "/a/b")
    }

    @Test func survivesARestart() throws {
        let suite = "RecentListTests-\(UUID().uuidString)"
        let defaults = try #require(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        var list = RecentList()
        list.push(["/a/one.ser"])
        list.push(["/s/1.tif", "/s/2.tif"])
        list.save(to: defaults)
        #expect(RecentList.load(from: defaults) == list)
    }

    @Test func clearEmptiesTheList() {
        var list = RecentList(entries: [["/a/one.ser"]])
        list.clear()
        #expect(list.entries.isEmpty)
    }
}
