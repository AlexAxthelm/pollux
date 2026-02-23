package com.alexaxthelm.pollux.presentation.library

import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import com.alexaxthelm.pollux.domain.feed.FeedParser
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import com.alexaxthelm.pollux.domain.usecase.RefreshAllPodcastsUseCase
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue
import kotlin.time.Instant

class LibraryViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setup() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private val fakeRefreshUseCase = FakeRefreshAllPodcastsUseCase()

    @Test
    fun should_EmitLoading_Initially_BeforeFirstEmission() {
        val repo = FakePodcastRepository()
        val vm = LibraryViewModel(repo, fakeRefreshUseCase)

        assertIs<LibraryState.Loading>(vm.state.value)
    }

    @Test
    fun should_EmitEmptyList_When_NoPodcastsSubscribed() = runTest(testDispatcher) {
        val repo = FakePodcastRepository()
        val vm = LibraryViewModel(repo, fakeRefreshUseCase)

        advanceUntilIdle()

        val state = vm.state.value
        assertIs<LibraryState.Loaded>(state)
        assertEquals(0, state.podcasts.size)
    }

    @Test
    fun should_EmitPodcasts_When_RepoHasSubscriptions() = runTest(testDispatcher) {
        val repo = FakePodcastRepository(
            initial = listOf(testPodcast("p1", "Podcast One"), testPodcast("p2", "Podcast Two")),
        )
        val vm = LibraryViewModel(repo, fakeRefreshUseCase)

        advanceUntilIdle()

        val state = vm.state.value
        assertIs<LibraryState.Loaded>(state)
        assertEquals(2, state.podcasts.size)
        assertEquals("Podcast One", state.podcasts[0].title)
    }

    @Test
    fun should_UpdateState_When_NewPodcastAdded() = runTest(testDispatcher) {
        val repo = FakePodcastRepository()
        val vm = LibraryViewModel(repo, fakeRefreshUseCase)
        advanceUntilIdle()

        repo.emit(listOf(testPodcast("p1", "New Podcast")))
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<LibraryState.Loaded>(state)
        assertEquals(1, state.podcasts.size)
        assertEquals("New Podcast", state.podcasts[0].title)
    }

    @Test
    fun should_SetIsRefreshing_While_RefreshInProgress() = runTest(testDispatcher) {
        val repo = FakePodcastRepository()
        val useCase = FakeRefreshAllPodcastsUseCase(delayMs = 1000)
        val vm = LibraryViewModel(repo, useCase)
        advanceUntilIdle()

        vm.refreshAll()
        // Run until the coroutine suspends inside the use case delay
        runCurrent()

        val stateAfterLaunch = vm.state.value
        assertIs<LibraryState.Loaded>(stateAfterLaunch)
        assertTrue(stateAfterLaunch.isRefreshing)

        advanceUntilIdle()

        // Once the use case finishes, isRefreshing should clear
        val stateAfterRefresh = vm.state.value
        assertIs<LibraryState.Loaded>(stateAfterRefresh)
        assertFalse(stateAfterRefresh.isRefreshing)
    }

    // --- Fixtures ---

    private fun testPodcast(id: String, title: String) = Podcast(
        id = id,
        feedUrl = "https://example.com/$id/feed.xml",
        title = title,
        lastRefreshed = Instant.parse("2023-01-01T00:00:00Z"),
    )
}

private class FakePodcastRepository(
    initial: List<Podcast> = emptyList(),
) : PodcastRepository {
    private val flow = MutableStateFlow(initial)

    fun emit(podcasts: List<Podcast>) {
        flow.value = podcasts
    }

    override fun observeAllPodcasts(): Flow<List<Podcast>> = flow
    override suspend fun getPodcastById(id: String): Podcast? = flow.value.find { it.id == id }
    override suspend fun savePodcast(podcast: Podcast) { flow.value = flow.value + podcast }
    override suspend fun savePodcasts(podcasts: List<Podcast>) { flow.value = flow.value + podcasts }
    override suspend fun deletePodcast(id: String) { flow.value = flow.value.filter { it.id != id } }
    override suspend fun markUnsubscribed(id: String) {
        flow.value = flow.value.map { if (it.id == id) it.copy(isSubscribed = false) else it }
    }
}

private class StubEpisodeRepository : EpisodeRepository {
    override fun observeEpisodesByPodcast(podcastId: String) = kotlinx.coroutines.flow.flowOf(emptyList<Episode>())
    override fun observeEpisodeById(id: String) = kotlinx.coroutines.flow.flowOf(null)
    override suspend fun getEpisodesByPodcast(podcastId: String) = emptyList<Episode>()
    override suspend fun getEpisodeById(id: String): Episode? = null
    override suspend fun saveEpisode(episode: Episode) = Unit
    override suspend fun saveEpisodes(episodes: List<Episode>) = Unit
    override suspend fun deleteEpisode(id: String) = Unit
    override suspend fun deleteEpisodesByPodcast(podcastId: String) = Unit
    override suspend fun markEpisodePlayed(id: String, played: Boolean) = Unit
    override suspend fun updatePlayPosition(id: String, positionSeconds: Int) = Unit
    override suspend fun updateDownloadStatus(id: String, isDownloaded: Boolean, localPath: String?) = Unit
}

private class FakeRefreshAllPodcastsUseCase(
    private val delayMs: Long = 0,
) : RefreshAllPodcastsUseCase(
    feedParser = object : FeedParser {
        override suspend fun parse(url: String): ParsedFeed = error("not used in fake")
    },
    podcastRepository = FakePodcastRepository(),
    episodeRepository = StubEpisodeRepository(),
) {
    override suspend fun refresh() {
        if (delayMs > 0) delay(delayMs)
    }
}
