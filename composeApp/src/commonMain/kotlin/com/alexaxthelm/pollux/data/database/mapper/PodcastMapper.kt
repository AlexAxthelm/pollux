package com.alexaxthelm.pollux.data.database.mapper

import com.alexaxthelm.pollux.domain.model.Podcast
import kotlin.time.Instant
import com.alexaxthelm.pollux.data.database.Podcast as DbPodcast

object PodcastMapper {

    fun toDomain(db: DbPodcast): Podcast = Podcast(
        id = db.id,
        feedUrl = db.feedUrl,
        title = db.title,
        author = db.author,
        description = db.description,
        artworkUrl = db.artworkUrl,
        lastRefreshed = db.lastRefreshedEpochMillis?.let { Instant.fromEpochMilliseconds(it) },
        isSubscribed = db.isSubscribed != 0L,
    )

    fun fromDomain(podcast: Podcast): DbPodcast = DbPodcast(
        id = podcast.id,
        feedUrl = podcast.feedUrl,
        title = podcast.title,
        author = podcast.author,
        description = podcast.description,
        artworkUrl = podcast.artworkUrl,
        lastRefreshedEpochMillis = podcast.lastRefreshed?.toEpochMilliseconds(),
        isSubscribed = if (podcast.isSubscribed) 1L else 0L,
    )
}
