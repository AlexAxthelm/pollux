package com.alexaxthelm.pollux

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import cafe.adriel.voyager.navigator.Navigator
import com.alexaxthelm.pollux.data.database.DatabaseDriverFactory
import com.alexaxthelm.pollux.data.database.PolluxDatabase
import com.alexaxthelm.pollux.data.feed.KtorFeedParser
import com.alexaxthelm.pollux.data.repository.EpisodeRepositoryImpl
import com.alexaxthelm.pollux.data.repository.PodcastRepositoryImpl
import com.alexaxthelm.pollux.domain.usecase.RefreshAllPodcastsUseCase
import com.alexaxthelm.pollux.domain.usecase.SubscribeToPodcastUseCase
import com.alexaxthelm.pollux.ui.library.LibraryScreen
import io.ktor.client.HttpClient

@Composable
fun App() {
    val deps = remember {
        val database = PolluxDatabase(DatabaseDriverFactory().createDriver())
        val podcastRepo = PodcastRepositoryImpl(database)
        val episodeRepo = EpisodeRepositoryImpl(database)
        val feedParser = KtorFeedParser(HttpClient())
        AppDependencies(
            podcastRepo = podcastRepo,
            episodeRepo = episodeRepo,
            subscribeUseCase = SubscribeToPodcastUseCase(
                feedParser = feedParser,
                podcastRepository = podcastRepo,
                episodeRepository = episodeRepo,
            ),
            refreshAllUseCase = RefreshAllPodcastsUseCase(
                feedParser = feedParser,
                podcastRepository = podcastRepo,
                episodeRepository = episodeRepo,
            ),
        )
    }
    MaterialTheme {
        Navigator(LibraryScreen(deps))
    }
}
