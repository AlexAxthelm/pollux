package com.alexaxthelm.pollux.data.database.mapper

import com.alexaxthelm.pollux.domain.model.Episode
import kotlin.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import com.alexaxthelm.pollux.data.database.Episode as DbEpisode

class EpisodeMapperTest {

    @Test
    fun should_RoundTripAllFields_When_AllFieldsPresent() {
        val episode = Episode(
            id = "e1",
            podcastId = "p1",
            title = "Episode 1",
            description = "First episode",
            audioUrl = "https://example.com/ep1.mp3",
            artworkUrl = "https://example.com/ep1.png",
            publishDate = Instant.fromEpochMilliseconds(1700000000000),
            duration = 45.minutes,
            episodeNumber = 1,
            isPlayed = true,
            playPositionSeconds = 120,
            isDownloaded = true,
            localPath = "/downloads/ep1.mp3",
        )

        val db = EpisodeMapper.fromDomain(episode)
        val roundTripped = EpisodeMapper.toDomain(db)

        assertEquals(episode, roundTripped)
    }

    @Test
    fun should_HandleNullFields_When_OptionalFieldsAreNull() {
        val episode = Episode(
            id = "e2",
            podcastId = "p1",
            title = "Minimal Episode",
            audioUrl = "https://example.com/ep2.mp3",
            publishDate = Instant.fromEpochMilliseconds(1700000000000),
            duration = 30.minutes,
        )

        val db = EpisodeMapper.fromDomain(episode)
        val roundTripped = EpisodeMapper.toDomain(db)

        assertEquals(episode, roundTripped)
        assertNull(roundTripped.description)
        assertNull(roundTripped.artworkUrl)
        assertNull(roundTripped.episodeNumber)
        assertNull(roundTripped.localPath)
    }

    @Test
    fun should_ConvertDurationCorrectly_When_MappingToDb() {
        val episode = Episode(
            id = "e3",
            podcastId = "p1",
            title = "Duration Test",
            audioUrl = "https://example.com/ep3.mp3",
            publishDate = Instant.fromEpochMilliseconds(1700000000000),
            duration = 90.seconds,
        )

        val db = EpisodeMapper.fromDomain(episode)

        assertEquals(90L, db.durationSeconds)
    }

    @Test
    fun should_ConvertBooleansCorrectly_When_MappingBothDirections() {
        val db = DbEpisode(
            id = "e4",
            podcastId = "p1",
            title = "Boolean Test",
            description = null,
            audioUrl = "https://example.com/ep4.mp3",
            artworkUrl = null,
            publishDateEpochMillis = 1700000000000,
            durationSeconds = 600,
            episodeNumber = null,
            isPlayed = 1L,
            playPositionSeconds = 0,
            isDownloaded = 0L,
            localPath = null,
        )

        val domain = EpisodeMapper.toDomain(db)

        assertEquals(true, domain.isPlayed)
        assertEquals(false, domain.isDownloaded)
    }
}
