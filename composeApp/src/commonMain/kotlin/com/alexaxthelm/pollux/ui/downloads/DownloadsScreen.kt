package com.alexaxthelm.pollux.ui.downloads

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
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
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.alexaxthelm.pollux.AppDependencies
import com.alexaxthelm.pollux.domain.download.DownloadItem
import com.alexaxthelm.pollux.domain.download.DownloadStatus
import com.alexaxthelm.pollux.presentation.downloads.DownloadsState
import com.alexaxthelm.pollux.presentation.downloads.DownloadsViewModel

class DownloadsScreen(private val deps: AppDependencies) : Screen {
    @OptIn(ExperimentalMaterial3Api::class)
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow
        val viewModel = remember { DownloadsViewModel(deps.downloadManager) }
        val state by viewModel.state.collectAsState()
        DownloadsContent(
            state = state,
            onBack = { navigator.pop() },
            onCancel = viewModel::cancel,
            onRetry = viewModel::retry,
            onRemove = viewModel::remove,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DownloadsContent(
    state: DownloadsState,
    onBack: () -> Unit,
    onCancel: (String) -> Unit,
    onRetry: (String) -> Unit,
    onRemove: (String) -> Unit,
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Downloads") },
                navigationIcon = {
                    IconButton(onClick = onBack) { Text("←") }
                },
            )
        },
    ) { padding ->
        when (state) {
            is DownloadsState.Empty -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(padding),
                    contentAlignment = Alignment.Center,
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Text(
                            text = "No downloads",
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Text(
                            text = "Download episodes from the episode action menu",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }

            is DownloadsState.Loaded -> {
                LazyColumn(
                    modifier = Modifier.fillMaxSize().padding(padding),
                    verticalArrangement = Arrangement.spacedBy(0.dp),
                ) {
                    // Active download
                    state.active?.let { item ->
                        item { SectionHeader("Downloading") }
                        item { ActiveDownloadRow(item = item, onCancel = { onCancel(item.episode.id) }) }
                        item { HorizontalDivider() }
                    }

                    // Queued downloads
                    if (state.queued.isNotEmpty()) {
                        item { SectionHeader("Queued") }
                        items(state.queued, key = { it.episode.id }) { item ->
                            QueuedDownloadRow(item = item, onCancel = { onCancel(item.episode.id) })
                        }
                        item { HorizontalDivider() }
                    }

                    // Failed downloads
                    if (state.failed.isNotEmpty()) {
                        item { SectionHeader("Failed") }
                        items(state.failed, key = { it.episode.id }) { item ->
                            FailedDownloadRow(
                                item = item,
                                onRetry = { onRetry(item.episode.id) },
                                onRemove = { onRemove(item.episode.id) },
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun SectionHeader(title: String) {
    Text(
        text = title,
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp),
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun ActiveDownloadRow(item: DownloadItem, onCancel: () -> Unit) {
    val status = item.status as? DownloadStatus.InProgress
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = item.episode.title,
                modifier = Modifier.weight(1f).padding(end = 8.dp),
                style = MaterialTheme.typography.bodyLarge,
                maxLines = 2,
            )
            OutlinedButton(onClick = onCancel) {
                Text("Cancel")
            }
        }
        if (status != null && status.totalBytes != null && status.totalBytes > 0) {
            val progress = status.bytesDownloaded.toFloat() / status.totalBytes.toFloat()
            LinearProgressIndicator(
                progress = { progress },
                modifier = Modifier.fillMaxWidth(),
            )
            Text(
                text = "${formatBytes(status.bytesDownloaded)} / ${formatBytes(status.totalBytes)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            Text(
                text = if (status != null) formatBytes(status.bytesDownloaded) else "Starting…",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

@Composable
private fun QueuedDownloadRow(item: DownloadItem, onCancel: () -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f).padding(end = 8.dp)) {
            Text(
                text = item.episode.title,
                style = MaterialTheme.typography.bodyLarge,
                maxLines = 2,
            )
            Text(
                text = "Waiting…",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        OutlinedButton(onClick = onCancel) {
            Text("Cancel")
        }
    }
}

@Composable
private fun FailedDownloadRow(item: DownloadItem, onRetry: () -> Unit, onRemove: () -> Unit) {
    val error = (item.status as? DownloadStatus.Failed)?.message ?: "Unknown error"
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Text(
            text = item.episode.title,
            style = MaterialTheme.typography.bodyLarge,
            maxLines = 2,
        )
        Text(
            text = error,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.error,
            maxLines = 1,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = onRetry) {
                Text("Retry")
            }
            OutlinedButton(onClick = onRemove) {
                Text("Remove")
            }
        }
    }
}

private fun formatBytes(bytes: Long): String {
    return when {
        bytes < 1024 -> "$bytes B"
        bytes < 1024 * 1024 -> "${bytes / 1024} KB"
        else -> "${"%.1f".format(bytes.toDouble() / (1024 * 1024))} MB"
    }
}
