package com.alexaxthelm.pollux.presentation.detail

import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast

sealed interface EpisodeDetailState {
    data object Loading : EpisodeDetailState
    data class Loaded(
        val episode: Episode,
        val podcast: Podcast,
    ) : EpisodeDetailState
}
