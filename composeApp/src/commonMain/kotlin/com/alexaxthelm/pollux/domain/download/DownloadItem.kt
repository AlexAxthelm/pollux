package com.alexaxthelm.pollux.domain.download

import com.alexaxthelm.pollux.domain.model.Episode

data class DownloadItem(
    val episode: Episode,
    val status: DownloadStatus,
)
