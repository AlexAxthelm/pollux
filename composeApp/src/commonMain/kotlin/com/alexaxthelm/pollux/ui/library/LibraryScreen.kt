package com.alexaxthelm.pollux.ui.library

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.alexaxthelm.pollux.AppDependencies
import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.presentation.library.LibraryState
import com.alexaxthelm.pollux.presentation.library.LibraryViewModel
import com.alexaxthelm.pollux.ui.detail.PodcastDetailScreen
import com.alexaxthelm.pollux.ui.subscribe.SubscribeScreen

class LibraryScreen(private val deps: AppDependencies) : Screen {
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow
        val viewModel = remember { LibraryViewModel(deps.podcastRepo) }
        val state by viewModel.state.collectAsState()
        LibraryScreenContent(
            state = state,
            onAddPodcast = { navigator.push(SubscribeScreen(deps)) },
            onPodcastClick = { id -> navigator.push(PodcastDetailScreen(deps, id)) },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LibraryScreenContent(
    state: LibraryState,
    onAddPodcast: () -> Unit,
    onPodcastClick: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(title = { Text("Library") })
        },
        floatingActionButton = {
            FloatingActionButton(onClick = onAddPodcast) {
                Text("+", style = MaterialTheme.typography.titleLarge)
            }
        },
    ) { innerPadding ->
        when (state) {
            is LibraryState.Loading -> {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(innerPadding),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator()
                }
            }

            is LibraryState.Loaded -> {
                if (state.podcasts.isEmpty()) {
                    EmptyLibrary(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(innerPadding),
                    )
                } else {
                    LazyColumn(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(innerPadding),
                        verticalArrangement = Arrangement.spacedBy(1.dp),
                    ) {
                        items(state.podcasts, key = { it.id }) { podcast ->
                            PodcastRow(podcast = podcast, onClick = { onPodcastClick(podcast.id) })
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun PodcastRow(podcast: Podcast, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // Artwork placeholder — real image loading deferred to a later step
        Box(
            modifier = Modifier
                .size(56.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = podcast.title.take(2).uppercase(),
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = podcast.title,
                style = MaterialTheme.typography.bodyLarge,
                maxLines = 1,
            )
            if (podcast.author != null) {
                Text(
                    text = podcast.author,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                )
            }
        }
    }
}

@Composable
private fun EmptyLibrary(modifier: Modifier = Modifier) {
    Box(modifier = modifier, contentAlignment = Alignment.Center) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                text = "No podcasts yet",
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = "Tap + to add your first podcast",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
