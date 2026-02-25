package com.alexaxthelm.pollux

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import cafe.adriel.voyager.navigator.Navigator
import com.alexaxthelm.pollux.data.database.DatabaseDriverFactory
import com.alexaxthelm.pollux.data.database.PolluxDatabase
import com.alexaxthelm.pollux.data.download.DownloadManagerImpl
import com.alexaxthelm.pollux.data.feed.KtorFeedParser
import com.alexaxthelm.pollux.data.repository.EpisodeRepositoryImpl
import com.alexaxthelm.pollux.data.repository.PodcastRepositoryImpl
import com.alexaxthelm.pollux.data.storage.createPlatformFileStorage
import com.alexaxthelm.pollux.domain.usecase.RefreshAllPodcastsUseCase
import com.alexaxthelm.pollux.domain.usecase.SubscribeToPodcastUseCase
import com.alexaxthelm.pollux.ui.library.LibraryScreen
import io.ktor.client.HttpClient

@Composable
fun App() {
    val appScope = rememberCoroutineScope()
    val deps = remember {
        val database = PolluxDatabase(DatabaseDriverFactory().createDriver())
        val podcastRepo = PodcastRepositoryImpl(database)
        val episodeRepo = EpisodeRepositoryImpl(database)
        val httpClient = HttpClient()
        val feedParser = KtorFeedParser(httpClient)
        val fileStorage = createPlatformFileStorage()
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
            downloadManager = DownloadManagerImpl(
                httpClient = httpClient,
                fileStorage = fileStorage,
                episodeRepository = episodeRepo,
                scope = appScope,
            ),
        )
    }
    MaterialTheme {
        Navigator(LibraryScreen(deps))
    }
}
