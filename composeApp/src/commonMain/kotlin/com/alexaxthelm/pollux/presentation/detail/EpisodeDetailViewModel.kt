package com.alexaxthelm.pollux.presentation.detail

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

class EpisodeDetailViewModel(
    private val episodeId: String,
    private val episodeRepo: EpisodeRepository,
    private val podcastRepo: PodcastRepository,
) : ViewModel() {

    val state: StateFlow<EpisodeDetailState> = flow {
        val initial = episodeRepo.getEpisodeById(episodeId) ?: return@flow
        val podcast = podcastRepo.getPodcastById(initial.podcastId) ?: return@flow
        episodeRepo.observeEpisodeById(episodeId).filterNotNull().collect { episode ->
            emit(EpisodeDetailState.Loaded(episode, podcast))
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, EpisodeDetailState.Loading)

    fun markEpisodePlayed(played: Boolean) {
        viewModelScope.launch {
            episodeRepo.markEpisodePlayed(episodeId, played)
        }
    }
}
