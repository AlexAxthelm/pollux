package com.alexaxthelm.pollux.data.download

import com.alexaxthelm.pollux.data.storage.FileStorage
import com.alexaxthelm.pollux.domain.download.DownloadItem
import com.alexaxthelm.pollux.domain.download.DownloadManager
import com.alexaxthelm.pollux.domain.download.DownloadStatus
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.statement.HttpResponse
import io.ktor.http.contentLength
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.withContext

class DownloadManagerImpl(
    private val httpClient: HttpClient,
    private val fileStorage: FileStorage,
    private val episodeRepository: EpisodeRepository,
    private val scope: CoroutineScope,
    private val fileDispatcher: CoroutineDispatcher = Dispatchers.Default,
) : DownloadManager {

    private val _downloads = MutableStateFlow<List<DownloadItem>>(emptyList())
    private val processingMutex = Mutex()

    override fun observeDownloads(): Flow<List<DownloadItem>> = _downloads.asStateFlow()

    override fun enqueue(episode: Episode) {
        if (episode.isDownloaded) return
        if (_downloads.value.any { it.episode.id == episode.id }) return
        _downloads.update { it + DownloadItem(episode, DownloadStatus.Queued) }
        processNext()
    }

    override fun cancel(episodeId: String) {
        _downloads.update { items -> items.filterNot { it.episode.id == episodeId } }
    }

    override fun retry(episodeId: String) {
        _downloads.update { items ->
            items.map { item ->
                if (item.episode.id == episodeId && item.status is DownloadStatus.Failed) {
                    item.copy(status = DownloadStatus.Queued)
                } else {
                    item
                }
            }
        }
        processNext()
    }

    override fun remove(episodeId: String) {
        _downloads.update { items -> items.filterNot { it.episode.id == episodeId } }
    }

    override fun deleteDownload(episode: Episode) {
        scope.launch {
            episode.localPath?.let { path ->
                withContext(fileDispatcher) {
                    fileStorage.deleteFile(path)
                }
            }
            episodeRepository.updateDownloadStatus(episode.id, isDownloaded = false, localPath = null)
        }
    }

    private fun processNext() {
        scope.launch {
            if (!processingMutex.tryLock()) return@launch
            try {
                while (true) {
                    val next = _downloads.value.firstOrNull { it.status == DownloadStatus.Queued }
                        ?: break
                    performDownload(next.episode)
                }
            } finally {
                processingMutex.unlock()
            }
        }
    }

    private suspend fun performDownload(episode: Episode) {
        updateStatus(episode.id, DownloadStatus.InProgress(bytesDownloaded = 0L, totalBytes = null))
        try {
            val path = fileStorage.getDownloadPath(episode.id)
            val response: HttpResponse = httpClient.get(episode.audioUrl)

            if (!response.status.isSuccess()) {
                throw Exception("HTTP ${response.status.value}: ${response.status.description}")
            }

            val contentLength = response.contentLength()
            updateStatus(episode.id, DownloadStatus.InProgress(bytesDownloaded = 0L, totalBytes = contentLength))

            val bytes: ByteArray = response.body()

            // If the user cancelled while we were downloading, discard the result.
            if (_downloads.value.none { it.episode.id == episode.id }) return

            withContext(fileDispatcher) {
                fileStorage.writeBytes(path, bytes)
            }

            episodeRepository.updateDownloadStatus(episode.id, isDownloaded = true, localPath = path)
            updateStatus(episode.id, DownloadStatus.Completed)

            // Remove completed items after a brief delay so the UI can show "done" state.
            // For Phase 1 we remove immediately; the episode's isDownloaded flag is the source of truth.
            _downloads.update { items -> items.filterNot { it.episode.id == episode.id } }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            updateStatus(episode.id, DownloadStatus.Failed(e.message ?: "Download failed"))
        }
    }

    private fun updateStatus(episodeId: String, status: DownloadStatus) {
        _downloads.update { items ->
            items.map { item ->
                if (item.episode.id == episodeId) item.copy(status = status) else item
            }
        }
    }
}
