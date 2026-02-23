package com.alexaxthelm.pollux.presentation.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import com.alexaxthelm.pollux.domain.usecase.RefreshAllPodcastsUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

class LibraryViewModel(
    podcastRepository: PodcastRepository,
    private val refreshAllUseCase: RefreshAllPodcastsUseCase,
) : ViewModel() {

    private val _isRefreshing = MutableStateFlow(false)

    val state: StateFlow<LibraryState> = combine(
        podcastRepository.observeAllPodcasts(),
        _isRefreshing,
    ) { podcasts, isRefreshing ->
        LibraryState.Loaded(podcasts, isRefreshing)
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.Eagerly,
        initialValue = LibraryState.Loading,
    )

    fun refreshAll() {
        viewModelScope.launch {
            _isRefreshing.value = true
            try {
                refreshAllUseCase.refresh()
            } finally {
                _isRefreshing.value = false
            }
        }
    }
}
