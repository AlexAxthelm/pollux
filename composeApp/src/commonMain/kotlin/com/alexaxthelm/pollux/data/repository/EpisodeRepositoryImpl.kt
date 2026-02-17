package com.alexaxthelm.pollux.data.repository

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import com.alexaxthelm.pollux.data.database.PolluxDatabase
import com.alexaxthelm.pollux.data.database.mapper.EpisodeMapper
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext

class EpisodeRepositoryImpl(
    private val database: PolluxDatabase,
) : EpisodeRepository {

    private val queries get() = database.polluxDatabaseQueries

    override fun observeEpisodesByPodcast(podcastId: String): Flow<List<Episode>> =
        queries.selectEpisodesByPodcast(podcastId)
            .asFlow()
            .mapToList(Dispatchers.Default)
            .map { list -> list.map(EpisodeMapper::toDomain) }

    override suspend fun getEpisodesByPodcast(podcastId: String): List<Episode> =
        withContext(Dispatchers.Default) {
            queries.selectEpisodesByPodcast(podcastId).executeAsList().map(EpisodeMapper::toDomain)
        }

    override suspend fun getEpisodeById(id: String): Episode? = withContext(Dispatchers.Default) {
        queries.selectEpisodeById(id).executeAsOneOrNull()?.let(EpisodeMapper::toDomain)
    }

    override suspend fun saveEpisode(episode: Episode) = withContext(Dispatchers.Default) {
        val db = EpisodeMapper.fromDomain(episode)
        queries.insertOrReplaceEpisode(
            id = db.id,
            podcastId = db.podcastId,
            title = db.title,
            description = db.description,
            audioUrl = db.audioUrl,
            artworkUrl = db.artworkUrl,
            publishDateEpochMillis = db.publishDateEpochMillis,
            durationSeconds = db.durationSeconds,
            episodeNumber = db.episodeNumber,
            isPlayed = db.isPlayed,
            playPositionSeconds = db.playPositionSeconds,
            isDownloaded = db.isDownloaded,
            localPath = db.localPath,
        )
    }

    override suspend fun saveEpisodes(episodes: List<Episode>) = withContext(Dispatchers.Default) {
        queries.transaction {
            episodes.forEach { episode ->
                val db = EpisodeMapper.fromDomain(episode)
                queries.insertOrReplaceEpisode(
                    id = db.id,
                    podcastId = db.podcastId,
                    title = db.title,
                    description = db.description,
                    audioUrl = db.audioUrl,
                    artworkUrl = db.artworkUrl,
                    publishDateEpochMillis = db.publishDateEpochMillis,
                    durationSeconds = db.durationSeconds,
                    episodeNumber = db.episodeNumber,
                    isPlayed = db.isPlayed,
                    playPositionSeconds = db.playPositionSeconds,
                    isDownloaded = db.isDownloaded,
                    localPath = db.localPath,
                )
            }
        }
    }

    override suspend fun deleteEpisode(id: String) = withContext(Dispatchers.Default) {
        queries.deleteEpisode(id)
    }

    override suspend fun deleteEpisodesByPodcast(podcastId: String) =
        withContext(Dispatchers.Default) {
            queries.deleteEpisodesByPodcast(podcastId)
        }

    override suspend fun markEpisodePlayed(id: String, played: Boolean) =
        withContext(Dispatchers.Default) {
            queries.markEpisodePlayed(isPlayed = if (played) 1L else 0L, id = id)
        }

    override suspend fun updatePlayPosition(id: String, positionSeconds: Int) =
        withContext(Dispatchers.Default) {
            queries.updatePlayPosition(playPositionSeconds = positionSeconds.toLong(), id = id)
        }

    override suspend fun updateDownloadStatus(id: String, isDownloaded: Boolean, localPath: String?) =
        withContext(Dispatchers.Default) {
            queries.updateDownloadStatus(
                isDownloaded = if (isDownloaded) 1L else 0L,
                localPath = localPath,
                id = id,
            )
        }
}
