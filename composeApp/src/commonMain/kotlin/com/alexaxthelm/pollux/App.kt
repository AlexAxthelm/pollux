package com.alexaxthelm.pollux

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.alexaxthelm.pollux.data.database.DatabaseDriverFactory
import com.alexaxthelm.pollux.data.database.PolluxDatabase
import com.alexaxthelm.pollux.data.feed.KtorFeedParser
import com.alexaxthelm.pollux.data.repository.EpisodeRepositoryImpl
import com.alexaxthelm.pollux.data.repository.PodcastRepositoryImpl
import com.alexaxthelm.pollux.domain.usecase.SubscribeToPodcastUseCase
import com.alexaxthelm.pollux.presentation.subscribe.SubscribeState
import com.alexaxthelm.pollux.presentation.subscribe.SubscribeViewModel
import com.alexaxthelm.pollux.ui.subscribe.SubscribeScreen
import com.alexaxthelm.pollux.ui.subscribe.SubscriptionPreviewScreen
import io.ktor.client.HttpClient

@Composable
fun App() {
    val database = remember {
        val driver = DatabaseDriverFactory().createDriver()
        PolluxDatabase(driver)
    }
    val useCase = remember(database) {
        SubscribeToPodcastUseCase(
            feedParser = KtorFeedParser(HttpClient()),
            podcastRepository = PodcastRepositoryImpl(database),
            episodeRepository = EpisodeRepositoryImpl(database),
        )
    }
    val viewModel = viewModel { SubscribeViewModel(useCase) }
    val state by viewModel.state.collectAsStateWithLifecycle()

    MaterialTheme {
        when (val s = state) {
            is SubscribeState.Idle -> SubscribeScreen(
                isLoading = false,
                errorMessage = null,
                onSubmit = viewModel::submit,
                onDismissError = viewModel::dismissError,
            )

            is SubscribeState.Loading -> SubscribeScreen(
                isLoading = true,
                errorMessage = null,
                onSubmit = {},
                onDismissError = {},
            )

            is SubscribeState.Error -> SubscribeScreen(
                isLoading = false,
                errorMessage = s.message,
                onSubmit = viewModel::submit,
                onDismissError = viewModel::dismissError,
            )

            is SubscribeState.Preview -> SubscriptionPreviewScreen(
                feed = s.feed,
                isLoading = false,
                onConfirm = viewModel::confirmSubscription,
                onCancel = viewModel::cancelPreview,
            )

            is SubscribeState.Saved -> {
                // Step 1.4 will navigate to the Library here.
                // For now, reset so the user can add another podcast.
                viewModel.cancelPreview()
            }
        }
    }
}
