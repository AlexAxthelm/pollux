package com.alexaxthelm.pollux.ui.subscribe

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
import androidx.compose.foundation.layout.safeContentPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.alexaxthelm.pollux.AppDependencies
import com.alexaxthelm.pollux.domain.feed.ParsedFeed
import com.alexaxthelm.pollux.presentation.subscribe.SubscribeState
import com.alexaxthelm.pollux.presentation.subscribe.SubscribeViewModel

class SubscribeScreen(private val deps: AppDependencies) : Screen {
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow
        val viewModel = remember { SubscribeViewModel(deps.subscribeUseCase) }
        val state by viewModel.state.collectAsState()
        if (state is SubscribeState.Saved) {
            LaunchedEffect(Unit) {
                viewModel.cancelPreview()
                navigator.pop()
            }
        } else {
            SubscribeScreenContent(
                state = state,
                onSubmit = viewModel::submit,
                onConfirm = viewModel::confirmSubscription,
                onCancel = { viewModel.cancelPreview(); navigator.pop() },
                onDismissError = viewModel::dismissError,
            )
        }
    }
}

@Composable
private fun SubscribeScreenContent(
    state: SubscribeState,
    onSubmit: (String) -> Unit,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
    onDismissError: () -> Unit,
    modifier: Modifier = Modifier,
) {
    when (state) {
        is SubscribeState.Idle,
        is SubscribeState.Loading,
        is SubscribeState.Error -> UrlEntryStep(
            state = state,
            onSubmit = onSubmit,
            onCancel = onCancel,
            onDismissError = onDismissError,
            modifier = modifier,
        )

        is SubscribeState.Preview -> PreviewStep(
            feed = state.feed,
            onConfirm = onConfirm,
            onCancel = onCancel,
            modifier = modifier,
        )

        // Saved is handled in Content() via LaunchedEffect — nothing to render here.
        is SubscribeState.Saved -> Unit
    }
}

@Composable
private fun UrlEntryStep(
    state: SubscribeState,
    onSubmit: (String) -> Unit,
    onCancel: () -> Unit,
    onDismissError: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var url by rememberSaveable { mutableStateOf("") }
    val isLoading = state is SubscribeState.Loading
    val errorMessage = (state as? SubscribeState.Error)?.message

    Box(
        modifier = modifier
            .fillMaxSize()
            .safeContentPadding(),
    ) {
        IconButton(
            onClick = onCancel,
            modifier = Modifier.align(Alignment.TopEnd),
        ) {
            Text("✕")
        }
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp)
                .align(Alignment.Center),
            verticalArrangement = Arrangement.spacedBy(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = "Add Podcast",
                style = MaterialTheme.typography.headlineMedium,
            )

            Text(
                text = "Enter the RSS or Atom feed URL for the podcast you want to subscribe to.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            OutlinedTextField(
                value = url,
                onValueChange = {
                    url = it
                    if (errorMessage != null) onDismissError()
                },
                label = { Text("Feed URL") },
                placeholder = {
                    Text(
                        "https://example.com/feed.xml",
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                    )
                },
                isError = errorMessage != null,
                supportingText = errorMessage?.let { { Text(it) } },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )

            Button(
                onClick = { onSubmit(url) },
                enabled = !isLoading,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text("Fetch Feed")
            }

            if (isLoading) {
                Spacer(modifier = Modifier.height(8.dp))
                CircularProgressIndicator()
            }
        }
    }
}

@Composable
private fun PreviewStep(
    feed: ParsedFeed,
    onConfirm: () -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxSize()
            .safeContentPadding()
            .verticalScroll(rememberScrollState())
            .padding(horizontal = 24.dp, vertical = 16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text(
            text = "Subscribe to Podcast?",
            style = MaterialTheme.typography.headlineMedium,
        )

        // Artwork placeholder — real image loading deferred to a later step
        Box(
            modifier = Modifier
                .size(120.dp)
                .clip(RoundedCornerShape(12.dp))
                .background(MaterialTheme.colorScheme.surfaceVariant),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = feed.title.take(2).uppercase(),
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Text(
            text = feed.title,
            style = MaterialTheme.typography.titleLarge,
        )

        if (feed.author != null) {
            Text(
                text = feed.author,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Text(
            text = "${feed.episodes.size} episode${if (feed.episodes.size == 1) "" else "s"} available",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.primary,
        )

        if (!feed.description.isNullOrBlank()) {
            Text(
                text = feed.description,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 5,
                overflow = TextOverflow.Ellipsis,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        Spacer(Modifier.height(8.dp))

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            OutlinedButton(
                onClick = onCancel,
                modifier = Modifier.weight(1f),
            ) {
                Text("Cancel")
            }
            Button(
                onClick = onConfirm,
                modifier = Modifier.weight(1f),
            ) {
                Text("Subscribe")
            }
        }
    }
}
