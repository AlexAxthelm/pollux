package com.alexaxthelm.pollux.presentation.downloads

import com.alexaxthelm.pollux.domain.download.DownloadItem

sealed interface DownloadsState {
    data object Empty : DownloadsState

    data class Loaded(
        val active: DownloadItem?,
        val queued: List<DownloadItem>,
        val failed: List<DownloadItem>,
    ) : DownloadsState
}
