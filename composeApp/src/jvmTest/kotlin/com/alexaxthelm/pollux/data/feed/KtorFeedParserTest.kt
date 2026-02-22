package com.alexaxthelm.pollux.data.feed

import com.alexaxthelm.pollux.domain.feed.FeedParseException
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull

class KtorFeedParserTest {

    private fun loadFixture(name: String) = javaClass.classLoader
        .getResourceAsStream("feed/$name")!!
        .bufferedReader()
        .readText()

    private fun parserWithFixture(name: String): KtorFeedParser {
        val xml = loadFixture(name)
        val engine = MockEngine { _ ->
            respond(
                content = xml,
                status = HttpStatusCode.OK,
                headers = headersOf("Content-Type", "application/rss+xml"),
            )
        }
        return KtorFeedParser(HttpClient(engine))
    }

    @Test
    fun should_DispatchToRssParser_When_RssFeedFetched() = runTest {
        val parser = parserWithFixture("rss_full.xml")
        val feed = parser.parse("https://example.com/feed.xml")

        assertEquals("Full RSS Podcast", feed.title)
        assertEquals(3, feed.episodes.size)
    }

    @Test
    fun should_DispatchToAtomParser_When_AtomFeedFetched() = runTest {
        val parser = parserWithFixture("atom_full.xml")
        val feed = parser.parse("https://example.com/atom.xml")

        assertEquals("Full Atom Podcast", feed.title)
        assertEquals(2, feed.episodes.size)
    }

    @Test
    fun should_ThrowFeedParseException_When_HttpError() = runTest {
        val engine = MockEngine { _ ->
            respond(content = "Not Found", status = HttpStatusCode.NotFound)
        }
        val parser = KtorFeedParser(HttpClient(engine))

        assertFailsWith<FeedParseException> {
            parser.parse("https://example.com/missing.xml")
        }
    }

    @Test
    fun should_ThrowFeedParseException_When_UnknownRootElement() = runTest {
        val unknownXml = """<?xml version="1.0"?><opml version="2.0"><head/></opml>"""
        val engine = MockEngine { _ ->
            respond(
                content = unknownXml,
                status = HttpStatusCode.OK,
                headers = headersOf("Content-Type", "text/xml"),
            )
        }
        val parser = KtorFeedParser(HttpClient(engine))

        val ex = assertFailsWith<FeedParseException> {
            parser.parse("https://example.com/opml.xml")
        }
        assertNotNull(ex.message)
    }

    @Test
    fun should_ParseXml_DirectlyWithoutHttp_When_UsingParseXmlEntryPoint() {
        val xml = loadFixture("rss_minimal.xml")
        val parser = KtorFeedParser(HttpClient(MockEngine { respond("", HttpStatusCode.OK) }))

        val feed = parser.parseXml("https://example.com/feed.xml", xml)

        assertEquals("Minimal Podcast", feed.title)
        assertEquals(1, feed.episodes.size)
    }
}
