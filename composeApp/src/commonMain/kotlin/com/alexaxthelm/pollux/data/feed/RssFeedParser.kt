package com.alexaxthelm.pollux.data.feed

import com.alexaxthelm.pollux.domain.feed.FeedParseException
import com.alexaxthelm.pollux.domain.feed.ParsedEpisode
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import nl.adaptivity.xmlutil.EventType
import nl.adaptivity.xmlutil.XmlReader
import kotlin.time.Duration

private const val ITUNES_NS = "http://www.itunes.com/dtds/podcast-1.0.dtd"

internal object RssFeedParser {
    fun parse(feedUrl: String, reader: XmlReader): ParsedFeed {
        // Advance past the <rss> root element to get into <channel>
        skipToElement(reader, "channel")

        var title = ""
        var author: String? = null
        var description: String? = null
        var artworkUrl: String? = null
        val episodes = LinkedHashMap<String, ParsedEpisode>()

        loop@ while (reader.hasNext()) {
            when (reader.next()) {
                EventType.START_ELEMENT -> when {
                    reader.localName == "title" && reader.namespaceURI.isEmpty() -> {
                        title = reader.readText()
                    }
                    reader.localName == "description" && reader.namespaceURI.isEmpty() -> {
                        if (description == null) description = reader.readText()
                    }
                    reader.localName == "managingEditor" && reader.namespaceURI.isEmpty() -> {
                        if (author == null) author = reader.readText()
                    }
                    reader.localName == "author" && reader.namespaceURI == ITUNES_NS -> {
                        author = reader.readText()
                    }
                    reader.localName == "image" && reader.namespaceURI == ITUNES_NS -> {
                        val href = reader.getAttributeValue(null, "href")
                        if (artworkUrl == null && href != null) artworkUrl = href
                        reader.skipElement()
                    }
                    reader.localName == "image" && reader.namespaceURI.isEmpty() -> {
                        val url = parseImageUrl(reader)
                        if (artworkUrl == null && url != null) artworkUrl = url
                    }
                    reader.localName == "item" -> {
                        val episode = parseItem(reader)
                        if (episode != null && !episodes.containsKey(episode.guid)) {
                            episodes[episode.guid] = episode
                        }
                    }
                    else -> reader.skipElement()
                }
                EventType.END_ELEMENT -> if (reader.localName == "channel") break@loop
                else -> Unit
            }
        }

        if (title.isEmpty()) throw FeedParseException("RSS feed missing <title>")
        return ParsedFeed(
            feedUrl = feedUrl,
            title = title,
            author = author,
            description = description,
            artworkUrl = artworkUrl,
            episodes = episodes.values.toList(),
        )
    }

    private fun parseItem(reader: XmlReader): ParsedEpisode? {
        var title = ""
        var description: String? = null
        var guid: String? = null
        var audioUrl: String? = null
        var artworkUrl: String? = null
        var publishDate = null as kotlin.time.Instant?
        var duration = Duration.ZERO
        var episodeNumber: Int? = null

        loop@ while (reader.hasNext()) {
            when (reader.next()) {
                EventType.START_ELEMENT -> when {
                    reader.localName == "title" && reader.namespaceURI.isEmpty() -> {
                        title = reader.readText()
                    }
                    reader.localName == "description" && reader.namespaceURI.isEmpty() -> {
                        description = reader.readText()
                    }
                    reader.localName == "guid" && reader.namespaceURI.isEmpty() -> {
                        guid = reader.readText()
                    }
                    reader.localName == "enclosure" && reader.namespaceURI.isEmpty() -> {
                        audioUrl = reader.getAttributeValue(null, "url")
                        reader.skipElement()
                    }
                    reader.localName == "image" && reader.namespaceURI == ITUNES_NS -> {
                        val href = reader.getAttributeValue(null, "href")
                        if (href != null) artworkUrl = href
                        reader.skipElement()
                    }
                    reader.localName == "pubDate" && reader.namespaceURI.isEmpty() -> {
                        publishDate = DateParser.parseRfc2822(reader.readText())
                    }
                    reader.localName == "duration" && reader.namespaceURI == ITUNES_NS -> {
                        duration = DurationParser.parse(reader.readText())
                    }
                    reader.localName == "episode" && reader.namespaceURI == ITUNES_NS -> {
                        episodeNumber = reader.readText().toIntOrNull()
                    }
                    else -> reader.skipElement()
                }
                EventType.END_ELEMENT -> if (reader.localName == "item") break@loop
                else -> Unit
            }
        }

        // Drop episode if no enclosure URL
        val url = audioUrl ?: return null
        val resolvedGuid = guid ?: url

        return ParsedEpisode(
            guid = resolvedGuid,
            title = title,
            description = description,
            audioUrl = url,
            artworkUrl = artworkUrl,
            publishDate = publishDate,
            duration = duration,
            episodeNumber = episodeNumber,
        )
    }

    private fun parseImageUrl(reader: XmlReader): String? {
        var url: String? = null
        loop@ while (reader.hasNext()) {
            when (reader.next()) {
                EventType.START_ELEMENT -> if (reader.localName == "url") {
                    url = reader.readText()
                } else {
                    reader.skipElement()
                }
                EventType.END_ELEMENT -> if (reader.localName == "image") break@loop
                else -> Unit
            }
        }
        return url
    }

    private fun skipToElement(reader: XmlReader, localName: String) {
        while (reader.hasNext()) {
            if (reader.next() == EventType.START_ELEMENT && reader.localName == localName) return
        }
        throw FeedParseException("Expected <$localName> element not found")
    }
}

/** Reads all text content up to the matching END_ELEMENT, then advances past it. */
private fun XmlReader.readText(): String {
    val sb = StringBuilder()
    while (hasNext()) {
        when (next()) {
            EventType.TEXT, EventType.CDSECT -> sb.append(text)
            EventType.END_ELEMENT -> break
            else -> Unit
        }
    }
    return sb.toString().trim()
}

/** Skips the current element and all its children. */
private fun XmlReader.skipElement() {
    var depth = 1
    while (hasNext() && depth > 0) {
        when (next()) {
            EventType.START_ELEMENT -> depth++
            EventType.END_ELEMENT -> depth--
            else -> Unit
        }
    }
}
