package com.alexaxthelm.pollux.presentation.library

import com.alexaxthelm.pollux.domain.model.Podcast

sealed interface LibraryState {
    data object Loading : LibraryState
    data class Loaded(val podcasts: List<Podcast>, val isRefreshing: Boolean = false) : LibraryState
}
