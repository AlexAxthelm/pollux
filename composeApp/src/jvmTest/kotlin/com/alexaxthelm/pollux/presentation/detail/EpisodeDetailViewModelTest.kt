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

class EpisodeDetailViewModelTest {

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
        val vm = EpisodeDetailViewModel("ep1", EpisodeDetailFakeEpisodeRepo(), EpisodeDetailFakePodcastRepo())

        assertIs<EpisodeDetailState.Loading>(vm.state.value)
    }

    @Test
    fun should_EmitLoaded_When_EpisodeAndPodcastAvailable() = runTest(testDispatcher) {
        val podcast = testPodcast("pod1")
        val episode = testEpisode("ep1", "pod1")
        val episodeRepo = EpisodeDetailFakeEpisodeRepo().apply { addEpisode(episode) }
        val vm = EpisodeDetailViewModel("ep1", episodeRepo, EpisodeDetailFakePodcastRepo(listOf(podcast)))

        advanceUntilIdle()

        val state = vm.state.value
        assertIs<EpisodeDetailState.Loaded>(state)
        assertEquals(episode, state.episode)
        assertEquals(podcast, state.podcast)
    }

    @Test
    fun should_UpdateState_When_EpisodeMarkedPlayed() = runTest(testDispatcher) {
        val podcast = testPodcast("pod1")
        val episode = testEpisode("ep1", "pod1", isPlayed = false)
        val episodeRepo = EpisodeDetailFakeEpisodeRepo().apply { addEpisode(episode) }
        val vm = EpisodeDetailViewModel("ep1", episodeRepo, EpisodeDetailFakePodcastRepo(listOf(podcast)))
        advanceUntilIdle()

        vm.markEpisodePlayed(true)
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<EpisodeDetailState.Loaded>(state)
        assertEquals(true, state.episode.isPlayed)
    }

    @Test
    fun should_StayLoading_When_EpisodeNotFound() = runTest(testDispatcher) {
        val vm = EpisodeDetailViewModel(
            episodeId = "missing",
            episodeRepo = EpisodeDetailFakeEpisodeRepo(),
            podcastRepo = EpisodeDetailFakePodcastRepo(),
        )

        advanceUntilIdle()

        assertIs<EpisodeDetailState.Loading>(vm.state.value)
    }

    @Test
    fun should_StayLoading_When_PodcastNotFound() = runTest(testDispatcher) {
        val episode = testEpisode("ep1", "pod1")
        val episodeRepo = EpisodeDetailFakeEpisodeRepo().apply { addEpisode(episode) }
        val vm = EpisodeDetailViewModel(
            episodeId = "ep1",
            episodeRepo = episodeRepo,
            podcastRepo = EpisodeDetailFakePodcastRepo(emptyList()),
        )

        advanceUntilIdle()

        assertIs<EpisodeDetailState.Loading>(vm.state.value)
    }

    private fun testPodcast(id: String) = Podcast(
        id = id,
        feedUrl = "https://example.com/$id/feed.xml",
        title = "Test Podcast $id",
    )

    private fun testEpisode(id: String, podcastId: String, isPlayed: Boolean = false) = Episode(
        id = id,
        podcastId = podcastId,
        title = "Episode $id",
        audioUrl = "https://example.com/$id.mp3",
        publishDate = Instant.parse("2024-12-15T00:00:00Z"),
        duration = 1.hours + 23.minutes,
        isPlayed = isPlayed,
    )
}

// Named to avoid collision with the same-package fakes in PodcastDetailViewModelTest

private class EpisodeDetailFakePodcastRepo(
    private val podcasts: List<Podcast> = emptyList(),
) : PodcastRepository {
    override fun observeAllPodcasts(): Flow<List<Podcast>> = MutableStateFlow(podcasts)
    override suspend fun getPodcastById(id: String): Podcast? = podcasts.find { it.id == id }
    override suspend fun savePodcast(podcast: Podcast) = Unit
    override suspend fun savePodcasts(podcasts: List<Podcast>) = Unit
    override suspend fun deletePodcast(id: String) = Unit
    override suspend fun markUnsubscribed(id: String) = Unit
}

private class EpisodeDetailFakeEpisodeRepo : EpisodeRepository {
    private val episodeFlows = mutableMapOf<String, MutableStateFlow<Episode?>>()

    fun addEpisode(episode: Episode) {
        episodeFlows[episode.id] = MutableStateFlow(episode)
    }

    override fun observeEpisodeById(id: String): Flow<Episode?> =
        episodeFlows[id] ?: MutableStateFlow(null)

    override fun observeEpisodesByPodcast(podcastId: String): Flow<List<Episode>> =
        MutableStateFlow(episodeFlows.values.mapNotNull { it.value }.filter { it.podcastId == podcastId })

    override suspend fun getEpisodeById(id: String): Episode? = episodeFlows[id]?.value

    override suspend fun getEpisodesByPodcast(podcastId: String): List<Episode> =
        episodeFlows.values.mapNotNull { it.value }.filter { it.podcastId == podcastId }

    override suspend fun markEpisodePlayed(id: String, played: Boolean) {
        episodeFlows[id]?.let { flow -> flow.value = flow.value?.copy(isPlayed = played) }
    }

    override suspend fun saveEpisode(episode: Episode) = Unit
    override suspend fun saveEpisodes(episodes: List<Episode>) = Unit
    override suspend fun deleteEpisode(id: String) = Unit
    override suspend fun deleteEpisodesByPodcast(podcastId: String) = Unit
    override suspend fun updatePlayPosition(id: String, positionSeconds: Int) = Unit
    override suspend fun updateDownloadStatus(id: String, isDownloaded: Boolean, localPath: String?) = Unit
}
