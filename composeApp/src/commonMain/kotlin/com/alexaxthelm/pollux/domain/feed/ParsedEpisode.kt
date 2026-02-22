package com.alexaxthelm.pollux.domain.feed

import kotlin.time.Duration
import kotlin.time.Instant

data class ParsedEpisode(
    val guid: String,
    val title: String,
    val description: String?,
    val audioUrl: String,
    val artworkUrl: String?,
    val publishDate: Instant?,
    val duration: Duration,
    val episodeNumber: Int?,
)
