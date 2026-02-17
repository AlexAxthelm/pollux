package com.alexaxthelm.pollux.domain.model

import kotlinx.datetime.Instant

data class Podcast(
    val id: String,
    val feedUrl: String,
    val title: String,
    val author: String? = null,
    val description: String? = null,
    val artworkUrl: String? = null,
    val lastRefreshed: Instant? = null,
    val isSubscribed: Boolean = true,
)
