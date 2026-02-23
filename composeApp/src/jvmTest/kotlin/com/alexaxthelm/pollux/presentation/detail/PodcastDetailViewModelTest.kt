package com.alexaxthelm.pollux.presentation.detail

import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
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
import kotlin.time.Duration.Companion.hours
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Instant

class PodcastDetailViewModelTest {

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
        val vm = PodcastDetailViewModel("pod1", FakePodcastRepository(), FakeEpisodeRepository())

        assertIs<PodcastDetailState.Loading>(vm.state.value)
    }

    @Test
    fun should_EmitLoaded_When_PodcastAndEpisodesAvailable() = runTest(testDispatcher) {
        val podcast = testPodcast("pod1")
        val episodes = listOf(testEpisode("ep1", "pod1"), testEpisode("ep2", "pod1"))
        val episodesFlow = MutableStateFlow(episodes)
        val vm = PodcastDetailViewModel(
            podcastId = "pod1",
            podcastRepo = FakePodcastRepository(listOf(podcast)),
            episodeRepo = FakeEpisodeRepository(mapOf("pod1" to episodesFlow)),
        )

        advanceUntilIdle()

        val state = vm.state.value
        assertIs<PodcastDetailState.Loaded>(state)
        assertEquals(podcast, state.podcast)
        assertEquals(2, state.episodes.size)
    }

    @Test
    fun should_UpdateEpisodes_When_EpisodeListChanges() = runTest(testDispatcher) {
        val podcast = testPodcast("pod1")
        val episodesFlow = MutableStateFlow(listOf(testEpisode("ep1", "pod1")))
        val vm = PodcastDetailViewModel(
            podcastId = "pod1",
            podcastRepo = FakePodcastRepository(listOf(podcast)),
            episodeRepo = FakeEpisodeRepository(mapOf("pod1" to episodesFlow)),
        )
        advanceUntilIdle()

        episodesFlow.value = listOf(testEpisode("ep1", "pod1"), testEpisode("ep2", "pod1"))
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<PodcastDetailState.Loaded>(state)
        assertEquals(2, state.episodes.size)
    }

    @Test
    fun should_StayLoading_When_PodcastNotFound() = runTest(testDispatcher) {
        val vm = PodcastDetailViewModel(
            podcastId = "missing-id",
            podcastRepo = FakePodcastRepository(emptyList()),
            episodeRepo = FakeEpisodeRepository(),
        )

        advanceUntilIdle()

        assertIs<PodcastDetailState.Loading>(vm.state.value)
    }

    private fun testPodcast(id: String) = Podcast(
        id = id,
        feedUrl = "https://example.com/$id/feed.xml",
        title = "Test Podcast $id",
    )

    private fun testEpisode(id: String, podcastId: String) = Episode(
        id = id,
        podcastId = podcastId,
        title = "Episode $id",
        audioUrl = "https://example.com/$id.mp3",
        publishDate = Instant.parse("2024-12-15T00:00:00Z"),
        duration = 1.hours + 23.minutes,
    )
}

private class FakePodcastRepository(
    private val podcasts: List<Podcast> = emptyList(),
) : PodcastRepository {
    override fun observeAllPodcasts(): Flow<List<Podcast>> = MutableStateFlow(podcasts)
    override suspend fun getPodcastById(id: String): Podcast? = podcasts.find { it.id == id }
    override suspend fun savePodcast(podcast: Podcast) = Unit
    override suspend fun savePodcasts(podcasts: List<Podcast>) = Unit
    override suspend fun deletePodcast(id: String) = Unit
    override suspend fun markUnsubscribed(id: String) = Unit
}

private class FakeEpisodeRepository(
    private val episodeFlows: Map<String, MutableStateFlow<List<Episode>>> = emptyMap(),
) : EpisodeRepository {
    override fun observeEpisodesByPodcast(podcastId: String): Flow<List<Episode>> =
        episodeFlows[podcastId] ?: MutableStateFlow(emptyList())

    override fun observeEpisodeById(id: String): Flow<Episode?> =
        MutableStateFlow(episodeFlows.values.flatMap { it.value }.find { it.id == id })

    override suspend fun getEpisodesByPodcast(podcastId: String): List<Episode> =
        episodeFlows[podcastId]?.value ?: emptyList()

    override suspend fun getEpisodeById(id: String): Episode? =
        episodeFlows.values.flatMap { it.value }.find { it.id == id }
    override suspend fun saveEpisode(episode: Episode) = Unit
    override suspend fun saveEpisodes(episodes: List<Episode>) = Unit
    override suspend fun deleteEpisode(id: String) = Unit
    override suspend fun deleteEpisodesByPodcast(podcastId: String) = Unit
    override suspend fun markEpisodePlayed(id: String, played: Boolean) = Unit
    override suspend fun updatePlayPosition(id: String, positionSeconds: Int) = Unit
    override suspend fun updateDownloadStatus(id: String, isDownloaded: Boolean, localPath: String?) = Unit
}
