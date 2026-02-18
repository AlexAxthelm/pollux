package com.alexaxthelm.pollux.domain.repository

import com.alexaxthelm.pollux.domain.model.Episode
import kotlinx.coroutines.flow.Flow

interface EpisodeRepository {
    fun observeEpisodesByPodcast(podcastId: String): Flow<List<Episode>>
    suspend fun getEpisodesByPodcast(podcastId: String): List<Episode>
    suspend fun getEpisodeById(id: String): Episode?
    suspend fun saveEpisode(episode: Episode)
    suspend fun saveEpisodes(episodes: List<Episode>)
    suspend fun deleteEpisode(id: String)
    suspend fun deleteEpisodesByPodcast(podcastId: String)
    suspend fun markEpisodePlayed(id: String, played: Boolean)
    suspend fun updatePlayPosition(id: String, positionSeconds: Int)
    suspend fun updateDownloadStatus(id: String, isDownloaded: Boolean, localPath: String?)
}
