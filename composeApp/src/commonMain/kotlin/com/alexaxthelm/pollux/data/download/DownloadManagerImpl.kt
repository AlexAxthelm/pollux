package com.alexaxthelm.pollux.data.download

import com.alexaxthelm.pollux.data.storage.FileStorage
import com.alexaxthelm.pollux.domain.download.DownloadItem
import com.alexaxthelm.pollux.domain.download.DownloadManager
import com.alexaxthelm.pollux.domain.download.DownloadStatus
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import io.ktor.client.HttpClient
import io.ktor.client.request.get
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsChannel
import io.ktor.http.contentLength
import io.ktor.http.isSuccess
import io.ktor.utils.io.readAvailable
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

            val totalBytes = response.contentLength()
            updateStatus(episode.id, DownloadStatus.InProgress(bytesDownloaded = 0L, totalBytes = totalBytes))

            // Stream the response body in chunks so we can report real download progress.
            val channel = response.bodyAsChannel()
            val chunkBuffer = ByteArray(CHUNK_SIZE)
            val chunks = mutableListOf<ByteArray>()
            var bytesDownloaded = 0L

            while (!channel.isClosedForRead) {
                val read = channel.readAvailable(chunkBuffer, 0, chunkBuffer.size)
                if (read <= 0) break
                chunks.add(chunkBuffer.copyOfRange(0, read))
                bytesDownloaded += read
                updateStatus(episode.id, DownloadStatus.InProgress(bytesDownloaded, totalBytes))
            }

            // If the user cancelled while we were downloading, discard the result.
            if (_downloads.value.none { it.episode.id == episode.id }) return

            // Combine all received chunks into a single array for writing.
            val bytes = ByteArray(bytesDownloaded.toInt())
            var writeOffset = 0
            for (chunk in chunks) {
                chunk.copyInto(bytes, writeOffset)
                writeOffset += chunk.size
            }

            withContext(fileDispatcher) {
                fileStorage.writeBytes(path, bytes)
            }

            episodeRepository.updateDownloadStatus(episode.id, isDownloaded = true, localPath = path)
            updateStatus(episode.id, DownloadStatus.Completed)

            // Remove completed items immediately; the episode's isDownloaded flag is the source of truth.
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

    companion object {
        private const val CHUNK_SIZE = 8 * 1024 // 8 KB
    }
}
