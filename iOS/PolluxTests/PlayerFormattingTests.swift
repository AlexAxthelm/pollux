import Testing

@testable import Pollux

@Suite struct PlayerFormattingTests {
    @Test func clockUnderAnHourIsMinutesAndSeconds() {
        #expect(PlayerFormatting.clock(0) == "0:00")
        #expect(PlayerFormatting.clock(9) == "0:09")
        #expect(PlayerFormatting.clock(123) == "2:03")
    }

    @Test func clockFromAnHourIncludesHours() {
        #expect(PlayerFormatting.clock(3723) == "1:02:03")
        #expect(PlayerFormatting.clock(36000) == "10:00:00")
    }

    @Test func remainingCountsDownAndNeverGoesNegative() {
        #expect(PlayerFormatting.remaining(position: 60, duration: 3600) == "-59:00")
        #expect(PlayerFormatting.remaining(position: 4000, duration: 3600) == "-0:00")
    }

    @Test func skipLabelsCarryTheirSign() {
        #expect(PlayerFormatting.skipLabel(30, forward: true) == "+30")
        #expect(PlayerFormatting.skipLabel(15, forward: false) == "-15")
    }

    @Test func fractionIsClampedAndZeroWithoutADuration() {
        #expect(PlayerFormatting.fraction(position: 50, duration: 100) == 0.5)
        #expect(PlayerFormatting.fraction(position: 150, duration: 100) == 1)
        #expect(PlayerFormatting.fraction(position: 50, duration: nil) == 0)
        #expect(PlayerFormatting.fraction(position: 50, duration: 0) == 0)
    }

    @Test func spokenTimeIsWords() {
        #expect(PlayerFormatting.spoken(3723).contains("hour"))
    }

    @Test func spokenTimeNamesEachUnitPresent() {
        let spoken = PlayerFormatting.spoken(3723)
        #expect(spoken.contains("hour"))
        #expect(spoken.contains("minute"))
        #expect(spoken.contains("second"))
        // Under an hour there is no hours part.
        #expect(!PlayerFormatting.spoken(123).contains("hour"))
    }

    @Test func spokenTimeIsNeverEmpty() {
        for seconds: UInt32 in [0, 1, 59, 60, 3599, 3600, 86400] {
            #expect(!PlayerFormatting.spoken(seconds).isEmpty)
        }
    }

    // MARK: - Whole seconds from the system's Doubles

    @Test func wholeSecondsTruncateOrdinaryValues() {
        #expect(PlayerFormatting.wholeSeconds(0) == 0)
        #expect(PlayerFormatting.wholeSeconds(12.9) == 12)
        #expect(PlayerFormatting.wholeSeconds(7200) == 7200)
        #expect(PlayerFormatting.wholeSeconds(Double(UInt32.max)) == UInt32.max)
    }

    @Test func wholeSecondsClampNegativesToZero() {
        #expect(PlayerFormatting.wholeSeconds(-1) == 0)
        #expect(PlayerFormatting.wholeSeconds(-0.5) == 0)
        #expect(PlayerFormatting.wholeSeconds(-.infinity) == nil)
    }

    @Test func wholeSecondsRefuseWhatIsNotATime() {
        #expect(PlayerFormatting.wholeSeconds(.nan) == nil)
        #expect(PlayerFormatting.wholeSeconds(.infinity) == nil)
        #expect(PlayerFormatting.wholeSeconds(Double(UInt32.max) + 1) == nil)
        #expect(PlayerFormatting.wholeSeconds(1e300) == nil)
    }

    @Test func spokenTimeIsStableAcrossRepeatedCalls() {
        // One shared formatter must give the same answer however often it is used.
        let first = PlayerFormatting.spoken(754)
        for _ in 0 ..< 100 {
            #expect(PlayerFormatting.spoken(754) == first)
        }
    }
}
