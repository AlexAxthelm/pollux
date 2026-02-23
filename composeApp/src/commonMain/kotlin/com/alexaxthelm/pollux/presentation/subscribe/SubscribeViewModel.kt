package com.alexaxthelm.pollux.presentation.subscribe

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.alexaxthelm.pollux.domain.feed.FeedParseException
import com.alexaxthelm.pollux.domain.usecase.SubscribeToPodcastUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class SubscribeViewModel(
    private val useCase: SubscribeToPodcastUseCase,
) : ViewModel() {

    private val _state = MutableStateFlow<SubscribeState>(SubscribeState.Idle)
    val state: StateFlow<SubscribeState> = _state.asStateFlow()

    fun submit(url: String) {
        val trimmed = url.trim()
        if (trimmed.isEmpty()) {
            _state.value = SubscribeState.Error("Please enter a feed URL")
            return
        }
        _state.value = SubscribeState.Loading
        viewModelScope.launch {
            _state.value = try {
                val feed = useCase.preview(trimmed)
                SubscribeState.Preview(feed)
            } catch (e: FeedParseException) {
                SubscribeState.Error(e.message ?: "Failed to fetch feed")
            } catch (e: Exception) {
                SubscribeState.Error("Unexpected error: ${e.message}")
            }
        }
    }

    fun confirmSubscription() {
        val current = _state.value as? SubscribeState.Preview ?: return
        _state.value = SubscribeState.Loading
        viewModelScope.launch {
            _state.value = try {
                useCase.subscribe(current.feed)
                SubscribeState.Saved
            } catch (e: Exception) {
                SubscribeState.Error(e.message ?: "Failed to save subscription")
            }
        }
    }

    fun cancelPreview() {
        _state.value = SubscribeState.Idle
    }

    fun dismissError() {
        _state.value = SubscribeState.Idle
    }
}
