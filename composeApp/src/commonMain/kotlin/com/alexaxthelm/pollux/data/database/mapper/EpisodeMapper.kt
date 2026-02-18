package com.alexaxthelm.pollux.data.database.mapper

import com.alexaxthelm.pollux.domain.model.Episode
import kotlin.time.Instant
import kotlin.time.Duration.Companion.seconds
import com.alexaxthelm.pollux.data.database.Episode as DbEpisode

object EpisodeMapper {

    fun toDomain(db: DbEpisode): Episode = Episode(
        id = db.id,
        podcastId = db.podcastId,
        title = db.title,
        description = db.description,
        audioUrl = db.audioUrl,
        artworkUrl = db.artworkUrl,
        publishDate = Instant.fromEpochMilliseconds(db.publishDateEpochMillis),
        duration = db.durationSeconds.seconds,
        episodeNumber = db.episodeNumber?.toInt(),
        isPlayed = db.isPlayed != 0L,
        playPositionSeconds = db.playPositionSeconds.toInt(),
        isDownloaded = db.isDownloaded != 0L,
        localPath = db.localPath,
    )

    fun fromDomain(episode: Episode): DbEpisode = DbEpisode(
        id = episode.id,
        podcastId = episode.podcastId,
        title = episode.title,
        description = episode.description,
        audioUrl = episode.audioUrl,
        artworkUrl = episode.artworkUrl,
        publishDateEpochMillis = episode.publishDate.toEpochMilliseconds(),
        durationSeconds = episode.duration.inWholeSeconds,
        episodeNumber = episode.episodeNumber?.toLong(),
        isPlayed = if (episode.isPlayed) 1L else 0L,
        playPositionSeconds = episode.playPositionSeconds.toLong(),
        isDownloaded = if (episode.isDownloaded) 1L else 0L,
        localPath = episode.localPath,
    )
}
