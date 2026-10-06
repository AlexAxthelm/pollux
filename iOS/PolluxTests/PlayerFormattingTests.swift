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
}
