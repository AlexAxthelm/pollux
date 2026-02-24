package com.alexaxthelm.pollux.data.download

import com.alexaxthelm.pollux.data.storage.FileStorage
import com.alexaxthelm.pollux.domain.download.DownloadItem
import com.alexaxthelm.pollux.domain.download.DownloadStatus
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
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
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.hours
import kotlin.time.Instant

class DownloadManagerImplTest {

    private val testDispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setup() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    // --- Helpers ---

    /** Manager where all coroutines run on [testDispatcher] — for queue/state tests. */
    private fun createManager(
        engine: MockEngine = successEngine(),
        fileStorage: FakeFileStorage = FakeFileStorage(),
        episodeRepo: DownloadFakeEpisodeRepository = DownloadFakeEpisodeRepository(),
    ) = DownloadManagerImpl(
        httpClient = HttpClient(engine),
        fileStorage = fileStorage,
        episodeRepository = episodeRepo,
        scope = CoroutineScope(testDispatcher + SupervisorJob()),
        fileDispatcher = testDispatcher,
    )

    /** Manager using real [Dispatchers.Default] — for tests that involve Ktor HTTP (MockEngine uses Dispatchers.IO). */
    private fun createRealScopeManager(
        engine: MockEngine = successEngine(),
        fileStorage: FakeFileStorage = FakeFileStorage(),
        episodeRepo: DownloadFakeEpisodeRepository = DownloadFakeEpisodeRepository(),
        scope: CoroutineScope = CoroutineScope(Dispatchers.Default + SupervisorJob()),
    ) = DownloadManagerImpl(
        httpClient = HttpClient(engine),
        fileStorage = fileStorage,
        episodeRepository = episodeRepo,
        scope = scope,
        fileDispatcher = Dispatchers.Default,
    )

    private fun successEngine(body: ByteArray = ByteArray(64) { it.toByte() }) = MockEngine { _ ->
        respond(
            content = body,
            status = HttpStatusCode.OK,
            headers = headersOf(HttpHeaders.ContentLength, body.size.toString()),
        )
    }

    private fun errorEngine() = MockEngine { _ ->
        respond(content = "Server Error", status = HttpStatusCode.InternalServerError)
    }

    private fun testEpisode(id: String, isDownloaded: Boolean = false) = Episode(
        id = id,
        podcastId = "pod1",
        title = "Episode $id",
        audioUrl = "https://example.com/$id.mp3",
        publishDate = Instant.parse("2024-06-01T00:00:00Z"),
        duration = 1.hours,
        isDownloaded = isDownloaded,
    )

    // --- Enqueue ---

    @Test
    fun should_AddToQueue_When_Enqueued() = runTest(testDispatcher) {
        val manager = createManager()
        val episode = testEpisode("ep1")

        manager.enqueue(episode)

        val downloads = manager.observeDownloads().first()
        assertEquals(1, downloads.size)
        assertEquals(episode.id, downloads.first().episode.id)
        assertIs<DownloadStatus.Queued>(downloads.first().status)
    }

    @Test
    fun should_NotDuplicate_When_SameEpisodeEnqueuedTwice() = runTest(testDispatcher) {
        val manager = createManager()
        val episode = testEpisode("ep1")

        manager.enqueue(episode)
        manager.enqueue(episode)

        assertEquals(1, manager.observeDownloads().first().size)
    }

    @Test
    fun should_IgnoreAlreadyDownloaded_When_Enqueued() = runTest(testDispatcher) {
        val manager = createManager()
        val episode = testEpisode("ep1", isDownloaded = true)

        manager.enqueue(episode)

        assertEquals(0, manager.observeDownloads().first().size)
    }

    // --- Cancel ---

    @Test
    fun should_RemoveFromList_When_Cancelled() = runTest(testDispatcher) {
        val manager = createManager()
        manager.enqueue(testEpisode("ep1"))

        manager.cancel("ep1")

        assertEquals(0, manager.observeDownloads().first().size)
    }

    @Test
    fun should_NotAffectOtherItems_When_OneItemCancelled() = runTest(testDispatcher) {
        val manager = createManager()
        manager.enqueue(testEpisode("ep1"))
        manager.enqueue(testEpisode("ep2"))

        manager.cancel("ep1")

        val downloads = manager.observeDownloads().first()
        assertEquals(1, downloads.size)
        assertEquals("ep2", downloads.first().episode.id)
    }

    // --- Retry ---

    // Uses runBlocking + real scope because Ktor's MockEngine dispatches to Dispatchers.IO,
    // which is not controlled by StandardTestDispatcher.advanceUntilIdle().
    @Test
    fun should_MoveToQueued_When_FailedItemRetried() {
        // Use a counting engine: each HTTP request increments the counter.
        // After retry() re-enqueues ep1, a second download attempt must occur.
        val downloadAttempts = java.util.concurrent.atomic.AtomicInteger(0)
        val countingErrorEngine = MockEngine { _ ->
            downloadAttempts.incrementAndGet()
            respond(content = "Server Error", status = HttpStatusCode.InternalServerError)
        }
        val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
        val manager = createRealScopeManager(engine = countingErrorEngine, scope = scope)

        manager.enqueue(testEpisode("ep1"))

        // Wait for the first failure
        runBlocking {
            withTimeout(5_000L) {
                manager.observeDownloads().first { list ->
                    list.any { it.episode.id == "ep1" && it.status is DownloadStatus.Failed }
                }
            }
        }

        manager.retry("ep1")

        // Retry re-enqueues the item which triggers a second download attempt.
        // The transient Queued→InProgress states may race past observation, so instead verify the
        // durable side effect: a second HTTP request was made and the item settled into Failed again.
        runBlocking {
            withTimeout(5_000L) {
                manager.observeDownloads().first { list ->
                    list.any { it.episode.id == "ep1" && it.status is DownloadStatus.Failed }
                        && downloadAttempts.get() >= 2
                }
            }
        }

        assertEquals(2, downloadAttempts.get())
        scope.cancel()
    }

    // --- Remove ---

    @Test
    fun should_RemoveFromList_When_FailedItemRemoved() {
        val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
        val manager = createRealScopeManager(engine = errorEngine(), scope = scope)

        manager.enqueue(testEpisode("ep1"))

        runBlocking {
            withTimeout(5_000L) {
                manager.observeDownloads().first { list ->
                    list.any { it.episode.id == "ep1" && it.status is DownloadStatus.Failed }
                }
            }
        }

        manager.remove("ep1")

        assertEquals(0, runBlocking { manager.observeDownloads().first() }.size)
        scope.cancel()
    }

    // --- Full download flow ---

    @Test
    fun should_UpdateEpisodeRepo_And_ClearList_When_DownloadSucceeds() {
        val fileStorage = FakeFileStorage()
        val episodeRepo = DownloadFakeEpisodeRepository()
        val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
        val manager = createRealScopeManager(
            fileStorage = fileStorage,
            episodeRepo = episodeRepo,
            scope = scope,
        )

        manager.enqueue(testEpisode("ep1"))

        // Wait until the download list becomes empty (completed items are removed)
        runBlocking {
            withTimeout(5_000L) {
                manager.observeDownloads().first { it.isEmpty() }
            }
        }

        // File was written
        assertEquals(1, fileStorage.writtenPaths.size)
        assertTrue(fileStorage.writtenPaths.first().contains("ep1"))

        // Repository was updated
        assertEquals(1, episodeRepo.downloadStatusUpdates.size)
        val (id, isDownloaded, path) = episodeRepo.downloadStatusUpdates.first()
        assertEquals("ep1", id)
        assertEquals(true, isDownloaded)
        assertTrue(path != null && path.contains("ep1"))

        scope.cancel()
    }

    @Test
    fun should_SetFailedStatus_When_HttpFails() {
        val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
        val manager = createRealScopeManager(engine = errorEngine(), scope = scope)

        manager.enqueue(testEpisode("ep1"))

        val failedList = runBlocking {
            withTimeout(5_000L) {
                manager.observeDownloads().first { list ->
                    list.any { it.status is DownloadStatus.Failed }
                }
            }
        }

        assertIs<DownloadStatus.Failed>(failedList.first { it.episode.id == "ep1" }.status)
        scope.cancel()
    }

    @Test
    fun should_ProcessNextInQueue_After_FirstDownloadCompletes() {
        val episodeRepo = DownloadFakeEpisodeRepository()
        val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
        val manager = createRealScopeManager(episodeRepo = episodeRepo, scope = scope)

        manager.enqueue(testEpisode("ep1"))
        manager.enqueue(testEpisode("ep2"))

        // Wait for both downloads to finish (list becomes empty)
        runBlocking {
            withTimeout(5_000L) {
                manager.observeDownloads().first { it.isEmpty() }
            }
        }

        val updatedIds = episodeRepo.downloadStatusUpdates.map { it.first }
        assertTrue("ep1" in updatedIds)
        assertTrue("ep2" in updatedIds)
        scope.cancel()
    }

    // --- deleteDownload ---

    @Test
    fun should_DeleteFileAndUpdateRepo_When_DeleteDownloadCalled() = runTest(testDispatcher) {
        val fileStorage = FakeFileStorage()
        val episodeRepo = DownloadFakeEpisodeRepository()
        val manager = createManager(fileStorage = fileStorage, episodeRepo = episodeRepo)
        val episode = testEpisode("ep1").copy(isDownloaded = true, localPath = "/fake/ep1.mp3")

        manager.deleteDownload(episode)
        advanceUntilIdle()

        assertEquals(listOf("/fake/ep1.mp3"), fileStorage.deletedPaths)
        val (id, isDownloaded, path) = episodeRepo.downloadStatusUpdates.first()
        assertEquals("ep1", id)
        assertEquals(false, isDownloaded)
        assertEquals(null, path)
    }
}

// --- Fakes ---

private class FakeFileStorage : FileStorage {
    val writtenPaths = mutableListOf<String>()
    val deletedPaths = mutableListOf<String>()

    override fun getDownloadPath(episodeId: String): String = "/fake/downloads/$episodeId.mp3"
    override fun fileExists(path: String): Boolean = false
    override fun deleteFile(path: String) { deletedPaths.add(path) }
    override fun writeBytes(path: String, bytes: ByteArray) { writtenPaths.add(path) }
}

private class DownloadFakeEpisodeRepository : EpisodeRepository {
    val downloadStatusUpdates = mutableListOf<Triple<String, Boolean, String?>>()

    override suspend fun updateDownloadStatus(id: String, isDownloaded: Boolean, localPath: String?) {
        downloadStatusUpdates.add(Triple(id, isDownloaded, localPath))
    }

    override fun observeEpisodeById(id: String): Flow<Episode?> = MutableStateFlow(null)
    override fun observeEpisodesByPodcast(podcastId: String): Flow<List<Episode>> =
        MutableStateFlow(emptyList())
    override suspend fun getEpisodeById(id: String): Episode? = null
    override suspend fun getEpisodesByPodcast(podcastId: String): List<Episode> = emptyList()
    override suspend fun saveEpisode(episode: Episode) = Unit
    override suspend fun saveEpisodes(episodes: List<Episode>) = Unit
    override suspend fun deleteEpisode(id: String) = Unit
    override suspend fun deleteEpisodesByPodcast(podcastId: String) = Unit
    override suspend fun markEpisodePlayed(id: String, played: Boolean) = Unit
    override suspend fun updatePlayPosition(id: String, positionSeconds: Int) = Unit
}
