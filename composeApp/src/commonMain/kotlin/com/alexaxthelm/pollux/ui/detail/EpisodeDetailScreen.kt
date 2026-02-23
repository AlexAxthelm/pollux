package com.alexaxthelm.pollux.ui.detail

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
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
import com.alexaxthelm.pollux.domain.model.Episode
import com.alexaxthelm.pollux.domain.model.Podcast
import com.alexaxthelm.pollux.presentation.detail.EpisodeDetailState
import com.alexaxthelm.pollux.presentation.detail.EpisodeDetailViewModel
import kotlin.time.Duration
import kotlin.time.Instant
import kotlinx.datetime.Month
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime

class EpisodeDetailScreen(
    private val deps: AppDependencies,
    private val episodeId: String,
) : Screen {
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow
        val viewModel = remember {
            EpisodeDetailViewModel(episodeId, deps.episodeRepo, deps.podcastRepo)
        }
        val state by viewModel.state.collectAsState()
        EpisodeDetailContent(
            state = state,
            onBack = { navigator.pop() },
            onMarkPlayed = { played -> viewModel.markEpisodePlayed(played) },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun EpisodeDetailContent(
    state: EpisodeDetailState,
    onBack: () -> Unit,
    onMarkPlayed: (Boolean) -> Unit,
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    val episodeNumber = (state as? EpisodeDetailState.Loaded)?.episode?.episodeNumber
                    Text(if (episodeNumber != null) "Episode $episodeNumber" else "Episode")
                },
                navigationIcon = {
                    IconButton(onClick = onBack) { Text("←") }
                },
            )
        },
    ) { padding ->
        when (state) {
            is EpisodeDetailState.Loading -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(padding),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator()
                }
            }

            is EpisodeDetailState.Loaded -> {
                EpisodeDetailBody(
                    episode = state.episode,
                    podcast = state.podcast,
                    modifier = Modifier.fillMaxSize().padding(padding),
                    onMarkPlayed = onMarkPlayed,
                )
            }
        }
    }
}

@Composable
private fun EpisodeDetailBody(
    episode: Episode,
    podcast: Podcast,
    modifier: Modifier = Modifier,
    onMarkPlayed: (Boolean) -> Unit,
) {
    Column(
        modifier = modifier.verticalScroll(rememberScrollState()),
    ) {
        // Header
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp),
            verticalAlignment = Alignment.Top,
        ) {
            Box(
                modifier = Modifier
                    .size(96.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(MaterialTheme.colorScheme.surfaceVariant),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = episode.title.take(2).uppercase(),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                Text(
                    text = episode.title,
                    style = MaterialTheme.typography.headlineSmall,
                )
                Text(
                    text = podcast.title,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                if (podcast.author != null) {
                    Text(
                        text = podcast.author,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Spacer(Modifier.height(4.dp))
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
        }

        HorizontalDivider(modifier = Modifier.padding(horizontal = 16.dp))
        Spacer(Modifier.height(16.dp))

        // Action buttons
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            OutlinedButton(
                onClick = { /* Step 1.9 — AudioPlayer not yet implemented */ },
                modifier = Modifier.weight(1f),
                enabled = false,
            ) {
                Text("Play")
            }
            OutlinedButton(
                onClick = { onMarkPlayed(!episode.isPlayed) },
                modifier = Modifier.weight(1f),
            ) {
                Text(if (episode.isPlayed) "Mark Unplayed" else "Mark Played")
            }
        }

        // Description
        if (episode.description != null) {
            val plainDescription = stripHtml(episode.description)
            if (plainDescription.isNotBlank()) {
                Spacer(Modifier.height(24.dp))
                Text(
                    text = "Description",
                    style = MaterialTheme.typography.titleSmall,
                    modifier = Modifier.padding(horizontal = 16.dp),
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    text = plainDescription,
                    style = MaterialTheme.typography.bodyMedium,
                    modifier = Modifier.padding(horizontal = 16.dp),
                )
            }
        }

        Spacer(Modifier.height(32.dp))
    }
}

private fun stripHtml(html: String): String {
    return html
        .replace(Regex("<br\\s*/?>", RegexOption.IGNORE_CASE), "\n")
        .replace(Regex("</?p[^>]*>", RegexOption.IGNORE_CASE), "\n")
        .replace(Regex("<li[^>]*>", RegexOption.IGNORE_CASE), "\n• ")
        .replace(Regex("<[^>]+>"), "")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace(Regex("\n{3,}"), "\n\n")
        .trim()
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
