package com.alexaxthelm.pollux.domain.model

import kotlinx.datetime.Instant
import kotlin.time.Duration

data class Episode(
    val id: String,
    val podcastId: String,
    val title: String,
    val description: String? = null,
    val audioUrl: String,
    val artworkUrl: String? = null,
    val publishDate: Instant,
    val duration: Duration,
    val episodeNumber: Int? = null,
    val isPlayed: Boolean = false,
    val playPositionSeconds: Int = 0,
    val isDownloaded: Boolean = false,
    val localPath: String? = null,
)
