package com.alexaxthelm.pollux.presentation.library

import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
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

    @Test
    fun should_EmitLoading_Initially_BeforeFirstEmission() {
        val repo = FakePodcastRepository()
        val vm = LibraryViewModel(repo)

        assertIs<LibraryState.Loading>(vm.state.value)
    }

    @Test
    fun should_EmitEmptyList_When_NoPodcastsSubscribed() = runTest(testDispatcher) {
        val repo = FakePodcastRepository()
        val vm = LibraryViewModel(repo)

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
        val vm = LibraryViewModel(repo)

        advanceUntilIdle()

        val state = vm.state.value
        assertIs<LibraryState.Loaded>(state)
        assertEquals(2, state.podcasts.size)
        assertEquals("Podcast One", state.podcasts[0].title)
    }

    @Test
    fun should_UpdateState_When_NewPodcastAdded() = runTest(testDispatcher) {
        val repo = FakePodcastRepository()
        val vm = LibraryViewModel(repo)
        advanceUntilIdle()

        repo.emit(listOf(testPodcast("p1", "New Podcast")))
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<LibraryState.Loaded>(state)
        assertEquals(1, state.podcasts.size)
        assertEquals("New Podcast", state.podcasts[0].title)
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
