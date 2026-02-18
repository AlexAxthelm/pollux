package com.alexaxthelm.pollux.data.repository

import com.alexaxthelm.pollux.domain.model.Podcast
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.time.Instant
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class PodcastRepositoryImplTest {

    private lateinit var repository: PodcastRepositoryImpl

    @BeforeTest
    fun setup() {
        repository = PodcastRepositoryImpl(createTestDatabase())
    }

    private fun testPodcast(
        id: String = "p1",
        title: String = "Test Podcast",
    ) = Podcast(
        id = id,
        feedUrl = "https://example.com/$id/feed.xml",
        title = title,
        author = "Author",
        description = "Description",
        artworkUrl = "https://example.com/$id/art.png",
        lastRefreshed = Instant.fromEpochMilliseconds(1700000000000),
        isSubscribed = true,
    )

    @Test
    fun should_SaveAndRetrieve_When_PodcastIsSaved() = runTest {
        val podcast = testPodcast()

        repository.savePodcast(podcast)
        val retrieved = repository.getPodcastById("p1")

        assertEquals(podcast, retrieved)
    }

    @Test
    fun should_ReturnNull_When_PodcastNotFound() = runTest {
        val result = repository.getPodcastById("nonexistent")

        assertNull(result)
    }

    @Test
    fun should_UpdateExisting_When_SavingWithSameId() = runTest {
        repository.savePodcast(testPodcast(title = "Original"))
        repository.savePodcast(testPodcast(title = "Updated"))

        val retrieved = repository.getPodcastById("p1")

        assertEquals("Updated", retrieved?.title)
    }

    @Test
    fun should_RemovePodcast_When_Deleted() = runTest {
        repository.savePodcast(testPodcast())
        repository.deletePodcast("p1")

        assertNull(repository.getPodcastById("p1"))
    }

    @Test
    fun should_EmitUpdatedList_When_ObservingPodcasts() = runTest {
        repository.savePodcast(testPodcast("p1", "Alpha"))
        repository.savePodcast(testPodcast("p2", "Beta"))

        val podcasts = repository.observeAllPodcasts().first()

        assertEquals(2, podcasts.size)
        assertEquals("Alpha", podcasts[0].title)
        assertEquals("Beta", podcasts[1].title)
    }

    @Test
    fun should_SetUnsubscribed_When_MarkUnsubscribedCalled() = runTest {
        repository.savePodcast(testPodcast())
        repository.markUnsubscribed("p1")

        // observeAllPodcasts only returns subscribed podcasts
        val subscribed = repository.observeAllPodcasts().first()
        assertTrue(subscribed.isEmpty())

        // but the podcast still exists in DB
        val podcast = repository.getPodcastById("p1")
        assertEquals(false, podcast?.isSubscribed)
    }

    @Test
    fun should_SaveAll_When_BatchSaving() = runTest {
        val podcasts = listOf(
            testPodcast("p1", "First"),
            testPodcast("p2", "Second"),
            testPodcast("p3", "Third"),
        )

        repository.savePodcasts(podcasts)

        val all = repository.observeAllPodcasts().first()
        assertEquals(3, all.size)
    }
}
