package com.alexaxthelm.pollux.data.feed

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.time.Duration
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds

class DurationParserTest {

    @Test
    fun should_ReturnZero_When_InputIsNull() {
        assertEquals(Duration.ZERO, DurationParser.parse(null))
    }

    @Test
    fun should_ReturnZero_When_InputIsBlank() {
        assertEquals(Duration.ZERO, DurationParser.parse("  "))
    }

    @Test
    fun should_ParsePlainSeconds_When_SingleSegment() {
        assertEquals(90.seconds, DurationParser.parse("90"))
    }

    @Test
    fun should_ParseMinutesAndSeconds_When_TwoSegments() {
        assertEquals(3.minutes + 45.seconds, DurationParser.parse("3:45"))
    }

    @Test
    fun should_ParseHoursMinutesSeconds_When_ThreeSegments() {
        assertEquals(1.hours + 2.minutes + 3.seconds, DurationParser.parse("1:02:03"))
    }

    @Test
    fun should_ReturnZero_When_MalformedInput() {
        assertEquals(Duration.ZERO, DurationParser.parse("abc"))
    }

    @Test
    fun should_ReturnZero_When_TooManySegments() {
        assertEquals(Duration.ZERO, DurationParser.parse("1:2:3:4"))
    }

    @Test
    fun should_ReturnZero_When_SegmentIsNotNumeric() {
        assertEquals(Duration.ZERO, DurationParser.parse("1:xx:03"))
    }
}
