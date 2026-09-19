use crux_core::{render::render, App, Command};
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::capabilities::download::{DownloadOperation, DownloadResult};
use crate::capabilities::http::{HttpOperation, HttpResult};
use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::domain::{DownloadStatus, Episode, EpisodeSortOrder};
use crate::effect::Effect;
use crate::feed_parser::parse_feed;
use crate::html::strip_html_preview;
use crate::model::{DownloadProgress, Model, QueuedDownload};
use crate::view_model::{
    EpisodeSummary, LibraryView, SubscriptionDetailView, SubscriptionSummary, ViewModel,
};

#[derive(Default)]
pub struct Pollux;

impl App for Pollux {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match event {
            Event::Started => {
                model.loading = true;
                let subscriptions =
                    Command::request_from_shell(StorageOperation::ListSubscriptions)
                        .then_send(|r| Event::SubscriptionsLoaded(Box::new(r)));
                // Rebuild the download queue for anything a previous session left
                // mid-download, and resume it.
                let pending = Command::request_from_shell(StorageOperation::ListPendingDownloads)
                    .then_send(|r| Event::PendingDownloadsLoaded(Box::new(r)));
                subscriptions.and(pending).and(render())
            }
            Event::SubscriptionsLoaded(result) => {
                model.loading = false;
                match *result {
                    StorageResult::Subscriptions(rows) => {
                        model.subscriptions = rows;
                        model.error = None;
                    }
                    StorageResult::Error(e) => model.error = Some(e),
                    unexpected => {
                        model.error = Some(format!("unexpected storage result: {unexpected:?}"))
                    }
                }
                render()
            }
            Event::PendingDownloadsLoaded(result) => {
                // Rebuild the queue from downloads a prior session left in flight and
                // resume them. Partial files don't survive a quit, so each restarts
                // from scratch. Best-effort: an error here just leaves nothing queued.
                let mut cmd = Command::done();
                if let StorageResult::Episodes(rows) = *result {
                    for episode in rows {
                        let already = model.downloading.as_deref() == Some(episode.id.as_str())
                            || model
                                .download_queue
                                .iter()
                                .any(|q| q.episode_id == episode.id);
                        if already {
                            continue;
                        }
                        // Normalize to Queued so an item waiting behind others no
                        // longer reads as "Downloading" in the DB.
                        cmd = cmd.and(persist_download_state(
                            &episode.id,
                            DownloadStatus::Queued,
                            None,
                            None,
                        ));
                        model.download_queue.push(QueuedDownload {
                            episode_id: episode.id,
                            url: episode.enclosure_url,
                        });
                    }
                }
                cmd.and(maybe_start_next(model)).and(render())
            }
            Event::FetchFeed(url) => {
                model.loading = true;
                model.error = None;
                Command::request_from_shell(HttpOperation::FetchFeed { url: url.clone() })
                    .then_send(move |r| Event::FeedFetched {
                        url,
                        result: Box::new(r),
                    })
                    .and(render())
            }
            Event::FeedFetched { url, result } => match *result {
                HttpResult::Error(e) => {
                    model.loading = false;
                    model.error = Some(e);
                    render()
                }
                HttpResult::Response { status: 200, body } => match parse_feed(&url, body) {
                    Ok((subscription, episodes)) => {
                        Command::request_from_shell(StorageOperation::UpsertFeedWithEpisodes {
                            subscription,
                            episodes,
                        })
                        .then_send(|r| Event::FeedSaved(Box::new(r)))
                    }
                    Err(e) => {
                        model.loading = false;
                        model.error = Some(e);
                        render()
                    }
                },
                HttpResult::Response { status, .. } => {
                    model.loading = false;
                    model.error = Some(format!("feed fetch failed: HTTP {status}"));
                    render()
                }
            },
            Event::FeedSaved(result) => {
                model.loading = false;
                match *result {
                    StorageResult::Subscription(sub) => {
                        if let Some(pos) = model.subscriptions.iter().position(|s| s.id == sub.id) {
                            model.subscriptions[pos] = sub;
                        } else {
                            model.subscriptions.push(sub);
                        }
                        model.error = None;
                    }
                    StorageResult::Error(e) => model.error = Some(e),
                    unexpected => {
                        model.error = Some(format!("unexpected storage result: {unexpected:?}"))
                    }
                }
                render()
            }
            Event::SelectSubscription(id) => {
                // Re-selecting the feed already on screen (e.g. returning from an
                // episode detail, which re-fires the shell's selection task) must
                // not reload it: that would clear the list and flash a spinner for
                // no new data. Reload only when the selection changes, or when the
                // previous load errored and should be retried.
                //
                // NOTE: this makes re-selection a no-op even if the feed's stored
                // episodes changed since the last load. That is fine today because
                // nothing refreshes an existing feed's episodes, but when a feed
                // refresh/refetch lands it must trigger an explicit reload event
                // rather than relying on re-selection. See docs/features/subscription.md.
                let already_showing = model
                    .selected_subscription
                    .as_ref()
                    .is_some_and(|s| s.id == id)
                    && model.detail_error.is_none();
                if already_showing {
                    render()
                } else {
                    // Remember the chosen feed (for the details header) from the
                    // already-loaded library, so the page needs no extra fetch.
                    model.selected_subscription =
                        model.subscriptions.iter().find(|s| s.id == id).cloned();
                    model.episodes.clear();
                    model.detail_loading = true;
                    model.detail_error = None;
                    Command::request_from_shell(StorageOperation::ListEpisodesBySubscription {
                        subscription_id: id.clone(),
                    })
                    .then_send(move |r| Event::EpisodesLoaded {
                        subscription_id: id,
                        result: Box::new(r),
                    })
                    .and(render())
                }
            }
            Event::EpisodesLoaded {
                subscription_id,
                result,
            } => {
                // Drop a response for a feed we're no longer showing: a load left
                // in flight by a previous selection must not overwrite the current
                // one. The requested id is carried on the event rather than inferred
                // from the rows, so an empty result still correlates correctly.
                if model
                    .selected_subscription
                    .as_ref()
                    .is_some_and(|s| s.id == subscription_id)
                {
                    model.detail_loading = false;
                    match *result {
                        StorageResult::Episodes(rows) => {
                            model.episodes = rows;
                            model.detail_error = None;
                        }
                        StorageResult::Error(e) => model.detail_error = Some(e),
                        unexpected => {
                            model.detail_error =
                                Some(format!("unexpected storage result: {unexpected:?}"))
                        }
                    }
                }
                render()
            }
            Event::SetEpisodeSort(order) => {
                // The sort is applied in view(), so this only records the choice
                // and re-renders — no storage round-trip needed.
                model.episode_sort = order;
                render()
            }
            Event::DownloadEpisode(episode_id) => {
                // Only a not-downloaded or failed (retry) episode can start a
                // download. Every other state — already queued/downloading, already
                // downloaded, removed from feed, or an episode we don't have loaded —
                // is a no-op, so a stray or repeated event can't double-enqueue or
                // re-download a file we already have.
                let found = model
                    .episodes
                    .iter()
                    .find(|e| e.id == episode_id)
                    .map(|e| (e.download_status.clone(), e.enclosure_url.clone()));
                match found {
                    Some((DownloadStatus::NotDownloaded, url))
                    | Some((DownloadStatus::Failed, url)) => {
                        if !download_allowed(model) {
                            // Fail-on-full: unlimited today, so this is unreachable.
                            // When a storage cap exists this is where a download is
                            // refused up front and marked Failed instead of started.
                            set_download_state(
                                model,
                                &episode_id,
                                DownloadStatus::Failed,
                                None,
                                None,
                            );
                            persist_download_state(&episode_id, DownloadStatus::Failed, None, None)
                                .and(render())
                        } else {
                            set_download_state(
                                model,
                                &episode_id,
                                DownloadStatus::Queued,
                                None,
                                None,
                            );
                            model.download_queue.push(QueuedDownload {
                                episode_id: episode_id.clone(),
                                url,
                            });
                            let persist = persist_download_state(
                                &episode_id,
                                DownloadStatus::Queued,
                                None,
                                None,
                            );
                            persist.and(maybe_start_next(model)).and(render())
                        }
                    }
                    _ => render(),
                }
            }
            Event::DownloadProgress {
                episode_id,
                received_bytes,
                total_bytes,
            } => {
                // Track only the active download's progress; drop a late or stale
                // report for an episode that already finished or was superseded.
                if model.downloading.as_deref() == Some(episode_id.as_str()) {
                    model.active_download_progress = Some(DownloadProgress {
                        episode_id,
                        received_bytes,
                        total_bytes,
                    });
                    render()
                } else {
                    Command::done()
                }
            }
            Event::DownloadFinished { episode_id, result } => match *result {
                DownloadResult::Completed {
                    local_path,
                    size_bytes,
                } => {
                    finish_active(model, &episode_id);
                    set_download_state(
                        model,
                        &episode_id,
                        DownloadStatus::Downloaded,
                        Some(local_path.clone()),
                        Some(size_bytes),
                    );
                    let persist = persist_download_state(
                        &episode_id,
                        DownloadStatus::Downloaded,
                        Some(local_path),
                        Some(size_bytes),
                    );
                    persist.and(maybe_start_next(model)).and(render())
                }
                DownloadResult::Error(_) => {
                    finish_active(model, &episode_id);
                    set_download_state(model, &episode_id, DownloadStatus::Failed, None, None);
                    let persist =
                        persist_download_state(&episode_id, DownloadStatus::Failed, None, None);
                    persist.and(maybe_start_next(model)).and(render())
                }
                DownloadResult::Cancelled => {
                    // The shell cancelled the task and flushed the partial file, so
                    // the episode goes back to not-downloaded and the queue drains on.
                    finish_active(model, &episode_id);
                    set_download_state(
                        model,
                        &episode_id,
                        DownloadStatus::NotDownloaded,
                        None,
                        None,
                    );
                    let persist = persist_download_state(
                        &episode_id,
                        DownloadStatus::NotDownloaded,
                        None,
                        None,
                    );
                    persist.and(maybe_start_next(model)).and(render())
                }
                // Deleted never arrives here (it resolves DeleteDownload instead).
                DownloadResult::Deleted => render(),
            },
            Event::CancelDownload(episode_id) => {
                if model.downloading.as_deref() == Some(episode_id.as_str()) {
                    // Active download: ask the shell to cancel the task and flush the
                    // partial file. The in-flight Download request then resolves as
                    // Cancelled (handled above), which finalizes the state.
                    Command::request_from_shell(DownloadOperation::Cancel {
                        episode_id: episode_id.clone(),
                    })
                    .then_send(|r| Event::DownloadCanceled(Box::new(r)))
                    .and(render())
                } else if let Some(pos) = model
                    .download_queue
                    .iter()
                    .position(|q| q.episode_id == episode_id)
                {
                    // Queued but not started: just drop it from the queue and reset —
                    // nothing is running and no file exists yet.
                    model.download_queue.remove(pos);
                    set_download_state(
                        model,
                        &episode_id,
                        DownloadStatus::NotDownloaded,
                        None,
                        None,
                    );
                    persist_download_state(&episode_id, DownloadStatus::NotDownloaded, None, None)
                        .and(render())
                } else {
                    // Not downloading and not queued — nothing to cancel.
                    render()
                }
            }
            Event::DownloadCanceled(result) => {
                // Best-effort: the reset rides on the Download request's Cancelled
                // resolution; only surface a failure to cancel here.
                if let DownloadResult::Error(e) = *result {
                    model.detail_error = Some(e);
                }
                render()
            }
            Event::DeleteDownload(episode_id) => {
                let local_path = model
                    .episodes
                    .iter()
                    .find(|e| e.id == episode_id)
                    .and_then(|e| e.local_path.clone());
                match local_path {
                    Some(path) => {
                        let id = episode_id.clone();
                        Command::request_from_shell(DownloadOperation::Delete { local_path: path })
                            .then_send(move |r| Event::DownloadDeleted {
                                episode_id: id.clone(),
                                result: Box::new(r),
                            })
                            .and(render())
                    }
                    None => {
                        // Nothing on disk (e.g. a Failed row): normalize to
                        // NotDownloaded without a filesystem round-trip.
                        set_download_state(
                            model,
                            &episode_id,
                            DownloadStatus::NotDownloaded,
                            None,
                            None,
                        );
                        persist_download_state(
                            &episode_id,
                            DownloadStatus::NotDownloaded,
                            None,
                            None,
                        )
                        .and(render())
                    }
                }
            }
            Event::DownloadDeleted { episode_id, result } => match *result {
                DownloadResult::Deleted => {
                    set_download_state(
                        model,
                        &episode_id,
                        DownloadStatus::NotDownloaded,
                        None,
                        None,
                    );
                    persist_download_state(&episode_id, DownloadStatus::NotDownloaded, None, None)
                        .and(render())
                }
                DownloadResult::Error(e) => {
                    // The file couldn't be removed; leave the row as-is and surface
                    // why on the details page rather than lying about the state.
                    model.detail_error = Some(e);
                    render()
                }
                // Only Deleted/Error arrive from a Delete request; the rest are
                // unreachable but keep the match total.
                DownloadResult::Completed { .. } | DownloadResult::Cancelled => render(),
            },
            Event::DownloadStatePersisted(result) => {
                if let StorageResult::Error(e) = *result {
                    model.detail_error = Some(e);
                }
                render()
            }
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        // Display order is owned here rather than at each mutation site, so the
        // core does not depend on the shell returning rows in any given order.
        let mut subscriptions: Vec<SubscriptionSummary> = model
            .subscriptions
            .iter()
            .map(|s| SubscriptionSummary {
                id: s.id.clone(),
                title: s.title.clone(),
                artwork_url: s.artwork_url.clone(),
            })
            .collect();
        subscriptions.sort_by_cached_key(|s| s.title.to_lowercase());

        ViewModel {
            library: LibraryView {
                subscriptions,
                loading: model.loading,
                error: model.error.clone(),
            },
            subscription_detail: build_subscription_detail(model),
        }
    }
}

/// Builds the details-page view from the selected subscription and its episodes.
/// Display order is owned here (like `subscriptions` above) so the core stays the
/// single source of truth for sort order, independent of how the shell's SQL
/// returned the rows.
fn build_subscription_detail(model: &Model) -> SubscriptionDetailView {
    let mut episodes: Vec<EpisodeSummary> = model.episodes.iter().map(episode_summary).collect();
    sort_episodes(&mut episodes, model.episode_sort);

    // Overlay the in-flight download's live byte progress onto its row. Progress is
    // transient runtime state (not stored on the Episode), so it is injected here at
    // projection time for the one episode currently downloading.
    if let Some(progress) = model.active_download_progress.as_ref() {
        if let Some(summary) = episodes.iter_mut().find(|e| e.id == progress.episode_id) {
            summary.download_received_bytes = Some(progress.received_bytes);
            summary.download_total_bytes = progress.total_bytes;
        }
    }

    let (subscription_id, title, artwork_url) = match &model.selected_subscription {
        Some(s) => (Some(s.id.clone()), s.title.clone(), s.artwork_url.clone()),
        None => (None, String::new(), None),
    };

    SubscriptionDetailView {
        subscription_id,
        title,
        artwork_url,
        episodes,
        sort_order: model.episode_sort,
        loading: model.detail_loading,
        error: model.detail_error.clone(),
    }
}

/// Number of characters of stripped description shipped for the row preview. A row
/// shows a single line, so this is well above what can be displayed; the rest is
/// never processed (see `strip_html_preview`).
const DESCRIPTION_PREVIEW_CHARS: usize = 200;

/// Projects a stored `Episode` into its display `EpisodeSummary`, stripping a short
/// plain-text preview of the description for the row while keeping the raw HTML for
/// the detail page. The preview is bounded, so this is cheap to run per render.
fn episode_summary(e: &Episode) -> EpisodeSummary {
    EpisodeSummary {
        id: e.id.clone(),
        title: e.title.clone(),
        description: e.description.clone(),
        description_text: e
            .description
            .as_deref()
            .map(|d| strip_html_preview(d, DESCRIPTION_PREVIEW_CHARS))
            .filter(|s| !s.is_empty()),
        pub_date: e.pub_date,
        duration_secs: e.duration_secs,
        artwork_url: e.artwork_url.clone(),
        playback_status: e.playback_status.clone(),
        playback_position_secs: e.playback_position_secs,
        download_status: e.download_status.clone(),
        // Live progress is overlaid in `build_subscription_detail` for the active
        // download only; every other row ships without it.
        download_received_bytes: None,
        download_total_bytes: None,
    }
}

/// Sorts episodes in place for display. For both date orders, episodes with no
/// `pub_date` sort last so undated items never crowd out the meaningful ordering.
fn sort_episodes(episodes: &mut [EpisodeSummary], order: EpisodeSortOrder) {
    match order {
        EpisodeSortOrder::PubDateDesc => {
            // Missing dates last, then newest first. `Reverse` gives the descending
            // order without negating (which would overflow at i64::MIN).
            episodes.sort_by_key(|e| (e.pub_date.is_none(), std::cmp::Reverse(e.pub_date)));
        }
        EpisodeSortOrder::PubDateAsc => {
            episodes.sort_by_key(|e| (e.pub_date.is_none(), e.pub_date));
        }
        EpisodeSortOrder::TitleAsc => {
            episodes.sort_by_cached_key(|e| e.title.to_lowercase());
        }
    }
}

/// Pre-download storage check. Unlimited today — the user-configurable soft cap
/// arrives with the Settings screen (see ROADMAP follow-ups). Device-full is not
/// predicted here; it surfaces as the shell's write failing (`DownloadResult::Error`
/// → `Failed`), which is the fail-on-full behavior the storage spec calls for.
fn download_allowed(_model: &Model) -> bool {
    true
}

/// Applies a download-state transition to the in-memory episode, if it's loaded.
///
/// `SelectSubscription` is intentionally idempotent (it won't reload the feed
/// already on screen), so a status change written only to storage would not show
/// up on the details page. Mutating `model.episodes` in place is what lets `view()`
/// reproject the new state without a reload — see the note at `SelectSubscription`.
/// `local_path`/`size_bytes`/`progress` are set (or cleared) together with the
/// status so the row never keeps a path for a file that isn't there.
fn set_download_state(
    model: &mut Model,
    episode_id: &str,
    status: DownloadStatus,
    local_path: Option<String>,
    size_bytes: Option<u64>,
) {
    if let Some(episode) = model.episodes.iter_mut().find(|e| e.id == episode_id) {
        episode.download_status = status;
        episode.local_path = local_path;
        episode.file_size_bytes = size_bytes;
        // No live percentage in the status-only pass; keep the column NULL rather
        // than leaving a stale value from a previous download.
        episode.download_progress = None;
    }
}

/// Persists a download-state transition through the storage capability. The result
/// is checked for errors (via `DownloadStatePersisted`) but success is silent — the
/// UI already reflects the change from `set_download_state`, so persistence is only
/// about durability across restarts.
fn persist_download_state(
    episode_id: &str,
    status: DownloadStatus,
    local_path: Option<String>,
    size_bytes: Option<u64>,
) -> Command<Effect, Event> {
    Command::request_from_shell(StorageOperation::UpdateDownloadState {
        episode_id: episode_id.to_string(),
        status,
        local_path,
        size_bytes,
        progress: None,
    })
    .then_send(|r| Event::DownloadStatePersisted(Box::new(r)))
}

/// Clears the in-flight marker (and its live progress) when the active download
/// reaches a terminal state.
fn finish_active(model: &mut Model, episode_id: &str) {
    if model.downloading.as_deref() == Some(episode_id) {
        model.downloading = None;
        model.active_download_progress = None;
    }
}

/// Starts the next queued download if nothing is in flight (serial: one at a time).
/// Returns the command that marks the episode `Downloading`, persists that, and
/// issues the shell download whose result comes back as `DownloadFinished`. A no-op
/// (empty command) when a download is already running or the queue is empty.
fn maybe_start_next(model: &mut Model) -> Command<Effect, Event> {
    if model.downloading.is_some() {
        return Command::done();
    }
    if model.download_queue.is_empty() {
        return Command::done();
    }
    // The queue entry carries the URL, so a download can start without its feed's
    // episodes being loaded (e.g. a resume at launch).
    let QueuedDownload { episode_id, url } = model.download_queue.remove(0);

    model.downloading = Some(episode_id.clone());
    // A fresh download starts with no progress until the shell reports the first bytes.
    model.active_download_progress = None;
    set_download_state(model, &episode_id, DownloadStatus::Downloading, None, None);

    let persist = persist_download_state(&episode_id, DownloadStatus::Downloading, None, None);
    let download = Command::request_from_shell(DownloadOperation::Download {
        episode_id: episode_id.clone(),
        url,
    })
    .then_send(move |r| Event::DownloadFinished {
        episode_id: episode_id.clone(),
        result: Box::new(r),
    });
    persist.and(download)
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Started,
    SubscriptionsLoaded(Box<StorageResult>),
    /// Episodes left `Downloading`/`Queued` by a previous session, loaded at launch
    /// so their downloads can be re-enqueued and resumed.
    PendingDownloadsLoaded(Box<StorageResult>),
    FetchFeed(String),
    FeedFetched {
        url: String,
        result: Box<HttpResult>,
    },
    FeedSaved(Box<StorageResult>),
    SelectSubscription(String),
    EpisodesLoaded {
        subscription_id: String,
        result: Box<StorageResult>,
    },
    SetEpisodeSort(EpisodeSortOrder),
    /// Queue an episode's audio for download (or retry a failed one).
    DownloadEpisode(String),
    /// Remove a downloaded (or failed) episode's local file.
    DeleteDownload(String),
    /// Stop an in-flight or queued download and reset the episode to not-downloaded.
    CancelDownload(String),
    /// Result of asking the shell to cancel an in-flight download. The state reset
    /// rides on the `Download` request resolving `Cancelled`; this only surfaces a
    /// cancel error.
    DownloadCanceled(Box<DownloadResult>),
    /// Live byte progress for the in-flight download, reported by the shell while
    /// the one-shot download request is still running. Transient — not persisted.
    DownloadProgress {
        episode_id: String,
        received_bytes: u64,
        total_bytes: Option<u64>,
    },
    /// Terminal result of a shell download for `episode_id`.
    DownloadFinished {
        episode_id: String,
        result: Box<DownloadResult>,
    },
    /// Terminal result of a shell delete for `episode_id`.
    DownloadDeleted {
        episode_id: String,
        result: Box<DownloadResult>,
    },
    /// Result of persisting a download-state transition. Success is silent; an
    /// error is surfaced on the details page (downloads are driven from there).
    DownloadStatePersisted(Box<StorageResult>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DownloadStatus, Episode, PlaybackStatus, Subscription};
    use crate::effect::Effect;

    fn make_subscription(id: &str, title: &str) -> Subscription {
        Subscription {
            id: id.to_string(),
            feed_url: format!("https://example.com/{id}.rss"),
            title: title.to_string(),
            artwork_url: None,
            description: None,
            last_refreshed: None,
            created_at: 0,
        }
    }

    fn make_episode(id: &str, title: &str, pub_date: Option<i64>) -> Episode {
        Episode {
            id: id.to_string(),
            feed_guid: format!("{id}-guid"),
            subscription_id: "sub-id".to_string(),
            title: title.to_string(),
            description: None,
            pub_date,
            duration_secs: None,
            enclosure_url: format!("https://example.com/{id}.mp3"),
            artwork_url: None,
            playback_status: PlaybackStatus::Unplayed,
            playback_position_secs: None,
            download_status: DownloadStatus::NotDownloaded,
            download_progress: None,
            is_flagged: false,
            file_size_bytes: None,
            local_path: None,
        }
    }

    fn detail_titles(app: &Pollux, model: &Model) -> Vec<String> {
        app.view(model)
            .subscription_detail
            .episodes
            .into_iter()
            .map(|e| e.title)
            .collect()
    }

    /// Loads episodes through the real event, so the at-load projection (including
    /// HTML stripping) is exercised — the shell never sets summaries directly. A
    /// matching subscription is selected first, since the handler now correlates
    /// the response to the current selection.
    fn load_episodes(app: &Pollux, model: &mut Model, episodes: Vec<Episode>) {
        let sub_id = episodes
            .first()
            .map(|e| e.subscription_id.clone())
            .unwrap_or_else(|| "sub-id".to_string());
        model.selected_subscription = Some(make_subscription(&sub_id, "Test Feed"));
        let _ = app.update(
            Event::EpisodesLoaded {
                subscription_id: sub_id,
                result: Box::new(StorageResult::Episodes(episodes)),
            },
            model,
        );
    }

    fn view_titles(app: &Pollux, model: &Model) -> Vec<String> {
        app.view(model)
            .library
            .subscriptions
            .into_iter()
            .map(|s| s.title)
            .collect()
    }

    const MINIMAL_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
<channel>
  <title>Test Podcast</title>
  <description>A test feed</description>
  <link>https://example.com</link>
  <item>
    <title>Episode 1</title>
    <guid>episode-1-guid</guid>
    <pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate>
    <itunes:duration>3600</itunes:duration>
    <enclosure url="https://example.com/ep1.mp3" type="audio/mpeg" length="12345678"/>
  </item>
</channel>
</rss>"#;

    #[test]
    fn init_sets_loading_and_issues_storage_request() {
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Started, &mut model);

        assert!(model.loading);

        let effects: Vec<Effect> = cmd.effects().collect();
        // ListSubscriptions + ListPendingDownloads + render.
        assert_eq!(effects.len(), 3);

        let has_storage = effects
            .iter()
            .any(|e| matches!(e, Effect::Storage(r) if matches!(r.operation, StorageOperation::ListSubscriptions)));
        assert!(has_storage, "expected a ListSubscriptions storage effect");

        let has_pending = effects
            .iter()
            .any(|e| matches!(e, Effect::Storage(r) if matches!(r.operation, StorageOperation::ListPendingDownloads)));
        assert!(
            has_pending,
            "expected a ListPendingDownloads storage effect to resume interrupted downloads"
        );

        let has_render = effects.iter().any(|e| matches!(e, Effect::Render(_)));
        assert!(has_render, "expected a render effect");
    }

    #[test]
    fn subscriptions_loaded_updates_view() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let subs = vec![
            make_subscription("id-1", "Podcast One"),
            make_subscription("id-2", "Podcast Two"),
        ];
        let mut cmd = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(subs))),
            &mut model,
        );

        assert!(!model.loading);
        assert!(
            model.error.is_none(),
            "successful load should clear previous error"
        );
        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert_eq!(view.library.subscriptions.len(), 2);
        assert_eq!(view.library.subscriptions[0].id, "id-1");
        assert_eq!(view.library.subscriptions[0].title, "Podcast One");
        assert_eq!(view.library.subscriptions[1].id, "id-2");
    }

    #[test]
    fn empty_subscriptions_loaded_shows_empty_state() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![]))),
            &mut model,
        );

        assert!(!model.loading);
        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert!(view.library.subscriptions.is_empty());
        assert!(!view.library.loading);
        assert!(view.library.error.is_none());
    }

    #[test]
    fn storage_error_sets_error_state() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Error(
                "db unavailable".to_string(),
            ))),
            &mut model,
        );

        assert!(!model.loading);
        assert_eq!(model.error.as_deref(), Some("db unavailable"));
    }

    #[test]
    fn successful_load_after_error_clears_error() {
        let app = Pollux;
        let mut model = Model::default();

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Error(
                "transient failure".to_string(),
            ))),
            &mut model,
        );
        assert!(model.error.is_some());

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![
                make_subscription("id-1", "Recovered"),
            ]))),
            &mut model,
        );

        assert!(
            model.error.is_none(),
            "error should be cleared after successful load"
        );
        assert_eq!(model.subscriptions.len(), 1);
    }

    #[test]
    fn unexpected_storage_result_surfaces_as_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::NotFound)),
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_some(), "unexpected result should set error");
        assert!(model.subscriptions.is_empty());
    }

    #[test]
    fn fetch_feed_sets_loading_and_emits_http_effect() {
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::FetchFeed("https://example.com/feed.rss".to_string()),
            &mut model,
        );

        assert!(model.loading);

        let effects: Vec<Effect> = cmd.effects().collect();
        assert_eq!(effects.len(), 2);

        let has_http = effects.iter().any(|e| matches!(
            e,
            Effect::Http(r) if matches!(&r.operation, HttpOperation::FetchFeed { url } if url == "https://example.com/feed.rss")
        ));
        assert!(has_http, "expected a FetchFeed http effect");

        let has_render = effects.iter().any(|e| matches!(e, Effect::Render(_)));
        assert!(has_render, "expected a render effect");
    }

    #[test]
    fn feed_fetched_http_error_sets_error_clears_loading() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/feed.rss".to_string(),
                result: Box::new(HttpResult::Error("connection refused".to_string())),
            },
            &mut model,
        );

        assert!(!model.loading);
        assert_eq!(model.error.as_deref(), Some("connection refused"));
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_fetched_non_200_sets_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/feed.rss".to_string(),
                result: Box::new(HttpResult::Response {
                    status: 404,
                    body: vec![],
                }),
            },
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_some());
        assert!(model.error.as_deref().is_some_and(|e| e.contains("404")));
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_fetched_valid_rss_emits_upsert_storage_effect() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let url = "https://example.com/feed.rss".to_string();
        let mut cmd = app.update(
            Event::FeedFetched {
                url: url.clone(),
                result: Box::new(HttpResult::Response {
                    status: 200,
                    body: MINIMAL_RSS.as_bytes().to_vec(),
                }),
            },
            &mut model,
        );

        // Should not have cleared loading yet — waiting for storage
        assert!(model.error.is_none(), "valid RSS should not set error");

        let effects: Vec<Effect> = cmd.effects().collect();
        assert_eq!(effects.len(), 1, "expected exactly one storage effect");

        let has_upsert = effects.iter().any(|e| matches!(
            e,
            Effect::Storage(r) if matches!(&r.operation, StorageOperation::UpsertFeedWithEpisodes { subscription, episodes }
                if subscription.feed_url == url && !episodes.is_empty())
        ));
        assert!(has_upsert, "expected UpsertFeedWithEpisodes storage effect");
    }

    #[test]
    fn feed_fetched_invalid_xml_sets_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/feed.rss".to_string(),
                result: Box::new(HttpResult::Response {
                    status: 200,
                    body: b"this is not xml".to_vec(),
                }),
            },
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_some(), "invalid XML should set error");
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_saved_adds_new_subscription_to_model() {
        let app = Pollux;
        let mut model = Model::default();
        model.loading = true;

        let sub = make_subscription("new-id", "New Podcast");
        let mut cmd = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(sub))),
            &mut model,
        );

        assert!(!model.loading);
        assert!(model.error.is_none());
        assert_eq!(model.subscriptions.len(), 1);
        assert_eq!(model.subscriptions[0].title, "New Podcast");
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn feed_saved_updates_existing_subscription_in_model() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "Old Title")];

        let updated = Subscription {
            title: "New Title".to_string(),
            ..make_subscription("sub-id", "New Title")
        };
        let _ = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(updated))),
            &mut model,
        );

        assert_eq!(model.subscriptions.len(), 1);
        assert_eq!(model.subscriptions[0].title, "New Title");
    }

    #[test]
    fn feed_saved_resorts_when_refresh_changes_title() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![
            make_subscription("a-id", "Alpha Podcast"),
            make_subscription("z-id", "Zebra Podcast"),
        ];

        // A refresh renames the first feed so it now sorts last.
        let renamed = make_subscription("a-id", "Zulu Podcast");
        let _ = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(renamed))),
            &mut model,
        );

        assert_eq!(model.subscriptions.len(), 2, "rename should not add a row");
        assert_eq!(
            view_titles(&app, &model),
            vec!["Zebra Podcast", "Zulu Podcast"],
            "view should stay alphabetical after an in-place update"
        );
    }

    #[test]
    fn view_sorts_unordered_subscriptions() {
        let app = Pollux;
        let mut model = Model::default();
        // The shell is not required to return rows in any particular order.
        model.subscriptions = vec![
            make_subscription("c-id", "charlie"),
            make_subscription("a-id", "Alpha"),
            make_subscription("b-id", "Bravo"),
        ];

        assert_eq!(
            view_titles(&app, &model),
            vec!["Alpha", "Bravo", "charlie"],
            "view sorts case-insensitively regardless of model order"
        );
    }

    #[test]
    fn feed_saved_sorted_alphabetically() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("b-id", "Zebra Podcast")];

        let sub = make_subscription("a-id", "Alpha Podcast");
        let _ = app.update(
            Event::FeedSaved(Box::new(StorageResult::Subscription(sub))),
            &mut model,
        );

        assert_eq!(model.subscriptions.len(), 2);
        assert_eq!(
            view_titles(&app, &model),
            vec!["Alpha Podcast", "Zebra Podcast"]
        );
    }

    #[test]
    fn select_subscription_sets_header_and_issues_storage_request() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "My Feed")];

        let mut cmd = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);

        assert!(model.detail_loading);
        assert!(model.detail_error.is_none());

        // The header comes from the already-loaded library, not a second fetch.
        let view = app.view(&model);
        assert_eq!(
            view.subscription_detail.subscription_id.as_deref(),
            Some("sub-id")
        );
        assert_eq!(view.subscription_detail.title, "My Feed");

        let effects: Vec<Effect> = cmd.effects().collect();
        let has_list = effects.iter().any(|e| {
            matches!(
                e,
                Effect::Storage(r) if matches!(&r.operation,
                    StorageOperation::ListEpisodesBySubscription { subscription_id }
                        if subscription_id == "sub-id")
            )
        });
        assert!(
            has_list,
            "expected a ListEpisodesBySubscription storage effect"
        );
        let has_render = effects.iter().any(|e| matches!(e, Effect::Render(_)));
        assert!(has_render, "expected a render effect");
    }

    #[test]
    fn episodes_loaded_populates_detail_view() {
        let app = Pollux;
        let mut model = Model::default();
        model.selected_subscription = Some(make_subscription("sub-id", "Feed"));
        model.detail_loading = true;

        let episodes = vec![
            make_episode("e1", "Episode One", Some(1_000)),
            make_episode("e2", "Episode Two", Some(2_000)),
        ];
        let mut cmd = app.update(
            Event::EpisodesLoaded {
                subscription_id: "sub-id".to_string(),
                result: Box::new(StorageResult::Episodes(episodes)),
            },
            &mut model,
        );

        assert!(!model.detail_loading);
        assert!(model.detail_error.is_none());
        cmd.expect_one_effect().expect_render();

        let view = app.view(&model);
        assert_eq!(view.subscription_detail.episodes.len(), 2);
    }

    #[test]
    fn episodes_loaded_defaults_to_newest_first() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("old", "Old", Some(1_000)),
                make_episode("new", "New", Some(3_000)),
                make_episode("mid", "Mid", Some(2_000)),
            ],
        );

        assert_eq!(detail_titles(&app, &model), vec!["New", "Mid", "Old"]);
    }

    #[test]
    fn set_episode_sort_reorders_the_view() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("b", "Bravo", Some(3_000)),
                make_episode("a", "Alpha", Some(1_000)),
                make_episode("c", "Charlie", Some(2_000)),
            ],
        );

        let _ = app.update(
            Event::SetEpisodeSort(EpisodeSortOrder::PubDateAsc),
            &mut model,
        );
        assert_eq!(model.episode_sort, EpisodeSortOrder::PubDateAsc);
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Alpha", "Charlie", "Bravo"]
        );

        let _ = app.update(
            Event::SetEpisodeSort(EpisodeSortOrder::TitleAsc),
            &mut model,
        );
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Alpha", "Bravo", "Charlie"]
        );

        let _ = app.update(
            Event::SetEpisodeSort(EpisodeSortOrder::PubDateDesc),
            &mut model,
        );
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Bravo", "Charlie", "Alpha"]
        );
    }

    #[test]
    fn episodes_without_pub_date_sort_last_in_both_date_orders() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("dated-old", "Dated Old", Some(1_000)),
                make_episode("undated", "Undated", None),
                make_episode("dated-new", "Dated New", Some(2_000)),
            ],
        );

        model.episode_sort = EpisodeSortOrder::PubDateDesc;
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Dated New", "Dated Old", "Undated"],
            "undated sorts last, newest first"
        );

        model.episode_sort = EpisodeSortOrder::PubDateAsc;
        assert_eq!(
            detail_titles(&app, &model),
            vec!["Dated Old", "Dated New", "Undated"],
            "undated sorts last, oldest first"
        );
    }

    #[test]
    fn episodes_loaded_storage_error_sets_detail_error() {
        let app = Pollux;
        let mut model = Model::default();
        model.selected_subscription = Some(make_subscription("sub-id", "Feed"));
        model.detail_loading = true;

        let _ = app.update(
            Event::EpisodesLoaded {
                subscription_id: "sub-id".to_string(),
                result: Box::new(StorageResult::Error("db gone".to_string())),
            },
            &mut model,
        );

        assert!(!model.detail_loading);
        assert_eq!(model.detail_error.as_deref(), Some("db gone"));
    }

    #[test]
    fn episodes_for_a_different_subscription_are_ignored() {
        let app = Pollux;
        let mut model = Model::default();
        // The user has navigated on to another feed; a load for the previous one
        // is still in flight.
        model.selected_subscription = Some(make_subscription("current", "Current"));
        model.detail_loading = true;

        let stale = vec![make_episode("e1", "Stale Episode", Some(1_000))];
        let _ = app.update(
            Event::EpisodesLoaded {
                subscription_id: "previous".to_string(),
                result: Box::new(StorageResult::Episodes(stale)),
            },
            &mut model,
        );

        assert!(
            model.episodes.is_empty(),
            "a response for a feed we left must not populate the current one"
        );
        assert!(
            model.detail_loading,
            "a stale response must not clear the in-flight load's spinner"
        );
    }

    #[test]
    fn selecting_a_subscription_does_not_disturb_library_state() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "My Feed")];
        model.loading = false;
        model.error = Some("library error".to_string());

        let _ = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);

        // Detail load is in flight, but the library's own flags are untouched.
        assert!(!model.loading);
        assert_eq!(model.error.as_deref(), Some("library error"));
        assert!(model.detail_loading);
    }

    #[test]
    fn reselecting_the_current_subscription_does_not_reload() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "My Feed")];

        // First selection loads the feed's episodes.
        let _ = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        assert!(!model.detail_loading);
        assert_eq!(model.episodes.len(), 1);

        // Re-selecting the same, already-loaded feed (e.g. on back-navigation)
        // is a no-op: no storage request, list and loading state untouched.
        let mut cmd = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);
        assert!(!model.detail_loading, "must not re-enter the loading state");
        assert_eq!(model.episodes.len(), 1, "list must be preserved");

        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            !effects.iter().any(|e| matches!(e, Effect::Storage(_))),
            "re-selecting the current feed must not re-query storage"
        );
        assert!(effects.iter().any(|e| matches!(e, Effect::Render(_))));
    }

    #[test]
    fn reselecting_after_an_error_retries_the_load() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-id", "My Feed")];

        let _ = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);
        let _ = app.update(
            Event::EpisodesLoaded {
                subscription_id: "sub-id".to_string(),
                result: Box::new(StorageResult::Error("boom".to_string())),
            },
            &mut model,
        );
        assert!(model.detail_error.is_some());

        // A failed load should not be sticky: re-selecting retries.
        let mut cmd = app.update(Event::SelectSubscription("sub-id".to_string()), &mut model);
        assert!(model.detail_loading);
        assert!(
            model.detail_error.is_none(),
            "retry clears the previous error"
        );

        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::Storage(r)
                    if matches!(&r.operation, StorageOperation::ListEpisodesBySubscription { .. })
            )),
            "an errored load must be retryable"
        );
    }

    #[test]
    fn episode_summary_exposes_raw_and_stripped_description() {
        let app = Pollux;
        let mut model = Model::default();
        let mut episode = make_episode("e1", "Ep", Some(1_000));
        episode.description = Some("<p>Hello <b>world</b></p>".to_string());
        load_episodes(&app, &mut model, vec![episode]);

        let summary = &app.view(&model).subscription_detail.episodes[0];
        // Raw HTML is preserved for the detail page's rich rendering...
        assert_eq!(
            summary.description.as_deref(),
            Some("<p>Hello <b>world</b></p>")
        );
        // ...and a plain-text version is provided for the row snippet.
        assert_eq!(summary.description_text.as_deref(), Some("Hello world"));
    }

    #[test]
    fn episode_summary_description_text_absent_when_no_description() {
        let app = Pollux;
        let mut model = Model::default();
        // make_episode leaves description as None.
        load_episodes(
            &app,
            &mut model,
            vec![make_episode("e1", "Ep", Some(1_000))],
        );

        let summary = &app.view(&model).subscription_detail.episodes[0];
        assert!(summary.description.is_none());
        assert!(summary.description_text.is_none());
    }

    // --- Download manager ---

    fn model_episode_status(model: &Model, id: &str) -> DownloadStatus {
        model
            .episodes
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.download_status.clone())
            .expect("episode should be loaded")
    }

    fn queued_ids(model: &Model) -> Vec<String> {
        model
            .download_queue
            .iter()
            .map(|q| q.episode_id.clone())
            .collect()
    }

    fn has_download_effect_for(effects: &[Effect], episode_id: &str) -> bool {
        effects.iter().any(|e| {
            matches!(
                e,
                Effect::Download(r) if matches!(&r.operation,
                    DownloadOperation::Download { episode_id: id, .. } if id == episode_id)
            )
        })
    }

    #[test]
    fn download_episode_marks_downloading_and_issues_download_effect() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);

        let mut cmd = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);

        // Serial: nothing else in flight, so it starts immediately.
        assert_eq!(model.downloading.as_deref(), Some("e1"));
        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::Downloading
        );
        assert!(
            model.download_queue.is_empty(),
            "the only item should be in flight, not queued"
        );

        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            has_download_effect_for(&effects, "e1"),
            "expected a Download effect for e1"
        );

        // The details page reflects the new state without any re-selection — the
        // in-place model update is what makes idempotent SelectSubscription safe.
        let view = app.view(&model);
        assert_eq!(
            view.subscription_detail.episodes[0].download_status,
            DownloadStatus::Downloading
        );
    }

    #[test]
    fn second_download_stays_queued_while_one_is_in_flight() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("e1", "First", Some(2)),
                make_episode("e2", "Second", Some(1)),
            ],
        );

        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        let mut cmd = app.update(Event::DownloadEpisode("e2".to_string()), &mut model);

        // e1 is still the one downloading; e2 waits its turn.
        assert_eq!(model.downloading.as_deref(), Some("e1"));
        assert_eq!(model_episode_status(&model, "e2"), DownloadStatus::Queued);
        assert_eq!(queued_ids(&model), vec!["e2"]);

        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            !has_download_effect_for(&effects, "e2"),
            "a second download must not start while one is in flight (serial)"
        );
    }

    #[test]
    fn download_completed_marks_downloaded_and_starts_next() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("e1", "First", Some(2)),
                make_episode("e2", "Second", Some(1)),
            ],
        );
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        let _ = app.update(Event::DownloadEpisode("e2".to_string()), &mut model);

        let mut cmd = app.update(
            Event::DownloadFinished {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Completed {
                    local_path: "Downloads/e1.mp3".to_string(),
                    size_bytes: 4_096,
                }),
            },
            &mut model,
        );

        // e1 is done, with its file recorded.
        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::Downloaded
        );
        let e1 = model
            .episodes
            .iter()
            .find(|e| e.id == "e1")
            .expect("e1 should be loaded");
        assert_eq!(e1.local_path.as_deref(), Some("Downloads/e1.mp3"));
        assert_eq!(e1.file_size_bytes, Some(4_096));

        // The queue drains to e2.
        assert_eq!(model.downloading.as_deref(), Some("e2"));
        assert_eq!(
            model_episode_status(&model, "e2"),
            DownloadStatus::Downloading
        );
        assert!(model.download_queue.is_empty());

        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            has_download_effect_for(&effects, "e2"),
            "finishing one download should start the next queued one"
        );
    }

    #[test]
    fn download_error_marks_failed_and_still_drains_queue() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("e1", "First", Some(2)),
                make_episode("e2", "Second", Some(1)),
            ],
        );
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        let _ = app.update(Event::DownloadEpisode("e2".to_string()), &mut model);

        let mut cmd = app.update(
            Event::DownloadFinished {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Error("disk full".to_string())),
            },
            &mut model,
        );

        assert_eq!(model_episode_status(&model, "e1"), DownloadStatus::Failed);
        // A failure must not stall the queue — the next episode still starts.
        assert_eq!(model.downloading.as_deref(), Some("e2"));
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(has_download_effect_for(&effects, "e2"));
    }

    #[test]
    fn delete_download_issues_delete_effect_then_resets_on_result() {
        let app = Pollux;
        let mut model = Model::default();
        let mut downloaded = make_episode("e1", "Ep", Some(1));
        downloaded.download_status = DownloadStatus::Downloaded;
        downloaded.local_path = Some("Downloads/e1.mp3".to_string());
        downloaded.file_size_bytes = Some(4_096);
        load_episodes(&app, &mut model, vec![downloaded]);

        // Deleting a downloaded episode goes to the shell first (best-effort file
        // removal), so the row stays Downloaded until the delete result comes back.
        let mut cmd = app.update(Event::DeleteDownload("e1".to_string()), &mut model);
        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::Downloaded
        );
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::Download(r) if matches!(&r.operation,
                    DownloadOperation::Delete { local_path } if local_path == "Downloads/e1.mp3")
            )),
            "expected a Delete effect carrying the stored path"
        );

        // The shell reports the file gone.
        let _ = app.update(
            Event::DownloadDeleted {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Deleted),
            },
            &mut model,
        );
        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::NotDownloaded
        );
        let e1 = model
            .episodes
            .iter()
            .find(|e| e.id == "e1")
            .expect("e1 should be loaded");
        assert!(e1.local_path.is_none(), "path must be cleared after delete");
        assert!(e1.file_size_bytes.is_none());
    }

    #[test]
    fn delete_without_a_local_file_normalizes_immediately() {
        let app = Pollux;
        let mut model = Model::default();
        let mut failed = make_episode("e1", "Ep", Some(1));
        failed.download_status = DownloadStatus::Failed;
        load_episodes(&app, &mut model, vec![failed]);

        // A Failed row has no file, so there's no shell round-trip — it just resets.
        let mut cmd = app.update(Event::DeleteDownload("e1".to_string()), &mut model);
        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::NotDownloaded
        );
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            !effects.iter().any(|e| matches!(e, Effect::Download(_))),
            "no file means no Delete effect"
        );
    }

    #[test]
    fn download_persist_error_surfaces_on_details_page() {
        let app = Pollux;
        let mut model = Model::default();
        let _ = app.update(
            Event::DownloadStatePersisted(Box::new(StorageResult::Error("db locked".to_string()))),
            &mut model,
        );
        assert_eq!(model.detail_error.as_deref(), Some("db locked"));
    }

    #[test]
    fn download_progress_overlays_bytes_on_the_active_episode() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);

        let _ = app.update(
            Event::DownloadProgress {
                episode_id: "e1".to_string(),
                received_bytes: 512,
                total_bytes: Some(1024),
            },
            &mut model,
        );

        let summary = &app.view(&model).subscription_detail.episodes[0];
        assert_eq!(summary.download_received_bytes, Some(512));
        assert_eq!(summary.download_total_bytes, Some(1024));
    }

    #[test]
    fn download_progress_for_an_inactive_episode_is_ignored() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);

        // Nothing is downloading, so a stray progress report must not stick.
        let _ = app.update(
            Event::DownloadProgress {
                episode_id: "e1".to_string(),
                received_bytes: 10,
                total_bytes: None,
            },
            &mut model,
        );

        assert!(model.active_download_progress.is_none());
        let summary = &app.view(&model).subscription_detail.episodes[0];
        assert!(summary.download_received_bytes.is_none());
    }

    #[test]
    fn progress_is_cleared_when_the_download_finishes() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        let _ = app.update(
            Event::DownloadProgress {
                episode_id: "e1".to_string(),
                received_bytes: 512,
                total_bytes: Some(1024),
            },
            &mut model,
        );
        assert!(model.active_download_progress.is_some());

        let _ = app.update(
            Event::DownloadFinished {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Completed {
                    local_path: "Downloads/e1.mp3".to_string(),
                    size_bytes: 1024,
                }),
            },
            &mut model,
        );

        assert!(
            model.active_download_progress.is_none(),
            "progress must be cleared once the download ends"
        );
        let summary = &app.view(&model).subscription_detail.episodes[0];
        assert!(summary.download_received_bytes.is_none());
    }

    #[test]
    fn pending_downloads_are_reenqueued_and_resumed_on_load() {
        let app = Pollux;
        let mut model = Model::default();

        // Two episodes a previous session left mid-download (their feed is not
        // loaded — mirroring a cold start before any subscription is selected).
        let mut e1 = make_episode("e1", "First", Some(2));
        e1.download_status = DownloadStatus::Downloading;
        let mut e2 = make_episode("e2", "Second", Some(1));
        e2.download_status = DownloadStatus::Queued;

        let mut cmd = app.update(
            Event::PendingDownloadsLoaded(Box::new(StorageResult::Episodes(vec![e1, e2]))),
            &mut model,
        );

        // Serial: the first resumes, the rest wait — even though model.episodes is
        // empty, because the queue carries the URL.
        assert_eq!(model.downloading.as_deref(), Some("e1"));
        assert_eq!(queued_ids(&model), vec!["e2"]);

        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            has_download_effect_for(&effects, "e1"),
            "an interrupted download should resume at launch"
        );
    }

    #[test]
    fn pending_downloads_loaded_empty_is_a_noop() {
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::PendingDownloadsLoaded(Box::new(StorageResult::Episodes(vec![]))),
            &mut model,
        );

        assert!(model.downloading.is_none());
        assert!(model.download_queue.is_empty());
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(!effects.iter().any(|e| matches!(e, Effect::Download(_))));
    }

    #[test]
    fn cancel_active_download_requests_a_shell_cancel() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        assert_eq!(model.downloading.as_deref(), Some("e1"));

        let mut cmd = app.update(Event::CancelDownload("e1".to_string()), &mut model);
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            effects.iter().any(|e| matches!(
                e,
                Effect::Download(r) if matches!(&r.operation,
                    DownloadOperation::Cancel { episode_id } if episode_id == "e1")
            )),
            "cancelling the active download should ask the shell to cancel the task"
        );
    }

    #[test]
    fn download_finished_cancelled_resets_and_starts_next() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("e1", "First", Some(2)),
                make_episode("e2", "Second", Some(1)),
            ],
        );
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        let _ = app.update(Event::DownloadEpisode("e2".to_string()), &mut model);
        let _ = app.update(Event::CancelDownload("e1".to_string()), &mut model);

        // The shell reports the cancellation back through the Download request.
        let mut cmd = app.update(
            Event::DownloadFinished {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Cancelled),
            },
            &mut model,
        );

        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::NotDownloaded,
            "a cancelled download goes back to not-downloaded, not failed"
        );
        assert!(model.active_download_progress.is_none());
        // The queue drains to e2.
        assert_eq!(model.downloading.as_deref(), Some("e2"));
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(has_download_effect_for(&effects, "e2"));
    }

    #[test]
    fn cancel_queued_download_drops_it_without_a_shell_cancel() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(
            &app,
            &mut model,
            vec![
                make_episode("e1", "First", Some(2)),
                make_episode("e2", "Second", Some(1)),
            ],
        );
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        let _ = app.update(Event::DownloadEpisode("e2".to_string()), &mut model);
        assert_eq!(queued_ids(&model), vec!["e2"]);

        let mut cmd = app.update(Event::CancelDownload("e2".to_string()), &mut model);

        // e2 leaves the queue and resets; e1 keeps downloading.
        assert!(model.download_queue.is_empty());
        assert_eq!(
            model_episode_status(&model, "e2"),
            DownloadStatus::NotDownloaded
        );
        assert_eq!(model.downloading.as_deref(), Some("e1"));
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            !effects.iter().any(|e| matches!(
                e,
                Effect::Download(r) if matches!(&r.operation, DownloadOperation::Cancel { .. })
            )),
            "a queued item needs no shell cancel — nothing is running"
        );
    }

    #[test]
    fn downloading_an_already_downloaded_episode_is_a_noop() {
        let app = Pollux;
        let mut model = Model::default();
        let mut downloaded = make_episode("e1", "Ep", Some(1));
        downloaded.download_status = DownloadStatus::Downloaded;
        downloaded.local_path = Some("Downloads/e1.mp3".to_string());
        load_episodes(&app, &mut model, vec![downloaded]);

        // A stray DownloadEpisode must not re-download a file we already have.
        let mut cmd = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);

        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::Downloaded
        );
        assert!(model.download_queue.is_empty());
        assert!(model.downloading.is_none());
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            !has_download_effect_for(&effects, "e1"),
            "downloaded episode must not be re-fetched"
        );
    }

    #[test]
    fn redownloading_an_in_flight_episode_is_a_noop() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);

        // Tapping download again while it's downloading must not re-enqueue.
        let mut cmd = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        assert!(model.download_queue.is_empty());
        let effects: Vec<Effect> = cmd.effects().collect();
        assert!(
            !has_download_effect_for(&effects, "e1"),
            "a redundant download request must not issue a second fetch"
        );
    }
}
