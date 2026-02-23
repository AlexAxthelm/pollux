package com.alexaxthelm.pollux.presentation.detail

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

class PodcastDetailViewModel(
    private val podcastId: String,
    private val podcastRepo: PodcastRepository,
    private val episodeRepo: EpisodeRepository,
) : ViewModel() {

    val state: StateFlow<PodcastDetailState> = flow {
        val podcast = podcastRepo.getPodcastById(podcastId) ?: return@flow
        episodeRepo.observeEpisodesByPodcast(podcastId).collect { episodes ->
            emit(PodcastDetailState.Loaded(podcast, episodes))
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, PodcastDetailState.Loading)

    fun markEpisodePlayed(episodeId: String, played: Boolean) {
        viewModelScope.launch {
            episodeRepo.markEpisodePlayed(episodeId, played)
        }
    }
}
