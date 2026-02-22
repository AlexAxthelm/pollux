package com.alexaxthelm.pollux.domain.usecase

import com.alexaxthelm.pollux.domain.feed.FeedParser
import com.alexaxthelm.pollux.domain.feed.ParsedEpisode
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import kotlin.time.Clock
import kotlin.time.Instant

class SubscribeToPodcastUseCase(
    private val feedParser: FeedParser,
    private val podcastRepository: PodcastRepository,
    private val episodeRepository: EpisodeRepository,
    private val clock: Clock = Clock.System,
) {
    /**
     * Fetches and parses [url] without persisting anything.
     * The caller can show a preview to the user before committing.
     * Throws [com.alexaxthelm.pollux.domain.feed.FeedParseException] on failure.
     */
    suspend fun preview(url: String): ParsedFeed = feedParser.parse(url)

    /**
     * Persists [parsedFeed] as a [Podcast] and its episodes to the database.
     * Uses [feedUrl] as the stable podcast ID so re-subscribing is idempotent.
     */
    suspend fun subscribe(parsedFeed: ParsedFeed) {
        val podcast = parsedFeed.toPodcast(clock.now())
        podcastRepository.savePodcast(podcast)
        val episodes = parsedFeed.episodes.map { it.toEpisode(podcast.id, clock.now()) }
        episodeRepository.saveEpisodes(episodes)
    }

    private fun ParsedFeed.toPodcast(now: Instant): Podcast = Podcast(
        id = feedUrl,
        feedUrl = feedUrl,
        title = title,
        author = author,
        description = description,
        artworkUrl = artworkUrl,
        lastRefreshed = now,
        isSubscribed = true,
    )

    private fun ParsedEpisode.toEpisode(podcastId: String, now: Instant): Episode = Episode(
        id = "$podcastId:$guid",
        podcastId = podcastId,
        title = title,
        description = description,
        audioUrl = audioUrl,
        artworkUrl = artworkUrl,
        publishDate = publishDate ?: now,
        duration = duration,
        episodeNumber = episodeNumber,
    )
}
