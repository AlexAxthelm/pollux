package com.alexaxthelm.pollux

import com.alexaxthelm.pollux.domain.repository.EpisodeRepository
import com.alexaxthelm.pollux.domain.repository.PodcastRepository
import com.alexaxthelm.pollux.domain.usecase.SubscribeToPodcastUseCase

data class AppDependencies(
    val podcastRepo: PodcastRepository,
    val episodeRepo: EpisodeRepository,
    val subscribeUseCase: SubscribeToPodcastUseCase,
)
