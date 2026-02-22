package com.alexaxthelm.pollux.data.feed

import com.alexaxthelm.pollux.domain.feed.FeedParseException
import com.alexaxthelm.pollux.domain.feed.ParsedEpisode
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import nl.adaptivity.xmlutil.EventType
import nl.adaptivity.xmlutil.XmlReader
import kotlin.time.Duration

private const val ITUNES_NS = "http://www.itunes.com/dtds/podcast-1.0.dtd"
private const val ATOM_NS = "http://www.w3.org/2005/Atom"

internal object AtomFeedParser {
    fun parse(feedUrl: String, reader: XmlReader): ParsedFeed {
        var title = ""
        var author: String? = null
        var description: String? = null
        var artworkUrl: String? = null
        val episodes = LinkedHashMap<String, ParsedEpisode>()

        loop@ while (reader.hasNext()) {
            when (reader.next()) {
                EventType.START_ELEMENT -> when {
                    reader.localName == "title" && isAtomOrEmpty(reader.namespaceURI) -> {
                        title = reader.readText()
                    }
                    reader.localName == "subtitle" && isAtomOrEmpty(reader.namespaceURI) -> {
                        if (description == null) description = reader.readText()
                    }
                    reader.localName == "summary" && isAtomOrEmpty(reader.namespaceURI) -> {
                        if (description == null) description = reader.readText()
                    }
                    reader.localName == "author" && isAtomOrEmpty(reader.namespaceURI) -> {
                        author = parseFeedAuthor(reader)
                    }
                    reader.localName == "image" && reader.namespaceURI == ITUNES_NS -> {
                        val href = reader.getAttributeValue(null, "href")
                        if (artworkUrl == null && href != null) artworkUrl = href
                        reader.skipElement()
                    }
                    reader.localName == "logo" && isAtomOrEmpty(reader.namespaceURI) -> {
                        if (artworkUrl == null) artworkUrl = reader.readText()
                    }
                    reader.localName == "entry" -> {
                        val episode = parseEntry(reader)
                        if (episode != null && !episodes.containsKey(episode.guid)) {
                            episodes[episode.guid] = episode
                        }
                    }
                    else -> reader.skipElement()
                }
                EventType.END_ELEMENT -> if (reader.localName == "feed") break@loop
                else -> Unit
            }
        }

        if (title.isEmpty()) throw FeedParseException("Atom feed missing <title>")
        return ParsedFeed(
            feedUrl = feedUrl,
            title = title,
            author = author,
            description = description,
            artworkUrl = artworkUrl,
            episodes = episodes.values.toList(),
        )
    }

    private fun parseFeedAuthor(reader: XmlReader): String? {
        var name: String? = null
        loop@ while (reader.hasNext()) {
            when (reader.next()) {
                EventType.START_ELEMENT -> if (reader.localName == "name") {
                    name = reader.readText()
                } else {
                    reader.skipElement()
                }
                EventType.END_ELEMENT -> if (reader.localName == "author") break@loop
                else -> Unit
            }
        }
        return name
    }

    private fun parseEntry(reader: XmlReader): ParsedEpisode? {
        var title = ""
        var description: String? = null
        var id: String? = null
        var audioUrl: String? = null
        var artworkUrl: String? = null
        var publishDate = null as kotlin.time.Instant?
        var duration = Duration.ZERO
        var episodeNumber: Int? = null

        loop@ while (reader.hasNext()) {
            when (reader.next()) {
                EventType.START_ELEMENT -> when {
                    reader.localName == "title" && isAtomOrEmpty(reader.namespaceURI) -> {
                        title = reader.readText()
                    }
                    reader.localName == "summary" && isAtomOrEmpty(reader.namespaceURI) -> {
                        description = reader.readText()
                    }
                    reader.localName == "id" && isAtomOrEmpty(reader.namespaceURI) -> {
                        id = reader.readText()
                    }
                    reader.localName == "link" && isAtomOrEmpty(reader.namespaceURI) -> {
                        val rel = reader.getAttributeValue(null, "rel")
                        val href = reader.getAttributeValue(null, "href")
                        if (rel == "enclosure" && href != null) {
                            audioUrl = href
                        }
                        reader.skipElement()
                    }
                    reader.localName == "image" && reader.namespaceURI == ITUNES_NS -> {
                        val href = reader.getAttributeValue(null, "href")
                        if (href != null) artworkUrl = href
                        reader.skipElement()
                    }
                    reader.localName == "published" && isAtomOrEmpty(reader.namespaceURI) -> {
                        publishDate = DateParser.parseIso8601(reader.readText())
                    }
                    reader.localName == "duration" && reader.namespaceURI == ITUNES_NS -> {
                        duration = DurationParser.parse(reader.readText())
                    }
                    reader.localName == "episode" && reader.namespaceURI == ITUNES_NS -> {
                        episodeNumber = reader.readText().toIntOrNull()
                    }
                    else -> reader.skipElement()
                }
                EventType.END_ELEMENT -> if (reader.localName == "entry") break@loop
                else -> Unit
            }
        }

        val url = audioUrl ?: return null
        val resolvedGuid = id ?: url

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

    private fun isAtomOrEmpty(ns: String) = ns.isEmpty() || ns == ATOM_NS
}

/** Reads all text content up to the matching END_ELEMENT. */
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
