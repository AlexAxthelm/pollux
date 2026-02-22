package com.alexaxthelm.pollux.data.feed

import kotlin.time.Instant

internal object DateParser {
    private val MONTHS = mapOf(
        "jan" to "01", "feb" to "02", "mar" to "03", "apr" to "04",
        "may" to "05", "jun" to "06", "jul" to "07", "aug" to "08",
        "sep" to "09", "oct" to "10", "nov" to "11", "dec" to "12",
    )

    // Offset in minutes from UTC for common named timezone abbreviations
    private val TZ_OFFSETS = mapOf(
        "GMT" to 0, "UTC" to 0, "UT" to 0,
        "EST" to -300, "EDT" to -240,
        "CST" to -360, "CDT" to -300,
        "MST" to -420, "MDT" to -360,
        "PST" to -480, "PDT" to -420,
        "Z" to 0, "A" to -60, "M" to -720, "N" to 60, "Y" to 720,
    )

    /**
     * Parses an RFC 2822 date string (as used in RSS pubDate) into an Instant.
     * Returns null on any parse failure rather than throwing.
     *
     * Examples:
     *   "Mon, 02 Jan 2006 15:04:05 -0700"
     *   "2 Jan 2006 15:04:05 GMT"
     */
    fun parseRfc2822(value: String?): Instant? {
        if (value.isNullOrBlank()) return null
        return try {
            // Strip optional day-of-week prefix ("Mon, ")
            val stripped = value.trim().let {
                val commaIdx = it.indexOf(',')
                if (commaIdx in 0..3) it.substring(commaIdx + 1).trim() else it
            }
            // Expected: DD Mon YYYY HH:MM:SS TZ
            val parts = stripped.split(Regex("\\s+"))
            if (parts.size < 5) return null

            val day = parts[0].padStart(2, '0')
            val month = MONTHS[parts[1].lowercase()] ?: return null
            val year = parts[2]
            val time = parts[3]
            val tz = parts[4]

            val offset = resolveOffset(tz) ?: return null
            val iso = "${year}-${month}-${day}T${time}${offset}"
            Instant.parse(iso)
        } catch (_: Exception) {
            null
        }
    }

    /**
     * Parses an ISO 8601 date string (as used in Atom published/updated) into an Instant.
     * Returns null on any parse failure.
     */
    fun parseIso8601(value: String?): Instant? {
        if (value.isNullOrBlank()) return null
        return try {
            Instant.parse(value.trim())
        } catch (_: Exception) {
            null
        }
    }

    /** Converts a timezone token to an ISO 8601 offset string like "+05:30" or "Z". */
    private fun resolveOffset(tz: String): String? {
        // Named abbreviation
        TZ_OFFSETS[tz]?.let { minutes ->
            if (minutes == 0) return "Z"
            val sign = if (minutes < 0) "-" else "+"
            val abs = kotlin.math.abs(minutes)
            return "${sign}${(abs / 60).toString().padStart(2, '0')}:${(abs % 60).toString().padStart(2, '0')}"
        }
        // Numeric: +HHMM or -HHMM
        if (tz.length == 5 && (tz[0] == '+' || tz[0] == '-')) {
            val sign = tz[0]
            val hh = tz.substring(1, 3)
            val mm = tz.substring(3, 5)
            if (hh.all { it.isDigit() } && mm.all { it.isDigit() }) {
                return "${sign}${hh}:${mm}"
            }
        }
        return null
    }
}
