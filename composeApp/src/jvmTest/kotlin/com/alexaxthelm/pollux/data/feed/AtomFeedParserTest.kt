package com.alexaxthelm.pollux.data.feed

import nl.adaptivity.xmlutil.EventType
import nl.adaptivity.xmlutil.XmlStreaming
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertNotNull
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class AtomFeedParserTest {

    private fun parseFixture(name: String) = javaClass.classLoader
        .getResourceAsStream("feed/$name")!!
        .bufferedReader()
        .readText()
        .let { xml ->
            val reader = XmlStreaming.newReader(xml)
            // Advance to the <feed> root element
            while (reader.hasNext()) {
                val evt = reader.next()
                if (evt == EventType.START_ELEMENT) break
            }
            AtomFeedParser.parse("https://example.com/atom.xml", reader)
        }

    @Test
    fun should_ParseAllFeedFields_When_FullAtomFeed() {
        val feed = parseFixture("atom_full.xml")

        assertEquals("Full Atom Podcast", feed.title)
        assertEquals("John Atom", feed.author)
        assertEquals("An atom feed with all fields present.", feed.description)
        assertEquals("https://example.com/atom-artwork.jpg", feed.artworkUrl)
        assertEquals(2, feed.episodes.size)
    }

    @Test
    fun should_ParseAllEntryFields_When_FullAtomFeed() {
        val feed = parseFixture("atom_full.xml")
        val ep = feed.episodes[0]

        assertEquals("atom-entry-one", ep.guid)
        assertEquals("Atom Episode One", ep.title)
        assertEquals("First atom episode description.", ep.description)
        assertEquals("https://example.com/atom-ep1.mp3", ep.audioUrl)
        assertEquals("https://example.com/atom-ep1-art.jpg", ep.artworkUrl)
        assertEquals(Instant.parse("2023-01-15T10:00:00Z"), ep.publishDate)
        assertEquals(1.hours, ep.duration)
        assertEquals(1, ep.episodeNumber)
    }

    @Test
    fun should_ParseMinimalAtomFeed_When_OnlyRequiredFieldsPresent() {
        val feed = parseFixture("atom_minimal.xml")

        assertEquals("Minimal Atom Podcast", feed.title)
        assertNull(feed.author)
        assertEquals(1, feed.episodes.size)
        assertEquals("https://example.com/atom-ep1.mp3", feed.episodes[0].audioUrl)
    }

    @Test
    fun should_FallbackGuidToHref_And_DropEntriesWithoutEnclosure_When_MissingFields() {
        val feed = parseFixture("atom_missing_fields.xml")

        // No enclosure entry dropped → 2 valid entries
        assertEquals(2, feed.episodes.size)

        val noIdEp = feed.episodes.find { it.audioUrl == "https://example.com/atom-no-id.mp3" }
        assertNotNull(noIdEp)
        assertEquals("https://example.com/atom-no-id.mp3", noIdEp.guid)
    }
}
