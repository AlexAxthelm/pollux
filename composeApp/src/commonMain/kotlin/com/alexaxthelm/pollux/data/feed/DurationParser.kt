package com.alexaxthelm.pollux.data.feed

import kotlin.time.Duration
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds

internal object DurationParser {
    /**
     * Parses itunes:duration values in HH:MM:SS, MM:SS, or plain seconds format.
     * Returns Duration.ZERO for null, blank, or malformed input.
     */
    fun parse(value: String?): Duration {
        if (value.isNullOrBlank()) return Duration.ZERO
        val trimmed = value.trim()
        val parts = trimmed.split(":")
        return when (parts.size) {
            1 -> parts[0].toLongOrNull()?.seconds ?: Duration.ZERO
            2 -> {
                val m = parts[0].toLongOrNull() ?: return Duration.ZERO
                val s = parts[1].toLongOrNull() ?: return Duration.ZERO
                m.minutes + s.seconds
            }
            3 -> {
                val h = parts[0].toLongOrNull() ?: return Duration.ZERO
                val m = parts[1].toLongOrNull() ?: return Duration.ZERO
                val s = parts[2].toLongOrNull() ?: return Duration.ZERO
                h.hours + m.minutes + s.seconds
            }
            else -> Duration.ZERO
        }
    }
}
