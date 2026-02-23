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
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class SubscribeToPodcastUseCaseTest {

    private lateinit var useCase: SubscribeToPodcastUseCase
    private lateinit var podcastRepo: PodcastRepositoryImpl
    private lateinit var episodeRepo: EpisodeRepositoryImpl

    @BeforeTest
    fun setup() {
        val db = createTestDatabase()
        podcastRepo = PodcastRepositoryImpl(db)
        episodeRepo = EpisodeRepositoryImpl(db)
        useCase = SubscribeToPodcastUseCase(
            feedParser = FakeFeedParser(SAMPLE_FEED),
            podcastRepository = podcastRepo,
            episodeRepository = episodeRepo,
        )
    }

    @Test
    fun should_ReturnParsedFeed_When_PreviewCalled() = runTest {
        val feed = useCase.preview("https://example.com/feed.xml")

        assertEquals(SAMPLE_FEED.title, feed.title)
        assertEquals(2, feed.episodes.size)
    }

    @Test
    fun should_PropagateException_When_ParserFails() = runTest {
        val failingUseCase = SubscribeToPodcastUseCase(
            feedParser = FakeFeedParser(error = FeedParseException("Network error")),
            podcastRepository = podcastRepo,
            episodeRepository = episodeRepo,
        )

        assertFailsWith<FeedParseException> {
            failingUseCase.preview("https://bad.example.com/feed.xml")
        }
    }

    @Test
    fun should_SavePodcastAndEpisodes_When_SubscribeCalled() = runTest {
        useCase.subscribe(SAMPLE_FEED)

        val podcast = podcastRepo.getPodcastById(SAMPLE_FEED.feedUrl)
        assertNotNull(podcast)
        assertEquals(SAMPLE_FEED.title, podcast.title)
        assertEquals(SAMPLE_FEED.author, podcast.author)

        val episodes = episodeRepo.getEpisodesByPodcast(SAMPLE_FEED.feedUrl)
        assertEquals(2, episodes.size)
    }

    @Test
    fun should_UseFeedUrlAsPodcastId_When_Subscribing() = runTest {
        useCase.subscribe(SAMPLE_FEED)

        val podcast = podcastRepo.getPodcastById(SAMPLE_FEED.feedUrl)
        assertNotNull(podcast)
        assertEquals(SAMPLE_FEED.feedUrl, podcast.id)
        assertEquals(SAMPLE_FEED.feedUrl, podcast.feedUrl)
    }

    @Test
    fun should_UseGuidAsPart_OfEpisodeId() = runTest {
        useCase.subscribe(SAMPLE_FEED)

        val episodes = episodeRepo.getEpisodesByPodcast(SAMPLE_FEED.feedUrl)
        val ep = episodes.find { it.id.contains("ep-one-guid") }
        assertNotNull(ep)
    }

    @Test
    fun should_FallbackPublishDateToNow_When_ParsedDateIsNull() = runTest {
        val feedWithNullDate = SAMPLE_FEED.copy(
            episodes = listOf(
                SAMPLE_EPISODE_1.copy(publishDate = null),
            ),
        )
        useCase.subscribe(feedWithNullDate)

        val episodes = episodeRepo.getEpisodesByPodcast(SAMPLE_FEED.feedUrl)
        assertNotNull(episodes.first().publishDate)
    }

    @Test
    fun should_BeIdempotent_When_SubscribedTwice() = runTest {
        useCase.subscribe(SAMPLE_FEED)
        useCase.subscribe(SAMPLE_FEED)

        val episodes = episodeRepo.getEpisodesByPodcast(SAMPLE_FEED.feedUrl)
        // Upsert — should still be 2, not 4
        assertEquals(2, episodes.size)
    }

    @Test
    fun should_MarkPodcastSubscribed_When_Saved() = runTest {
        useCase.subscribe(SAMPLE_FEED)

        val podcast = podcastRepo.getPodcastById(SAMPLE_FEED.feedUrl)
        assertTrue(podcast!!.isSubscribed)
    }

    // --- Fixtures ---

    private val SAMPLE_EPISODE_1 = ParsedEpisode(
        guid = "ep-one-guid",
        title = "Episode One",
        description = "First episode",
        audioUrl = "https://example.com/ep1.mp3",
        artworkUrl = null,
        publishDate = Instant.parse("2023-01-01T00:00:00Z"),
        duration = 45.minutes,
        episodeNumber = 1,
    )

    private val SAMPLE_EPISODE_2 = ParsedEpisode(
        guid = "ep-two-guid",
        title = "Episode Two",
        description = null,
        audioUrl = "https://example.com/ep2.mp3",
        artworkUrl = "https://example.com/ep2-art.jpg",
        publishDate = null,
        duration = 30.minutes,
        episodeNumber = 2,
    )

    private val SAMPLE_FEED = ParsedFeed(
        feedUrl = "https://example.com/feed.xml",
        title = "Test Podcast",
        author = "Test Author",
        description = "A test podcast",
        artworkUrl = "https://example.com/artwork.jpg",
        episodes = listOf(SAMPLE_EPISODE_1, SAMPLE_EPISODE_2),
    )
}

private class FakeFeedParser(
    private val result: ParsedFeed? = null,
    private val error: Exception? = null,
) : FeedParser {
    override suspend fun parse(url: String): ParsedFeed {
        error?.let { throw it }
        return result ?: error("No result configured")
    }
}
