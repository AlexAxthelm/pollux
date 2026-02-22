package com.alexaxthelm.pollux.data.feed

import nl.adaptivity.xmlutil.XmlStreaming
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertNotNull
import kotlin.time.Duration
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import kotlin.time.Instant

class RssFeedParserTest {

    private fun parseFixture(name: String) = javaClass.classLoader
        .getResourceAsStream("feed/$name")!!
        .bufferedReader()
        .readText()
        .let { xml ->
            val reader = XmlStreaming.newReader(xml)
            // Advance past the START_DOCUMENT and the root <rss> element
            reader.next() // START_DOCUMENT or first event
            while (reader.hasNext()) {
                val evt = reader.next()
                if (evt == nl.adaptivity.xmlutil.EventType.START_ELEMENT) break
            }
            RssFeedParser.parse("https://example.com/feed.xml", reader)
        }

    @Test
    fun should_ParseAllChannelFields_When_FullRssFeed() {
        val feed = parseFixture("rss_full.xml")

        assertEquals("Full RSS Podcast", feed.title)
        assertEquals("Jane Doe", feed.author)
        assertEquals("A podcast with all fields present.", feed.description)
        assertEquals("https://example.com/artwork.jpg", feed.artworkUrl)
        assertEquals("https://example.com/feed.xml", feed.feedUrl)
        assertEquals(3, feed.episodes.size)
    }

    @Test
    fun should_ParseAllEpisodeFields_When_FullRssFeed() {
        val feed = parseFixture("rss_full.xml")
        val ep = feed.episodes[0]

        assertEquals("episode-one-guid", ep.guid)
        assertEquals("Episode One", ep.title)
        assertEquals("First episode description.", ep.description)
        assertEquals("https://example.com/ep1.mp3", ep.audioUrl)
        assertEquals("https://example.com/ep1-art.jpg", ep.artworkUrl)
        assertEquals(Instant.parse("2006-01-02T15:04:05Z"), ep.publishDate)
        assertEquals(1.hours + 2.minutes + 3.seconds, ep.duration)
        assertEquals(1, ep.episodeNumber)
    }

    @Test
    fun should_ParseMinimalFeed_When_OnlyRequiredFieldsPresent() {
        val feed = parseFixture("rss_minimal.xml")

        assertEquals("Minimal Podcast", feed.title)
        assertNull(feed.author)
        assertNull(feed.description)
        assertNull(feed.artworkUrl)
        assertEquals(1, feed.episodes.size)

        val ep = feed.episodes[0]
        assertEquals("https://example.com/ep1.mp3", ep.audioUrl)
        assertEquals(Duration.ZERO, ep.duration)
        assertNull(ep.publishDate)
    }

    @Test
    fun should_FallbackGuidToEnclosureUrl_And_DropEpisodesWithoutEnclosure_When_MissingFields() {
        val feed = parseFixture("rss_missing_fields.xml")

        // No enclosure episode should be dropped → only 2 valid episodes
        assertEquals(2, feed.episodes.size)

        val noGuidEp = feed.episodes.find { it.audioUrl == "https://example.com/ep-no-guid.mp3" }
        assertNotNull(noGuidEp)
        assertEquals("https://example.com/ep-no-guid.mp3", noGuidEp.guid)
    }

    @Test
    fun should_ReturnNullPublishDate_And_ZeroDuration_When_DatesAreMalformed() {
        val feed = parseFixture("rss_malformed_dates.xml")
        val badEp = feed.episodes.find { it.guid == "bad-date-guid" }

        assertNotNull(badEp)
        assertNull(badEp.publishDate)
        assertEquals(Duration.ZERO, badEp.duration)
    }

    @Test
    fun should_KeepFirstOccurrence_When_DuplicateGuids() {
        val feed = parseFixture("rss_duplicate_guids.xml")

        // Only 2 distinct guids
        assertEquals(2, feed.episodes.size)
        val shared = feed.episodes.find { it.guid == "shared-guid" }
        assertNotNull(shared)
        // First occurrence wins
        assertEquals("https://example.com/original.mp3", shared.audioUrl)
    }
}
