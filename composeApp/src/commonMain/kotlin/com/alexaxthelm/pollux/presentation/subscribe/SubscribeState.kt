package com.alexaxthelm.pollux.presentation.subscribe

import com.alexaxthelm.pollux.domain.feed.ParsedFeed

sealed interface SubscribeState {
    data object Idle : SubscribeState
    data object Loading : SubscribeState
    data class Preview(val feed: ParsedFeed) : SubscribeState
    data class Error(val message: String) : SubscribeState
    data object Saved : SubscribeState
}
