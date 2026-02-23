package com.alexaxthelm.pollux.presentation.subscribe

import com.alexaxthelm.pollux.domain.feed.FeedParseException
import com.alexaxthelm.pollux.domain.feed.FeedParser
import com.alexaxthelm.pollux.domain.feed.ParsedEpisode
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import com.alexaxthelm.pollux.domain.usecase.SubscribeToPodcastUseCase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class SubscribeViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setup() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun viewModelWith(
        parser: FeedParser,
        podcastRepo: PodcastRepository = FakePodcastRepository(),
        episodeRepo: EpisodeRepository = FakeEpisodeRepository(),
    ): SubscribeViewModel {
        val useCase = SubscribeToPodcastUseCase(
            feedParser = parser,
            podcastRepository = podcastRepo,
            episodeRepository = episodeRepo,
        )
        return SubscribeViewModel(useCase)
    }

    @Test
    fun should_StartIdle_When_Created() {
        val vm = viewModelWith(FakeFeedParser(SAMPLE_FEED))
        assertIs<SubscribeState.Idle>(vm.state.value)
    }

    @Test
    fun should_EmitLoadingImmediately_ThenPreview_When_SubmitSucceeds() = runTest(testDispatcher) {
        val vm = viewModelWith(FakeFeedParser(SAMPLE_FEED))

        vm.submit("https://example.com/feed.xml")
        // Loading is set synchronously before the coroutine launches
        assertEquals(SubscribeState.Loading, vm.state.value)

        advanceUntilIdle()
        assertIs<SubscribeState.Preview>(vm.state.value)
        assertEquals(SAMPLE_FEED.title, (vm.state.value as SubscribeState.Preview).feed.title)
    }

    @Test
    fun should_EmitError_When_SubmitFails() = runTest(testDispatcher) {
        val vm = viewModelWith(FakeFeedParser(error = FeedParseException("Not found")))

        vm.submit("https://bad.example.com/feed.xml")
        advanceUntilIdle()

        assertIs<SubscribeState.Error>(vm.state.value)
        assertEquals("Not found", (vm.state.value as SubscribeState.Error).message)
    }

    @Test
    fun should_EmitError_Synchronously_When_UrlIsBlank() {
        val vm = viewModelWith(FakeFeedParser(SAMPLE_FEED))

        vm.submit("   ")

        assertIs<SubscribeState.Error>(vm.state.value)
    }

    @Test
    fun should_ReturnToIdle_When_DismissErrorCalled() = runTest(testDispatcher) {
        val vm = viewModelWith(FakeFeedParser(error = FeedParseException("boom")))
        vm.submit("https://example.com/feed.xml")
        advanceUntilIdle()

        vm.dismissError()

        assertIs<SubscribeState.Idle>(vm.state.value)
    }

    @Test
    fun should_ReturnToIdle_When_CancelPreviewCalled() = runTest(testDispatcher) {
        val vm = viewModelWith(FakeFeedParser(SAMPLE_FEED))
        vm.submit("https://example.com/feed.xml")
        advanceUntilIdle()

        vm.cancelPreview()

        assertIs<SubscribeState.Idle>(vm.state.value)
    }

    @Test
    fun should_EmitLoadingThenSaved_When_ConfirmSucceeds() = runTest(testDispatcher) {
        val vm = viewModelWith(FakeFeedParser(SAMPLE_FEED))
        vm.submit("https://example.com/feed.xml")
        advanceUntilIdle()

        vm.confirmSubscription()
        assertEquals(SubscribeState.Loading, vm.state.value)

        advanceUntilIdle()
        assertIs<SubscribeState.Saved>(vm.state.value)
    }

    @Test
    fun should_EmitError_When_ConfirmFails() = runTest(testDispatcher) {
        val failingRepo = object : FakePodcastRepository() {
            override suspend fun savePodcast(podcast: Podcast) = throw Exception("DB error")
        }
        val vm = viewModelWith(FakeFeedParser(SAMPLE_FEED), podcastRepo = failingRepo)
        vm.submit("https://example.com/feed.xml")
        advanceUntilIdle()

        vm.confirmSubscription()
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<SubscribeState.Error>(state)
        assertEquals("DB error", state.message)
    }

    @Test
    fun should_DoNothing_When_ConfirmCalledWithoutPreview() = runTest(testDispatcher) {
        val vm = viewModelWith(FakeFeedParser(SAMPLE_FEED))

        vm.confirmSubscription()
        advanceUntilIdle()

        assertIs<SubscribeState.Idle>(vm.state.value)
    }

    // --- Fixtures ---

    private val SAMPLE_FEED = ParsedFeed(
        feedUrl = "https://example.com/feed.xml",
        title = "Test Podcast",
        author = "Test Author",
        description = "A test podcast",
        artworkUrl = null,
        episodes = listOf(
            ParsedEpisode(
                guid = "ep1",
                title = "Episode One",
                description = null,
                audioUrl = "https://example.com/ep1.mp3",
                artworkUrl = null,
                publishDate = Instant.parse("2023-01-01T00:00:00Z"),
                duration = 30.minutes,
                episodeNumber = 1,
            ),
        ),
    )
}

// --- Fakes ---

private class FakeFeedParser(
    private val result: ParsedFeed? = null,
    private val error: Exception? = null,
) : FeedParser {
    override suspend fun parse(url: String): ParsedFeed {
        error?.let { throw it }
        return result ?: error("No result configured")
    }
}

private open class FakePodcastRepository : PodcastRepository {
    private val store = mutableMapOf<String, Podcast>()
    private val flow = MutableStateFlow<List<Podcast>>(emptyList())

    override fun observeAllPodcasts(): Flow<List<Podcast>> = flow
    override suspend fun getPodcastById(id: String): Podcast? = store[id]
    override suspend fun savePodcast(podcast: Podcast) {
        store[podcast.id] = podcast
        flow.value = store.values.toList()
    }
    override suspend fun savePodcasts(podcasts: List<Podcast>) = podcasts.forEach { savePodcast(it) }
    override suspend fun deletePodcast(id: String) {
        store.remove(id)
        flow.value = store.values.toList()
    }
    override suspend fun markUnsubscribed(id: String) {
        store[id]?.let { savePodcast(it.copy(isSubscribed = false)) }
    }
}

private class FakeEpisodeRepository : EpisodeRepository {
    private val store = mutableMapOf<String, Episode>()
    private val flow = MutableStateFlow<List<Episode>>(emptyList())

    override fun observeEpisodesByPodcast(podcastId: String): Flow<List<Episode>> = flow
    override fun observeEpisodeById(id: String): Flow<Episode?> = MutableStateFlow(store[id])
    override suspend fun getEpisodesByPodcast(podcastId: String): List<Episode> =
        store.values.filter { it.podcastId == podcastId }
    override suspend fun getEpisodeById(id: String): Episode? = store[id]
    override suspend fun saveEpisode(episode: Episode) { store[episode.id] = episode }
    override suspend fun saveEpisodes(episodes: List<Episode>) = episodes.forEach { saveEpisode(it) }
    override suspend fun deleteEpisode(id: String) { store.remove(id) }
    override suspend fun deleteEpisodesByPodcast(podcastId: String) {
        store.entries.removeAll { it.value.podcastId == podcastId }
    }
    override suspend fun markEpisodePlayed(id: String, played: Boolean) {
        store[id]?.let { store[id] = it.copy(isPlayed = played) }
    }
    override suspend fun updatePlayPosition(id: String, positionSeconds: Int) {
        store[id]?.let { store[id] = it.copy(playPositionSeconds = positionSeconds) }
    }
    override suspend fun updateDownloadStatus(id: String, isDownloaded: Boolean, localPath: String?) {
        store[id]?.let { store[id] = it.copy(isDownloaded = isDownloaded, localPath = localPath) }
    }
}
