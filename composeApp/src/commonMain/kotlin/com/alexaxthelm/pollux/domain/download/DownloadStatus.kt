package com.alexaxthelm.pollux.domain.download

sealed interface DownloadStatus {
    data object Queued : DownloadStatus

    data class InProgress(
        val bytesDownloaded: Long,
        val totalBytes: Long?,
    ) : DownloadStatus

    data object Completed : DownloadStatus

    data class Failed(val message: String) : DownloadStatus
}
