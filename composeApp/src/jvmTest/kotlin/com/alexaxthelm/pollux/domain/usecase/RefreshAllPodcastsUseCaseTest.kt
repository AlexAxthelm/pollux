package com.alexaxthelm.pollux.domain.usecase

import com.alexaxthelm.pollux.data.repository.EpisodeRepositoryImpl
import com.alexaxthelm.pollux.data.repository.PodcastRepositoryImpl
import com.alexaxthelm.pollux.data.repository.createTestDatabase
import com.alexaxthelm.pollux.domain.feed.FeedParseException
import com.alexaxthelm.pollux.domain.feed.FeedParser
import com.alexaxthelm.pollux.domain.feed.ParsedEpisode
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import kotlinx.coroutines.test.runTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class RefreshAllPodcastsUseCaseTest {

    private lateinit var podcastRepo: PodcastRepositoryImpl
    private lateinit var episodeRepo: EpisodeRepositoryImpl
    private lateinit var subscribeUseCase: SubscribeToPodcastUseCase

    @BeforeTest
    fun setup() {
        val db = createTestDatabase()
        podcastRepo = PodcastRepositoryImpl(db)
        episodeRepo = EpisodeRepositoryImpl(db)
        subscribeUseCase = SubscribeToPodcastUseCase(
            feedParser = FixedFeedParser(mapOf(FEED_URL to INITIAL_FEED)),
            podcastRepository = podcastRepo,
            episodeRepository = episodeRepo,
        )
    }

    @Test
    fun should_InsertNewEpisodes_When_FeedHasNewContent() = runTest {
        subscribeUseCase.subscribe(INITIAL_FEED)

        val updatedFeed = INITIAL_FEED.copy(episodes = INITIAL_FEED.episodes + NEW_EPISODE)
        val useCase = buildUseCase(mapOf(FEED_URL to updatedFeed))

        useCase.refresh()

        val episodes = episodeRepo.getEpisodesByPodcast(FEED_URL)
        assertEquals(3, episodes.size)
    }

    @Test
    fun should_NotOverwriteExistingEpisodes_When_Refreshing() = runTest {
        subscribeUseCase.subscribe(INITIAL_FEED)
        episodeRepo.markEpisodePlayed("$FEED_URL:ep-one-guid", played = true)

        val useCase = buildUseCase(mapOf(FEED_URL to INITIAL_FEED))
        useCase.refresh()

        val episode = episodeRepo.getEpisodeById("$FEED_URL:ep-one-guid")
        assertTrue(episode!!.isPlayed, "isPlayed should be preserved after refresh")
    }

    @Test
    fun should_SkipFailingFeed_When_ParseErrorOccurs() = runTest {
        subscribeUseCase.subscribe(INITIAL_FEED)

        // Parser always throws — refresh should complete without crashing
        val useCase = RefreshAllPodcastsUseCase(
            feedParser = FailingFeedParser(),
            podcastRepository = podcastRepo,
            episodeRepository = episodeRepo,
        )

        useCase.refresh() // Should not throw

        // Existing episodes unchanged
        val episodes = episodeRepo.getEpisodesByPodcast(FEED_URL)
        assertEquals(2, episodes.size)
    }

    @Test
    fun should_ContinueRefreshingOtherFeeds_When_OneFails() = runTest {
        val feedB = INITIAL_FEED.copy(
            feedUrl = "https://example.com/feedB.xml",
            title = "Podcast B",
            episodes = listOf(NEW_EPISODE.copy(guid = "b-ep-guid")),
        )
        subscribeUseCase.subscribe(INITIAL_FEED)
        subscribeUseCase.subscribe(feedB)

        // Feed A fails, feed B succeeds with a new episode
        val extraEpisode = NEW_EPISODE.copy(guid = "b-ep-new")
        val updatedFeedB = feedB.copy(episodes = feedB.episodes + extraEpisode)
        val useCase = RefreshAllPodcastsUseCase(
            feedParser = FixedFeedParser(
                results = mapOf(feedB.feedUrl to updatedFeedB),
                defaultError = FeedParseException("Network error"),
            ),
            podcastRepository = podcastRepo,
            episodeRepository = episodeRepo,
        )

        useCase.refresh()

        val episodesB = episodeRepo.getEpisodesByPodcast(feedB.feedUrl)
        assertEquals(2, episodesB.size, "Feed B should have received its new episode")
    }

    @Test
    fun should_DoNothing_When_NoSubscribedPodcasts() = runTest {
        val useCase = buildUseCase(emptyMap())
        useCase.refresh() // Should complete without error
    }

    // --- Helpers ---

    private fun buildUseCase(feeds: Map<String, ParsedFeed>) = RefreshAllPodcastsUseCase(
        feedParser = FixedFeedParser(feeds),
        podcastRepository = podcastRepo,
        episodeRepository = episodeRepo,
    )

    // --- Fixtures ---

    companion object {
        const val FEED_URL = "https://example.com/feed.xml"

        val NEW_EPISODE = ParsedEpisode(
            guid = "ep-three-guid",
            title = "Episode Three",
            description = null,
            audioUrl = "https://example.com/ep3.mp3",
            artworkUrl = null,
            publishDate = Instant.parse("2023-03-01T00:00:00Z"),
            duration = 20.minutes,
            episodeNumber = 3,
        )

        val INITIAL_FEED = ParsedFeed(
            feedUrl = FEED_URL,
            title = "Test Podcast",
            author = "Test Author",
            description = "A test podcast",
            artworkUrl = null,
            episodes = listOf(
                ParsedEpisode(
                    guid = "ep-one-guid",
                    title = "Episode One",
                    description = null,
                    audioUrl = "https://example.com/ep1.mp3",
                    artworkUrl = null,
                    publishDate = Instant.parse("2023-01-01T00:00:00Z"),
                    duration = 45.minutes,
                    episodeNumber = 1,
                ),
                ParsedEpisode(
                    guid = "ep-two-guid",
                    title = "Episode Two",
                    description = null,
                    audioUrl = "https://example.com/ep2.mp3",
                    artworkUrl = null,
                    publishDate = Instant.parse("2023-02-01T00:00:00Z"),
                    duration = 30.minutes,
                    episodeNumber = 2,
                ),
            ),
        )
    }
}

private class FixedFeedParser(
    private val results: Map<String, ParsedFeed> = emptyMap(),
    private val defaultError: Exception? = null,
) : FeedParser {
    override suspend fun parse(url: String): ParsedFeed =
        results[url] ?: defaultError?.let { throw it } ?: error("No result for $url")
}

private class FailingFeedParser : FeedParser {
    override suspend fun parse(url: String): ParsedFeed =
        throw FeedParseException("Network error")
}
