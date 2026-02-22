package com.alexaxthelm.pollux.domain.feed

interface FeedParser {
    suspend fun parse(url: String): ParsedFeed
}
