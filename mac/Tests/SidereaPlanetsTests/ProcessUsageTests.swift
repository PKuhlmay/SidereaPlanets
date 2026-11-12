import Foundation
import Testing

@testable import SidereaPlanets

struct ProcessUsageTests {
    private func sample(cpu: Double, at uptime: Double) -> ProcessSample {
        ProcessSample(cpuSeconds: cpu, memoryBytes: 0, uptime: uptime)
    }

    private let english = Locale(identifier: "en_US")

    // MARK: CPU load between two readings

    @Test("One busy core is 100 percent, several cores add up, an idle process is 0")
    func cpuPercentOfOneCore() {
        #expect(ProcessUsage.cpuPercent(from: sample(cpu: 10, at: 100), to: sample(cpu: 12, at: 102)) == 100)
        #expect(ProcessUsage.cpuPercent(from: sample(cpu: 0, at: 0), to: sample(cpu: 6, at: 2)) == 300)
        #expect(ProcessUsage.cpuPercent(from: sample(cpu: 5, at: 0), to: sample(cpu: 5, at: 1)) == 0)
    }

    @Test("Readings that are not in order give no value")
    func cpuPercentWithoutValue() {
        #expect(ProcessUsage.cpuPercent(from: sample(cpu: 1, at: 3), to: sample(cpu: 2, at: 3)) == nil)
        #expect(ProcessUsage.cpuPercent(from: sample(cpu: 1, at: 5), to: sample(cpu: 2, at: 4)) == nil)
        #expect(ProcessUsage.cpuPercent(from: sample(cpu: 9, at: 0), to: sample(cpu: 8, at: 1)) == nil)
    }

    // MARK: CPU text

    @Test("The text is whole percent of the whole machine, padded to three digits")
    func cpuReadingIsTheShareOfAllCores() {
        // Six fully busy cores of twelve are half the machine, not 600 %.
        #expect(ProcessUsage.cpuReading(600, coreCount: 12) == " 50 %")
        #expect(ProcessUsage.cpuReading(387.2, coreCount: 4) == " 97 %")
        #expect(ProcessUsage.cpuReading(25, coreCount: 8) == "  3 %")
        #expect(ProcessUsage.cpuReading(0, coreCount: 1) == "  0 %")
        #expect(ProcessUsage.cpuReading(12.5, coreCount: 1) == " 13 %")
    }

    @Test("The text never exceeds 100 percent, not even for absurd values")
    func cpuReadingIsCapped() {
        #expect(ProcessUsage.cpuReading(1250, coreCount: 12) == "100 %")
        #expect(ProcessUsage.cpuReading(1e30, coreCount: 1) == "100 %")
    }

    @Test("A value that is no reading shows nothing")
    func cpuReadingWithoutReading() {
        for missing in [nil, -1, .nan, .infinity, -.infinity] as [Double?] {
            #expect(ProcessUsage.cpuReading(missing, coreCount: 4) == nil, "\(String(describing: missing))")
        }
        #expect(ProcessUsage.cpuReading(50, coreCount: 0) == nil)
    }

    // MARK: Memory text

    @Test("Bytes are plain below one kilobyte and count in 1024s above")
    func byteTextUnits() {
        #expect(ProcessUsage.byteText(0, locale: english) == "0 B")
        #expect(ProcessUsage.byteText(1000, locale: english) == "1000 B")
        #expect(ProcessUsage.byteText(1024, locale: english) == "1.0 KB")
        #expect(ProcessUsage.byteText(1_572_864, locale: english) == "1.5 MB")
        #expect(ProcessUsage.byteText(250 * 1_048_576, locale: english) == "250 MB")
        #expect(ProcessUsage.byteText(3 * 1_073_741_824, locale: english) == "3.0 GB")
        #expect(ProcessUsage.byteText(-5, locale: english) == "0 B")
    }

    @Test("Rounding that reaches the next unit moves up a whole unit")
    func byteTextRoundsUp() {
        // 1,048,575 bytes are 1023.999 KB, which rounds to 1024.0: it reads 1.0 MB.
        #expect(ProcessUsage.byteText(1_048_575, locale: english) == "1.0 MB")
        // 99.97 KB rounds to 100.0 at one decimal; from 100 on no decimal is shown.
        #expect(ProcessUsage.byteText(102_370, locale: english) == "100 KB")
        #expect(ProcessUsage.byteText(102_300, locale: english) == "99.9 KB")
    }

    @Test("The decimal separator is the one of the locale")
    func byteTextLocale() {
        #expect(ProcessUsage.byteText(1_572_864, locale: Locale(identifier: "de_DE")) == "1,5 MB")
    }

    @Test("The text is never longer than 8 characters, so the readout keeps its width")
    func byteTextLength() {
        let samples: [Int64] = [0, 999, 1_048_575, 1_023_999_999, 1_073_741_823, 12_345_678_901, 999_999_999_999_999]
        for locale in [Locale(identifier: "en_US"), Locale(identifier: "de_DE")] {
            for bytes in samples {
                let text = ProcessUsage.byteText(bytes, locale: locale)
                #expect(text.count <= 8, "\(text) for \(bytes) in \(locale.identifier)")
            }
        }
    }

    // MARK: High load

    @Test("The load is high from 90 percent of all cores on")
    func highLoadThreshold() {
        #expect(ProcessUsage.isHigh(cpuPercent: 360, coreCount: 4))
        #expect(ProcessUsage.isHigh(cpuPercent: 400, coreCount: 4))
        #expect(ProcessUsage.isHigh(cpuPercent: 90, coreCount: 1))
        #expect(!ProcessUsage.isHigh(cpuPercent: 359.9, coreCount: 4))
        #expect(!ProcessUsage.isHigh(cpuPercent: 89.9, coreCount: 1))
    }

    @Test("No value, no cores or no reading is never a high load")
    func highLoadWithoutReading() {
        #expect(!ProcessUsage.isHigh(cpuPercent: nil, coreCount: 8))
        #expect(!ProcessUsage.isHigh(cpuPercent: 500, coreCount: 0))
        for value in [Double.infinity, .nan, -400] {
            #expect(!ProcessUsage.isHigh(cpuPercent: value, coreCount: 4), "\(value)")
        }
    }

    // MARK: Switch

    @Test("Measuring runs only while the readout is shown and the window is on screen")
    func measuringConditions() {
        #expect(ResourceSettings.isMeasuring(isShown: true, isBackground: false, isOnScreen: true))
        #expect(!ResourceSettings.isMeasuring(isShown: false, isBackground: false, isOnScreen: true))
        #expect(!ResourceSettings.isMeasuring(isShown: true, isBackground: true, isOnScreen: true))
        #expect(!ResourceSettings.isMeasuring(isShown: true, isBackground: false, isOnScreen: false))
    }

    @Test("The readout is on by default")
    func onByDefault() {
        #expect(ResourceSettings.defaultIsShown)
    }
}
