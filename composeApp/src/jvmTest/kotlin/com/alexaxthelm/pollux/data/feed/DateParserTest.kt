package com.alexaxthelm.pollux.data.feed

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.time.Instant

class DateParserTest {

    // RFC 2822

    @Test
    fun should_ParseRfc2822_When_FullWithDayOfWeek() {
        val result = DateParser.parseRfc2822("Mon, 02 Jan 2006 15:04:05 -0700")
        assertEquals(Instant.parse("2006-01-02T15:04:05-07:00"), result)
    }

    @Test
    fun should_ParseRfc2822_When_NoDayOfWeek() {
        val result = DateParser.parseRfc2822("02 Jan 2006 15:04:05 GMT")
        assertEquals(Instant.parse("2006-01-02T15:04:05Z"), result)
    }

    @Test
    fun should_ParseRfc2822_When_NamedTimezoneEst() {
        val result = DateParser.parseRfc2822("Mon, 02 Jan 2006 12:00:00 EST")
        assertEquals(Instant.parse("2006-01-02T12:00:00-05:00"), result)
    }

    @Test
    fun should_ParseRfc2822_When_NamedTimezonePst() {
        val result = DateParser.parseRfc2822("Mon, 02 Jan 2006 12:00:00 PST")
        assertEquals(Instant.parse("2006-01-02T12:00:00-08:00"), result)
    }

    @Test
    fun should_ReturnNull_When_Rfc2822Malformed() {
        assertNull(DateParser.parseRfc2822("not a date"))
    }

    @Test
    fun should_ReturnNull_When_Rfc2822IsNull() {
        assertNull(DateParser.parseRfc2822(null))
    }

    @Test
    fun should_ReturnNull_When_Rfc2822IsEmpty() {
        assertNull(DateParser.parseRfc2822(""))
    }

    // ISO 8601

    @Test
    fun should_ParseIso8601_When_UtcTimestamp() {
        val result = DateParser.parseIso8601("2023-10-15T14:30:00Z")
        assertEquals(Instant.parse("2023-10-15T14:30:00Z"), result)
    }

    @Test
    fun should_ParseIso8601_When_WithOffset() {
        val result = DateParser.parseIso8601("2023-10-15T14:30:00+05:30")
        assertEquals(Instant.parse("2023-10-15T14:30:00+05:30"), result)
    }

    @Test
    fun should_ReturnNull_When_Iso8601Malformed() {
        assertNull(DateParser.parseIso8601("2023-99-99"))
    }

    @Test
    fun should_ReturnNull_When_Iso8601IsNull() {
        assertNull(DateParser.parseIso8601(null))
    }
}
