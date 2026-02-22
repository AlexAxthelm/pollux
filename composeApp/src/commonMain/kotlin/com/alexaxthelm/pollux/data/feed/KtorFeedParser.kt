package com.alexaxthelm.pollux.data.feed

import com.alexaxthelm.pollux.domain.feed.FeedParseException
import com.alexaxthelm.pollux.domain.feed.FeedParser
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import io.ktor.client.HttpClient
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.isSuccess
import nl.adaptivity.xmlutil.EventType
import nl.adaptivity.xmlutil.XmlStreaming

class KtorFeedParser(private val httpClient: HttpClient) : FeedParser {

    override suspend fun parse(url: String): ParsedFeed {
        val response = try {
            httpClient.get(url)
        } catch (e: Exception) {
            throw FeedParseException("Failed to fetch feed: $url", e)
        }
        if (!response.status.isSuccess()) {
            throw FeedParseException("HTTP ${response.status.value} fetching feed: $url")
        }
        val xml = response.bodyAsText()
        return parseXml(url, xml)
    }

    internal fun parseXml(feedUrl: String, xml: String): ParsedFeed {
        val reader = XmlStreaming.newReader(xml)

        // Advance to first START_ELEMENT to determine feed format
        while (reader.hasNext()) {
            if (reader.next() == EventType.START_ELEMENT) break
        }
        if (!reader.hasNext() && reader.eventType != EventType.START_ELEMENT) {
            throw FeedParseException("Empty or non-XML content at: $feedUrl")
        }

        return when (val root = reader.localName) {
            "rss" -> RssFeedParser.parse(feedUrl, reader)
            "feed" -> AtomFeedParser.parse(feedUrl, reader)
            else -> throw FeedParseException("Unknown feed format (root element: <$root>) at: $feedUrl")
        }
    }
}
