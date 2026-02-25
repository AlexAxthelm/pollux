package com.alexaxthelm.pollux.domain.download

import com.alexaxthelm.pollux.domain.model.Episode
import kotlinx.coroutines.flow.Flow

interface DownloadManager {
    /** Observe all active, queued, and failed download items. Completed items are removed automatically. */
    fun observeDownloads(): Flow<List<DownloadItem>>

    /** Add an episode to the download queue. No-op if already downloaded or already queued. */
    fun enqueue(episode: Episode)

    /** Remove a queued or in-progress download. Best-effort for in-progress; does not guarantee abort. */
    fun cancel(episodeId: String)

    /** Move a failed download back to Queued and retry it. */
    fun retry(episodeId: String)

    /** Remove a failed (or completed) download record from the list without retrying. */
    fun remove(episodeId: String)

    /** Delete a downloaded file and mark the episode as not downloaded in the repository. */
    fun deleteDownload(episode: Episode)
}
