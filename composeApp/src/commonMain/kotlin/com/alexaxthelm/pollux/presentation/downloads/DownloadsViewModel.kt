package com.alexaxthelm.pollux.presentation.downloads

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.alexaxthelm.pollux.domain.download.DownloadManager
import com.alexaxthelm.pollux.domain.download.DownloadStatus
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class DownloadsViewModel(private val downloadManager: DownloadManager) : ViewModel() {

    val state: StateFlow<DownloadsState> = downloadManager.observeDownloads()
        .map { downloads ->
            if (downloads.isEmpty()) return@map DownloadsState.Empty
            DownloadsState.Loaded(
                active = downloads.firstOrNull { it.status is DownloadStatus.InProgress },
                queued = downloads.filter { it.status == DownloadStatus.Queued },
                failed = downloads.filter { it.status is DownloadStatus.Failed },
            )
        }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.Eagerly,
            initialValue = DownloadsState.Empty,
        )

    fun cancel(episodeId: String) = downloadManager.cancel(episodeId)

    fun retry(episodeId: String) = downloadManager.retry(episodeId)

    fun remove(episodeId: String) = downloadManager.remove(episodeId)
}
