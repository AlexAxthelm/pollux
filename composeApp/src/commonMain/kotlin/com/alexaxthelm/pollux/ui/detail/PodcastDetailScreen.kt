package com.alexaxthelm.pollux.ui.detail

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.alexaxthelm.pollux.AppDependencies
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.presentation.detail.PodcastDetailState
import com.alexaxthelm.pollux.presentation.detail.PodcastDetailViewModel
import kotlin.math.roundToInt
import kotlin.time.Duration
import kotlin.time.Instant
import kotlinx.coroutines.launch
import kotlinx.datetime.Month
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime

class PodcastDetailScreen(private val deps: AppDependencies, private val podcastId: String) : Screen {
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow
        val viewModel = remember { PodcastDetailViewModel(podcastId, deps.podcastRepo, deps.episodeRepo) }
        val state by viewModel.state.collectAsState()
        PodcastDetailContent(
            state = state,
            onBack = { navigator.pop() },
            onNavigateToEpisode = { id -> navigator.push(EpisodeDetailScreen(deps, id)) },
            onMarkEpisodePlayed = { id, played -> viewModel.markEpisodePlayed(id, played) },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun PodcastDetailContent(
    state: PodcastDetailState,
    onBack: () -> Unit,
    onNavigateToEpisode: (String) -> Unit,
    onMarkEpisodePlayed: (String, Boolean) -> Unit,
) {
    var actionEpisode by remember { mutableStateOf<Episode?>(null) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(if (state is PodcastDetailState.Loaded) state.podcast.title else "Podcast")
                },
                navigationIcon = {
                    IconButton(onClick = onBack) { Text("←") }
                },
            )
        },
    ) { padding ->
        when (state) {
            is PodcastDetailState.Loading -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(padding),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator()
                }
            }

            is PodcastDetailState.Loaded -> {
                LazyColumn(
                    modifier = Modifier.fillMaxSize().padding(padding),
                ) {
                    item { PodcastHeader(state.podcast) }
                    items(state.episodes, key = { it.id }) { episode ->
                        SwipeableEpisodeRow(
                            episode = episode,
                            onRowClick = { onNavigateToEpisode(episode.id) },
                            onMoreClick = { actionEpisode = episode },
                            onMarkPlayed = { played -> onMarkEpisodePlayed(episode.id, played) },
                        )
                    }
                }
            }
        }
    }

    // Action sheet — shown when user taps "…" on an episode row
    val sheetState = rememberModalBottomSheetState()
    val currentActionEpisode = actionEpisode
    if (currentActionEpisode != null) {
        ModalBottomSheet(
            onDismissRequest = { actionEpisode = null },
            sheetState = sheetState,
        ) {
            EpisodeActionSheet(
                episode = currentActionEpisode,
                onDismiss = { actionEpisode = null },
                onEpisodeInfo = {
                    actionEpisode = null
                    onNavigateToEpisode(currentActionEpisode.id)
                },
                onMarkPlayed = {
                    onMarkEpisodePlayed(currentActionEpisode.id, !currentActionEpisode.isPlayed)
                    actionEpisode = null
                },
            )
        }
    }
}

@Composable
private fun SwipeableEpisodeRow(
    episode: Episode,
    onRowClick: () -> Unit,
    onMoreClick: () -> Unit,
    onMarkPlayed: (Boolean) -> Unit,
) {
    val coroutineScope = rememberCoroutineScope()
    val offsetX = remember(episode.id) { Animatable(0f) }
    val density = LocalDensity.current
    val swipeThresholdPx = remember(density) { with(density) { 80.dp.toPx() } }

    Box(modifier = Modifier.fillMaxWidth()) {
        // Reveal layer shown behind the row as the user swipes right
        Box(
            modifier = Modifier
                .matchParentSize()
                .background(
                    if (!episode.isPlayed) MaterialTheme.colorScheme.primary
                    else MaterialTheme.colorScheme.surfaceVariant,
                ),
            contentAlignment = Alignment.CenterStart,
        ) {
            Text(
                text = if (!episode.isPlayed) "Mark Played" else "Mark Unplayed",
                modifier = Modifier.padding(start = 16.dp),
                style = MaterialTheme.typography.labelMedium,
                color = if (!episode.isPlayed) MaterialTheme.colorScheme.onPrimary
                else MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        // Episode row content, shifted right during the drag
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .offset { IntOffset(offsetX.value.roundToInt(), 0) }
                .clickable(onClick = onRowClick)
                .pointerInput(episode.id) {
                    detectHorizontalDragGestures(
                        onDragStart = {
                            coroutineScope.launch { offsetX.snapTo(0f) }
                        },
                        onDragEnd = {
                            coroutineScope.launch {
                                if (offsetX.value >= swipeThresholdPx) {
                                    onMarkPlayed(!episode.isPlayed)
                                }
                                offsetX.animateTo(0f, spring(stiffness = Spring.StiffnessMedium))
                            }
                        },
                        onDragCancel = {
                            coroutineScope.launch {
                                offsetX.animateTo(0f, spring(stiffness = Spring.StiffnessMedium))
                            }
                        },
                        onHorizontalDrag = { change, dragAmount ->
                            change.consume()
                            coroutineScope.launch {
                                // Only allow swiping right; cap at 1.5× threshold for rubber-band feel
                                offsetX.snapTo(
                                    (offsetX.value + dragAmount).coerceIn(0f, swipeThresholdPx * 1.5f),
                                )
                            }
                        },
                    )
                },
        ) {
            EpisodeRow(episode = episode, onMoreClick = onMoreClick)
        }
    }
}

@Composable
private fun EpisodeRow(episode: Episode, onMoreClick: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.surface)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .size(56.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = episode.title.take(2).uppercase(),
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Text(
                text = episode.title,
                style = MaterialTheme.typography.bodyLarge,
                maxLines = 2,
            )
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "${formatDate(episode.publishDate)}  ·  ${formatDuration(episode.duration)}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                if (episode.isPlayed) {
                    Text(
                        text = "PLAYED",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
        IconButton(onClick = onMoreClick) {
            Text("…")
        }
    }
}

@Composable
private fun PodcastHeader(podcast: Podcast) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Box(
            modifier = Modifier
                .size(96.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = podcast.title.take(2).uppercase(),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Text(
            text = podcast.title,
            style = MaterialTheme.typography.headlineMedium,
        )
        if (podcast.author != null) {
            Text(
                text = podcast.author,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        if (podcast.description != null) {
            Text(
                text = podcast.description,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 5,
            )
        }
    }
}

@Composable
private fun EpisodeActionSheet(
    episode: Episode,
    onDismiss: () -> Unit,
    onEpisodeInfo: () -> Unit,
    onMarkPlayed: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 32.dp),
    ) {
        // Episode header inside the sheet
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(RoundedCornerShape(6.dp))
                    .background(MaterialTheme.colorScheme.surfaceVariant),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = episode.title.take(2).uppercase(),
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = episode.title,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 2,
                )
                Text(
                    text = formatDuration(episode.duration),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        HorizontalDivider()

        ActionItem(label = "Play", enabled = false, onClick = {})
        ActionItem(label = "Download", enabled = false, onClick = {})
        ActionItem(
            label = if (episode.isPlayed) "Mark Unplayed" else "Mark Played",
            onClick = onMarkPlayed,
        )
        ActionItem(label = "Episode Info", onClick = onEpisodeInfo)
        ActionItem(label = "Share", onClick = onDismiss)
    }
}

@Composable
private fun ActionItem(label: String, enabled: Boolean = true, onClick: () -> Unit) {
    Text(
        text = label,
        modifier = Modifier
            .fillMaxWidth()
            .then(if (enabled) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(horizontal = 16.dp, vertical = 14.dp),
        style = MaterialTheme.typography.bodyLarge,
        color = if (enabled) MaterialTheme.colorScheme.onSurface
        else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f),
    )
}

private fun formatDate(instant: Instant): String {
    val local = instant.toLocalDateTime(TimeZone.currentSystemDefault())
    val monthShort = when (local.month) {
        Month.JANUARY -> "Jan"; Month.FEBRUARY -> "Feb"; Month.MARCH -> "Mar"
        Month.APRIL -> "Apr"; Month.MAY -> "May"; Month.JUNE -> "Jun"
        Month.JULY -> "Jul"; Month.AUGUST -> "Aug"; Month.SEPTEMBER -> "Sep"
        Month.OCTOBER -> "Oct"; Month.NOVEMBER -> "Nov"; else -> "Dec"
    }
    return "$monthShort ${local.day}, ${local.year}"
}

private fun formatDuration(duration: Duration): String {
    val totalSeconds = duration.inWholeSeconds
    val hours = totalSeconds / 3600
    val minutes = (totalSeconds % 3600) / 60
    return if (hours > 0L) "${hours}h ${minutes}m" else "${minutes}m"
}
