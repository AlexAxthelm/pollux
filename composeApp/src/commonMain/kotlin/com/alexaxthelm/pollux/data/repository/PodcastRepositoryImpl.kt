package com.alexaxthelm.pollux.data.repository

import app.cash.sqldelight.coroutines.asFlow
import app.cash.sqldelight.coroutines.mapToList
import com.alexaxthelm.pollux.data.database.PolluxDatabase
import com.alexaxthelm.pollux.data.database.mapper.PodcastMapper
import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext

class PodcastRepositoryImpl(
    private val database: PolluxDatabase,
) : PodcastRepository {

    private val queries get() = database.polluxDatabaseQueries

    override fun observeAllPodcasts(): Flow<List<Podcast>> =
        queries.selectAllPodcasts()
            .asFlow()
            .mapToList(Dispatchers.Default)
            .map { list -> list.map(PodcastMapper::toDomain) }

    override suspend fun getPodcastById(id: String): Podcast? = withContext(Dispatchers.Default) {
        queries.selectPodcastById(id).executeAsOneOrNull()?.let(PodcastMapper::toDomain)
    }

    override suspend fun savePodcast(podcast: Podcast) = withContext(Dispatchers.Default) {
        val db = PodcastMapper.fromDomain(podcast)
        queries.insertOrReplacePodcast(
            id = db.id,
            feedUrl = db.feedUrl,
            title = db.title,
            author = db.author,
            description = db.description,
            artworkUrl = db.artworkUrl,
            lastRefreshedEpochMillis = db.lastRefreshedEpochMillis,
            isSubscribed = db.isSubscribed,
        )
    }

    override suspend fun savePodcasts(podcasts: List<Podcast>) = withContext(Dispatchers.Default) {
        queries.transaction {
            podcasts.forEach { podcast ->
                val db = PodcastMapper.fromDomain(podcast)
                queries.insertOrReplacePodcast(
                    id = db.id,
                    feedUrl = db.feedUrl,
                    title = db.title,
                    author = db.author,
                    description = db.description,
                    artworkUrl = db.artworkUrl,
                    lastRefreshedEpochMillis = db.lastRefreshedEpochMillis,
                    isSubscribed = db.isSubscribed,
                )
            }
        }
    }

    override suspend fun deletePodcast(id: String) = withContext(Dispatchers.Default) {
        queries.deletePodcast(id)
    }

    override suspend fun markUnsubscribed(id: String) = withContext(Dispatchers.Default) {
        queries.markPodcastUnsubscribed(id)
    }
}
