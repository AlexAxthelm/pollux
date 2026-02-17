package com.alexaxthelm.pollux.data.database.mapper

import com.alexaxthelm.pollux.domain.model.Podcast
import kotlin.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import com.alexaxthelm.pollux.data.database.Podcast as DbPodcast

class PodcastMapperTest {

    @Test
    fun should_RoundTripAllFields_When_AllFieldsPresent() {
        val podcast = Podcast(
            id = "p1",
            feedUrl = "https://example.com/feed.xml",
            title = "Test Podcast",
            author = "Author",
            description = "A great podcast",
            artworkUrl = "https://example.com/art.png",
            lastRefreshed = Instant.fromEpochMilliseconds(1700000000000),
            isSubscribed = true,
        )

        val db = PodcastMapper.fromDomain(podcast)
        val roundTripped = PodcastMapper.toDomain(db)

        assertEquals(podcast, roundTripped)
    }

    @Test
    fun should_HandleNullFields_When_OptionalFieldsAreNull() {
        val podcast = Podcast(
            id = "p2",
            feedUrl = "https://example.com/feed.xml",
            title = "Minimal Podcast",
        )

        val db = PodcastMapper.fromDomain(podcast)
        val roundTripped = PodcastMapper.toDomain(db)

        assertEquals(podcast, roundTripped)
        assertNull(roundTripped.author)
        assertNull(roundTripped.description)
        assertNull(roundTripped.artworkUrl)
        assertNull(roundTripped.lastRefreshed)
    }

    @Test
    fun should_MapBooleanCorrectly_When_SubscribedIsFalse() {
        val db = DbPodcast(
            id = "p3",
            feedUrl = "https://example.com/feed.xml",
            title = "Unsubscribed",
            author = null,
            description = null,
            artworkUrl = null,
            lastRefreshedEpochMillis = null,
            isSubscribed = 0L,
        )

        val domain = PodcastMapper.toDomain(db)

        assertEquals(false, domain.isSubscribed)
    }
}
