package com.alexaxthelm.pollux.domain.repository

import com.alexaxthelm.pollux.domain.model.Podcast
import kotlinx.coroutines.flow.Flow

interface PodcastRepository {
    fun observeAllPodcasts(): Flow<List<Podcast>>
    suspend fun getPodcastById(id: String): Podcast?
    suspend fun savePodcast(podcast: Podcast)
    suspend fun savePodcasts(podcasts: List<Podcast>)
    suspend fun deletePodcast(id: String)
    suspend fun markUnsubscribed(id: String)
}
