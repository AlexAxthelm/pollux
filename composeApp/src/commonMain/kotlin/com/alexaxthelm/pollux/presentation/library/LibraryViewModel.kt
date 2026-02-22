package com.alexaxthelm.pollux.presentation.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class LibraryViewModel(
    podcastRepository: PodcastRepository,
) : ViewModel() {

    val state: StateFlow<LibraryState> = podcastRepository
        .observeAllPodcasts()
        .map { LibraryState.Loaded(it) }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.Eagerly,
            initialValue = LibraryState.Loading,
        )
}
