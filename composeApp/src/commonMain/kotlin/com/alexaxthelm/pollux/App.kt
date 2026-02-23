package com.alexaxthelm.pollux

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import com.alexaxthelm.pollux.data.database.DatabaseDriverFactory
import com.alexaxthelm.pollux.data.database.PolluxDatabase
import com.alexaxthelm.pollux.data.feed.KtorFeedParser
import com.alexaxthelm.pollux.data.repository.EpisodeRepositoryImpl
import com.alexaxthelm.pollux.data.repository.PodcastRepositoryImpl
import com.alexaxthelm.pollux.domain.usecase.SubscribeToPodcastUseCase
import com.alexaxthelm.pollux.presentation.library.LibraryViewModel
import com.alexaxthelm.pollux.presentation.subscribe.SubscribeState
import com.alexaxthelm.pollux.presentation.subscribe.SubscribeViewModel
import com.alexaxthelm.pollux.ui.library.LibraryScreen
import com.alexaxthelm.pollux.ui.subscribe.SubscribeScreen
import io.ktor.client.HttpClient

private enum class Destination { Library, Subscribe }

@Composable
fun App() {
    val database = remember {
        val driver = DatabaseDriverFactory().createDriver()
        PolluxDatabase(driver)
    }

    // Repos created once and shared between both ViewModels so they see the same data.
    val podcastRepo = remember(database) { PodcastRepositoryImpl(database) }
    val episodeRepo = remember(database) { EpisodeRepositoryImpl(database) }

    val subscribeUseCase = remember(podcastRepo, episodeRepo) {
        SubscribeToPodcastUseCase(
            feedParser = KtorFeedParser(HttpClient()),
            podcastRepository = podcastRepo,
            episodeRepository = episodeRepo,
        )
    }

    val libraryViewModel = remember(podcastRepo) { LibraryViewModel(podcastRepo) }
    val subscribeViewModel = remember(subscribeUseCase) { SubscribeViewModel(subscribeUseCase) }

    var destination by remember { mutableStateOf(Destination.Library) }

    MaterialTheme {
        when (destination) {
            Destination.Library -> {
                val state by libraryViewModel.state.collectAsState()
                LibraryScreen(
                    state = state,
                    onAddPodcast = { destination = Destination.Subscribe },
                )
            }

            Destination.Subscribe -> {
                val state by subscribeViewModel.state.collectAsState()

                if (state is SubscribeState.Saved) {
                    // Reset the subscribe flow and return to Library.
                    // Must be in LaunchedEffect — mutating state during composition is a bug.
                    LaunchedEffect(Unit) {
                        subscribeViewModel.cancelPreview()
                        destination = Destination.Library
                    }
                } else {
                    SubscribeScreen(
                        state = state,
                        onSubmit = subscribeViewModel::submit,
                        onConfirm = subscribeViewModel::confirmSubscription,
                        onCancel = {
                            subscribeViewModel.cancelPreview()
                            destination = Destination.Library
                        },
                        onDismissError = subscribeViewModel::dismissError,
                    )
                }
            }
        }
    }
}
