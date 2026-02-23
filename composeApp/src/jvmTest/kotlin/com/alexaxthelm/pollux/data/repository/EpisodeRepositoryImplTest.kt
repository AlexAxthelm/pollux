package com.alexaxthelm.pollux.data.repository

import com.alexaxthelm.pollux.data.database.PolluxDatabase
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.time.Instant
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.minutes

class EpisodeRepositoryImplTest {

    private lateinit var database: PolluxDatabase
    private lateinit var podcastRepo: PodcastRepositoryImpl
    private lateinit var episodeRepo: EpisodeRepositoryImpl

    @BeforeTest
    fun setup() {
        database = createTestDatabase()
        podcastRepo = PodcastRepositoryImpl(database)
        episodeRepo = EpisodeRepositoryImpl(database)
    }

    private suspend fun insertTestPodcast(id: String = "p1") {
        podcastRepo.savePodcast(
            Podcast(
                id = id,
                feedUrl = "https://example.com/$id/feed.xml",
                title = "Podcast $id",
            )
        )
    }

    private fun testEpisode(
        id: String = "e1",
        podcastId: String = "p1",
        title: String = "Episode 1",
        publishMillis: Long = 1700000000000,
    ) = Episode(
        id = id,
        podcastId = podcastId,
        title = title,
        description = "Description",
        audioUrl = "https://example.com/$id.mp3",
        artworkUrl = "https://example.com/$id.png",
        publishDate = Instant.fromEpochMilliseconds(publishMillis),
        duration = 30.minutes,
        episodeNumber = 1,
        isPlayed = false,
        playPositionSeconds = 0,
        isDownloaded = false,
        localPath = null,
    )

    @Test
    fun should_SaveAndRetrieve_When_EpisodeIsSaved() = runTest {
        insertTestPodcast()
        val episode = testEpisode()

        episodeRepo.saveEpisode(episode)
        val retrieved = episodeRepo.getEpisodeById("e1")

        assertEquals(episode, retrieved)
    }

    @Test
    fun should_ReturnNull_When_EpisodeNotFound() = runTest {
        assertNull(episodeRepo.getEpisodeById("nonexistent"))
    }

    @Test
    fun should_FilterByPodcast_When_QueryingByPodcastId() = runTest {
        insertTestPodcast("p1")
        insertTestPodcast("p2")
        episodeRepo.saveEpisode(testEpisode("e1", "p1"))
        episodeRepo.saveEpisode(testEpisode("e2", "p1"))
        episodeRepo.saveEpisode(testEpisode("e3", "p2"))

        val p1Episodes = episodeRepo.getEpisodesByPodcast("p1")
        val p2Episodes = episodeRepo.getEpisodesByPodcast("p2")

        assertEquals(2, p1Episodes.size)
        assertEquals(1, p2Episodes.size)
    }

    @Test
    fun should_DeleteEpisodes_When_PodcastIsDeleted() = runTest {
        insertTestPodcast()
        episodeRepo.saveEpisode(testEpisode("e1"))
        episodeRepo.saveEpisode(testEpisode("e2"))

        podcastRepo.deletePodcast("p1")

        // ON DELETE CASCADE should remove episodes
        assertTrue(episodeRepo.getEpisodesByPodcast("p1").isEmpty())
    }

    @Test
    fun should_MarkPlayed_When_MarkEpisodePlayedCalled() = runTest {
        insertTestPodcast()
        episodeRepo.saveEpisode(testEpisode())

        episodeRepo.markEpisodePlayed("e1", true)

        assertEquals(true, episodeRepo.getEpisodeById("e1")?.isPlayed)
    }

    @Test
    fun should_UpdatePosition_When_UpdatePlayPositionCalled() = runTest {
        insertTestPodcast()
        episodeRepo.saveEpisode(testEpisode())

        episodeRepo.updatePlayPosition("e1", 300)

        assertEquals(300, episodeRepo.getEpisodeById("e1")?.playPositionSeconds)
    }

    @Test
    fun should_UpdateDownloadStatus_When_Called() = runTest {
        insertTestPodcast()
        episodeRepo.saveEpisode(testEpisode())

        episodeRepo.updateDownloadStatus("e1", true, "/downloads/ep1.mp3")

        val updated = episodeRepo.getEpisodeById("e1")
        assertEquals(true, updated?.isDownloaded)
        assertEquals("/downloads/ep1.mp3", updated?.localPath)
    }

    @Test
    fun should_EmitUpdatedList_When_ObservingEpisodes() = runTest {
        insertTestPodcast()
        episodeRepo.saveEpisode(testEpisode("e1", publishMillis = 1700000002000))
        episodeRepo.saveEpisode(testEpisode("e2", publishMillis = 1700000001000))

        val episodes = episodeRepo.observeEpisodesByPodcast("p1").first()

        assertEquals(2, episodes.size)
        // Ordered by publishDate DESC
        assertEquals("e1", episodes[0].id)
        assertEquals("e2", episodes[1].id)
    }

    @Test
    fun should_DeleteByPodcast_When_DeleteEpisodesByPodcastCalled() = runTest {
        insertTestPodcast("p1")
        insertTestPodcast("p2")
        episodeRepo.saveEpisode(testEpisode("e1", "p1"))
        episodeRepo.saveEpisode(testEpisode("e2", "p2"))

        episodeRepo.deleteEpisodesByPodcast("p1")

        assertTrue(episodeRepo.getEpisodesByPodcast("p1").isEmpty())
        assertEquals(1, episodeRepo.getEpisodesByPodcast("p2").size)
    }

    @Test
    fun should_EmitEpisode_When_ObservingById() = runTest {
        insertTestPodcast()
        episodeRepo.saveEpisode(testEpisode())

        val episode = episodeRepo.observeEpisodeById("e1").first()

        assertEquals("e1", episode?.id)
    }

    @Test
    fun should_EmitUpdatedEpisode_When_PlayedStatusChanges() = runTest {
        insertTestPodcast()
        episodeRepo.saveEpisode(testEpisode())

        episodeRepo.markEpisodePlayed("e1", true)
        val episode = episodeRepo.observeEpisodeById("e1").first()

        assertEquals(true, episode?.isPlayed)
    }

    @Test
    fun should_SaveAll_When_BatchSaving() = runTest {
        insertTestPodcast()
        val episodes = listOf(
            testEpisode("e1"),
            testEpisode("e2"),
            testEpisode("e3"),
        )

        episodeRepo.saveEpisodes(episodes)

        assertEquals(3, episodeRepo.getEpisodesByPodcast("p1").size)
    }
}
