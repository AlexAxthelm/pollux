package com.alexaxthelm.pollux.domain.feed

data class ParsedFeed(
    val feedUrl: String,
    val title: String,
    val author: String?,
    val description: String?,
    val artworkUrl: String?,
    val episodes: List<ParsedEpisode>,
)
