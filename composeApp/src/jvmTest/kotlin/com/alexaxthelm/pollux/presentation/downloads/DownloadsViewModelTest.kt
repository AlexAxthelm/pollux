package com.alexaxthelm.pollux.presentation.downloads

import com.alexaxthelm.pollux.domain.download.DownloadItem
import com.alexaxthelm.pollux.domain.download.DownloadManager
import com.alexaxthelm.pollux.domain.download.DownloadStatus
import com.alexaxthelm.pollux.domain.model.Episode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
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
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.hours
import kotlin.time.Instant

class DownloadsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setup() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun testEpisode(id: String) = Episode(
        id = id,
        podcastId = "pod1",
        title = "Episode $id",
        audioUrl = "https://example.com/$id.mp3",
        publishDate = Instant.parse("2024-06-01T00:00:00Z"),
        duration = 1.hours,
    )

    // --- State mapping ---

    @Test
    fun should_EmitEmpty_When_NoDownloads() = runTest(testDispatcher) {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)
        advanceUntilIdle()

        assertIs<DownloadsState.Empty>(vm.state.value)
    }

    @Test
    fun should_EmitLoaded_With_ActiveItem_When_InProgressDownloadExists() = runTest(testDispatcher) {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)
        val episode = testEpisode("ep1")
        val item = DownloadItem(episode, DownloadStatus.InProgress(512L, 1024L))

        fakeManager.emit(listOf(item))
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<DownloadsState.Loaded>(state)
        assertEquals(episode.id, state.active?.episode?.id)
        assertIs<DownloadStatus.InProgress>(state.active?.status)
        assertTrue(state.queued.isEmpty())
        assertTrue(state.failed.isEmpty())
    }

    @Test
    fun should_EmitLoaded_With_QueuedItems_When_QueuedDownloadsExist() = runTest(testDispatcher) {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)
        val items = listOf(
            DownloadItem(testEpisode("ep1"), DownloadStatus.Queued),
            DownloadItem(testEpisode("ep2"), DownloadStatus.Queued),
        )

        fakeManager.emit(items)
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<DownloadsState.Loaded>(state)
        assertNull(state.active)
        assertEquals(2, state.queued.size)
        assertTrue(state.failed.isEmpty())
    }

    @Test
    fun should_EmitLoaded_With_FailedItems_When_FailedDownloadsExist() = runTest(testDispatcher) {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)
        val items = listOf(
            DownloadItem(testEpisode("ep1"), DownloadStatus.Failed("Network error")),
        )

        fakeManager.emit(items)
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<DownloadsState.Loaded>(state)
        assertNull(state.active)
        assertTrue(state.queued.isEmpty())
        assertEquals(1, state.failed.size)
        assertEquals("Network error", (state.failed.first().status as DownloadStatus.Failed).message)
    }

    @Test
    fun should_SeparateActiveQueuedFailed_When_MixedItemsPresent() = runTest(testDispatcher) {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)
        val items = listOf(
            DownloadItem(testEpisode("ep1"), DownloadStatus.InProgress(0L, null)),
            DownloadItem(testEpisode("ep2"), DownloadStatus.Queued),
            DownloadItem(testEpisode("ep3"), DownloadStatus.Failed("Timeout")),
        )

        fakeManager.emit(items)
        advanceUntilIdle()

        val state = vm.state.value
        assertIs<DownloadsState.Loaded>(state)
        assertEquals("ep1", state.active?.episode?.id)
        assertEquals(1, state.queued.size)
        assertEquals("ep2", state.queued.first().episode.id)
        assertEquals(1, state.failed.size)
        assertEquals("ep3", state.failed.first().episode.id)
    }

    @Test
    fun should_UpdateState_When_DownloadsChange() = runTest(testDispatcher) {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)

        fakeManager.emit(listOf(DownloadItem(testEpisode("ep1"), DownloadStatus.Queued)))
        advanceUntilIdle()
        assertIs<DownloadsState.Loaded>(vm.state.value)

        fakeManager.emit(emptyList())
        advanceUntilIdle()
        assertIs<DownloadsState.Empty>(vm.state.value)
    }

    // --- Delegation ---

    @Test
    fun should_DelegateCancel_To_DownloadManager() {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)

        vm.cancel("ep1")

        assertEquals(listOf("ep1"), fakeManager.cancelledIds)
    }

    @Test
    fun should_DelegateRetry_To_DownloadManager() {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)

        vm.retry("ep1")

        assertEquals(listOf("ep1"), fakeManager.retriedIds)
    }

    @Test
    fun should_DelegateRemove_To_DownloadManager() {
        val fakeManager = FakeDownloadManager()
        val vm = DownloadsViewModel(fakeManager)

        vm.remove("ep1")

        assertEquals(listOf("ep1"), fakeManager.removedIds)
    }
}

// --- Fake ---

private class FakeDownloadManager : DownloadManager {
    private val _downloads = MutableStateFlow<List<DownloadItem>>(emptyList())
    val cancelledIds = mutableListOf<String>()
    val retriedIds = mutableListOf<String>()
    val removedIds = mutableListOf<String>()

    fun emit(items: List<DownloadItem>) { _downloads.value = items }

    override fun observeDownloads(): Flow<List<DownloadItem>> = _downloads.asStateFlow()
    override fun enqueue(episode: Episode) = Unit
    override fun cancel(episodeId: String) { cancelledIds.add(episodeId) }
    override fun retry(episodeId: String) { retriedIds.add(episodeId) }
    override fun remove(episodeId: String) { removedIds.add(episodeId) }
    override fun deleteDownload(episode: Episode) = Unit
}
