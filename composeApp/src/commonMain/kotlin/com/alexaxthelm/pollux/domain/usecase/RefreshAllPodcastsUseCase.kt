package com.alexaxthelm.pollux.domain.usecase

import com.alexaxthelm.pollux.domain.feed.FeedParser
import com.alexaxthelm.pollux.domain.feed.ParsedEpisode
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import kotlin.time.Clock
import kotlin.time.Instant
import kotlinx.coroutines.flow.first

open class RefreshAllPodcastsUseCase(
    private val feedParser: FeedParser,
    private val podcastRepository: PodcastRepository,
    private val episodeRepository: EpisodeRepository,
    private val clock: Clock = Clock.System,
) {
    /**
     * Re-fetches all subscribed podcast feeds and inserts any new episodes.
     * Existing episodes are intentionally skipped to preserve user state
     * (isPlayed, playPosition, download status).
     *
     * Failures on individual feeds are silently skipped so one bad feed does not
     * block refreshing the rest of the library.
     */
    open suspend fun refresh() {
        val podcasts = podcastRepository.observeAllPodcasts().first()

        podcasts.forEach { existingPodcast ->
            try {
                val parsed = feedParser.parse(existingPodcast.feedUrl)

                val existingIds = episodeRepository
                    .getEpisodesByPodcast(existingPodcast.id)
                    .map { it.id }
                    .toSet()

                val newEpisodes = parsed.episodes
                    .map { it.toEpisode(existingPodcast.id, clock.now()) }
                    .filter { it.id !in existingIds }

                if (newEpisodes.isNotEmpty()) {
                    episodeRepository.saveEpisodes(newEpisodes)
                }
            } catch (_: Exception) {
                // Skip this feed — a parse failure should not abort the full refresh
            }
        }
    }

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
