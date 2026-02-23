package com.alexaxthelm.pollux.presentation.detail

import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast

sealed interface PodcastDetailState {
    data object Loading : PodcastDetailState
    data class Loaded(
        val podcast: Podcast,
        val episodes: List<Episode>,
    ) : PodcastDetailState
}
