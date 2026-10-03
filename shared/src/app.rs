use crux_core::{render::render, App, Command};
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::capabilities::download::{DownloadOperation, DownloadResult};
use crate::capabilities::http::{HttpOperation, HttpResult};
use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::defaults::{
    FAILURE_BACKOFF_SECS, MAX_RETRY_AFTER_SECS, RATE_LIMIT_BACKOFF_SECS, REFRESH_INTERVAL_HOURS,
};
use crate::domain::{DownloadStatus, Episode, EpisodeSortOrder, Subscription};
use crate::effect::Effect;
use crate::feed_parser::{now_unix, parse_feed};
use crate::html::strip_html_preview;
use crate::model::{DownloadProgress, Model, QueuedDownload};
use crate::theme::{theme_view, ThemeId, ThemeMode};
use crate::view_model::{
    DownloadNotice, EpisodeSummary, LibraryView, SubscriptionDetailView, SubscriptionSummary,
    ViewModel,
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
                // Interrupted downloads are deliberately NOT resumed here: the core is
                // also created when iOS launches the app in the background (for a
                // refresh task), and that launch must stay metadata-only. They resume
                // on the first foreground activation instead (`ResumePendingDownloads`).
                Command::request_from_shell(StorageOperation::ListSubscriptions)
                    .then_send(|r| Event::SubscriptionsLoaded(Box::new(r)))
                    .and(render())
            }
            Event::ResumePendingDownloads => {
                // Once per process: the shell sends this on every activation, but the
                // queue only needs rebuilding from the DB the first time. Later
                // activations would re-list episodes already in the in-memory queue.
                if model.pending_downloads_requested {
                    return Command::done();
                }
                model.pending_downloads_requested = true;
                // Rebuild the download queue for anything a previous session left
                // mid-download, and resume it.
                Command::request_from_shell(StorageOperation::ListPendingDownloads)
                    .then_send(|r| Event::PendingDownloadsLoaded(Box::new(r)))
            }
            Event::SubscriptionsLoaded(result) => {
                model.loading = false;
                match *result {
                    StorageResult::Subscriptions(rows) => {
                        model.subscriptions = rows;
                        model.error = None;
                        model.subscriptions_loaded = true;
                    }
                    StorageResult::Error(e) => model.error = Some(e),
                    unexpected => {
                        model.error = Some(format!("unexpected storage result: {unexpected:?}"))
                    }
                }
                if model.subscriptions_loaded && model.auto_refresh_pending {
                    model.auto_refresh_pending = false;
                    enqueue_due_refreshes(model);
                }
                maybe_start_next_refresh(model).and(render())
            }
            Event::PendingDownloadsLoaded(result) => {
                // Rebuild the queue from downloads a prior session left in flight and
                // resume them. Partial files don't survive a quit, so each restarts
                // from scratch. Best-effort: an error here just leaves nothing queued.
                let mut cmd = Command::done();
                if let StorageResult::Episodes(rows) = *result {
                    // Nothing is in flight at launch, so `maybe_start_next` will start
                    // the first re-enqueued item (persisting it as `Downloading`).
                    let will_start_head = model.downloading.is_none();
                    for episode in rows {
                        let already = model.downloading.as_deref() == Some(episode.id.as_str())
                            || model
                                .download_queue
                                .iter()
                                .any(|q| q.episode_id == episode.id);
                        if already {
                            continue;
                        }
                        // The head item is about to be started, so skip a `Queued` write
                        // that its `Downloading` write would immediately supersede. Items
                        // waiting behind it are normalized to `Queued` so one left
                        // "Downloading" by a prior session no longer reads that way.
                        let is_head = will_start_head && model.download_queue.is_empty();
                        if !is_head {
                            cmd = cmd.and(persist_download_state(
                                &episode.id,
                                DownloadStatus::Queued,
                                None,
                                None,
                            ));
                        }
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
                Command::request_from_shell(HttpOperation::FetchFeed {
                    url: url.clone(),
                    etag: None,
                    last_modified: None,
                })
                .then_send(move |r| Event::FeedFetched {
                    url,
                    result: Box::new(r),
                })
                .and(render())
            }
            Event::FeedFetched { url, result } => match *result {
                HttpResult::Error(e) | HttpResult::Unreachable(e) => {
                    model.loading = false;
                    model.error = Some(e);
                    render()
                }
                HttpResult::Response {
                    status: 200,
                    body,
                    etag,
                    last_modified,
                    ..
                } => match parse_feed(&url, body) {
                    Ok((mut subscription, episodes)) => {
                        // Re-adding a feed we already have goes through the same upsert
                        // as a refresh, so it gets the same protection.
                        if let Some(existing) =
                            model.subscriptions.iter().find(|s| s.feed_url == url)
                        {
                            subscription.inherit_missing_metadata(existing);
                        }
                        // Keep the validators so the first refresh can be conditional.
                        subscription.etag = etag;
                        subscription.last_modified = last_modified;
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
            Event::RefreshSubscription(id) => {
                enqueue_refresh(model, &id);
                maybe_start_next_refresh(model).and(render())
            }
            Event::RefreshStale => {
                if model.subscriptions_loaded {
                    enqueue_due_refreshes(model);
                    maybe_start_next_refresh(model).and(render())
                } else {
                    // Nothing to judge staleness against yet: hold the request until the
                    // library arrives (see `SubscriptionsLoaded`).
                    model.auto_refresh_pending = true;
                    if model.loading {
                        // The launch load is still in flight and will honour it.
                        render()
                    } else {
                        // Not loaded and nothing loading means the last load failed.
                        // Nothing else retries it, so without this the held request
                        // (and the library itself) would stay stuck until the app is
                        // relaunched. Retry on each activation; success honours it.
                        model.loading = true;
                        Command::request_from_shell(StorageOperation::ListSubscriptions)
                            .then_send(|r| Event::SubscriptionsLoaded(Box::new(r)))
                            .and(render())
                    }
                }
            }
            Event::RefreshAll => {
                enqueue_in_library_order(model, |_| true);
                maybe_start_next_refresh(model).and(render())
            }
            Event::CancelRefresh => {
                // Drops what is waiting, not what is running: the fetch already in
                // flight resolves normally (its outcome is still recorded), but
                // finishing it no longer starts the next feed. Also drops an
                // auto-refresh held for the library to load, which would otherwise
                // fire after cancellation.
                model.refresh_queue.clear();
                model.auto_refresh_pending = false;
                render()
            }
            Event::RefreshFetched {
                subscription_id,
                result,
            } => {
                let Some(sub) = model
                    .subscriptions
                    .iter()
                    .find(|s| s.id == subscription_id)
                    .cloned()
                else {
                    // Unsubscribed while the fetch was in flight.
                    finish_refresh(model, &subscription_id);
                    return maybe_start_next_refresh(model).and(render());
                };
                match *result {
                    // The device is offline (or similar): nothing is wrong with this
                    // feed. Record why it didn't refresh so the user sees it, but leave
                    // `retry_after_until` alone. Backing the feed off would keep
                    // auto-refresh away for 15 minutes after connectivity returns, and
                    // would do it to every feed at once.
                    HttpResult::Unreachable(e) => record_refresh_outcome(
                        model,
                        &subscription_id,
                        None,
                        Some(e),
                        sub.retry_after_until,
                    ),
                    HttpResult::Error(e) => record_refresh_outcome(
                        model,
                        &subscription_id,
                        None,
                        Some(e),
                        Some(failure_backoff_until()),
                    ),
                    HttpResult::Response {
                        status: 200,
                        body,
                        etag,
                        last_modified,
                        ..
                    } => match parse_feed(&sub.feed_url, body) {
                        Ok((mut fresh, episodes)) => {
                            // A partial response must not wipe the title, artwork or
                            // description we already have (storage writes these as-is).
                            fresh.inherit_missing_metadata(&sub);
                            fresh.etag = etag;
                            fresh.last_modified = last_modified;
                            Command::request_from_shell(StorageOperation::UpsertFeedWithEpisodes {
                                subscription: fresh,
                                episodes,
                            })
                            .then_send(move |r| Event::RefreshSaved {
                                subscription_id: subscription_id.clone(),
                                result: Box::new(r),
                            })
                        }
                        Err(e) => record_refresh_outcome(
                            model,
                            &subscription_id,
                            None,
                            Some(e),
                            Some(failure_backoff_until()),
                        ),
                    },
                    // Unchanged since the validators we sent: only the timestamp moves.
                    HttpResult::Response { status: 304, .. } => record_refresh_outcome(
                        model,
                        &subscription_id,
                        Some(now_unix()),
                        None,
                        None,
                    ),
                    HttpResult::Response {
                        status: 429,
                        retry_after_secs,
                        ..
                    } => record_refresh_outcome(
                        model,
                        &subscription_id,
                        None,
                        Some("The host asked us to slow down (HTTP 429)".to_string()),
                        Some(now_unix().saturating_add(rate_limit_wait_secs(retry_after_secs))),
                    ),
                    HttpResult::Response { status, .. } => record_refresh_outcome(
                        model,
                        &subscription_id,
                        None,
                        Some(format!("feed fetch failed: HTTP {status}")),
                        Some(failure_backoff_until()),
                    ),
                }
            }
            Event::RefreshSaved {
                subscription_id,
                result,
            } => match *result {
                StorageResult::Subscription(saved) => {
                    let selected = model
                        .selected_subscription
                        .as_ref()
                        .is_some_and(|s| s.id == saved.id);
                    if let Some(pos) = model.subscriptions.iter().position(|s| s.id == saved.id) {
                        model.subscriptions[pos] = saved.clone();
                    }
                    finish_refresh(model, &subscription_id);
                    let mut cmd = Command::done();
                    if selected {
                        // `SelectSubscription` is idempotent, so a refresh must reload
                        // the open feed's episodes explicitly. No `detail_loading`: the
                        // existing list stays on screen while the new rows load.
                        model.selected_subscription = Some(saved);
                        let id = subscription_id.clone();
                        cmd = Command::request_from_shell(
                            StorageOperation::ListEpisodesBySubscription {
                                subscription_id: id.clone(),
                            },
                        )
                        .then_send(move |r| Event::EpisodesLoaded {
                            subscription_id: id.clone(),
                            result: Box::new(r),
                        });
                    }
                    cmd.and(maybe_start_next_refresh(model)).and(render())
                }
                StorageResult::Error(e) => record_refresh_outcome(
                    model,
                    &subscription_id,
                    None,
                    Some(e),
                    Some(failure_backoff_until()),
                ),
                unexpected => record_refresh_outcome(
                    model,
                    &subscription_id,
                    None,
                    Some(format!("unexpected storage result: {unexpected:?}")),
                    Some(failure_backoff_until()),
                ),
            },
            Event::RefreshStatePersisted(_result) => {
                // Required sink for the `UpdateRefreshState` request. The in-memory
                // subscription already reflects the outcome, and a failed write only
                // costs durability across a restart, so there is nothing to surface.
                Command::done()
            }
            Event::SelectSubscription(id) => {
                // Re-selecting the feed already on screen (e.g. returning from an
                // episode detail, which re-fires the shell's selection task) must
                // not reload it: that would clear the list and flash a spinner for
                // no new data. Reload only when the selection changes, or when the
                // previous load errored and should be retried.
                //
                // NOTE: this makes re-selection a no-op even if the feed's stored
                // episodes changed since the last load. A feed refresh therefore
                // reloads the open feed's episodes explicitly (see `RefreshSaved`)
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
                    // A notice from the previous feed doesn't apply to this one.
                    model.download_notice = None;
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
                // A fresh user action supersedes any stale op-failure notice.
                model.download_notice = None;
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
                DownloadResult::Error(reason) => {
                    finish_active(model, &episode_id);
                    set_download_state(model, &episode_id, DownloadStatus::Failed, None, None);
                    // Keep the shell's specific reason so the UI can show it beside
                    // Retry (set_download_state cleared any older one above).
                    model.download_errors.insert(episode_id.clone(), reason);
                    let persist =
                        persist_download_state(&episode_id, DownloadStatus::Failed, None, None);
                    persist.and(maybe_start_next(model)).and(render())
                }
                DownloadResult::Cancelled => {
                    // The shell cancelled the task and flushed the partial file, so
                    // the episode goes back to not-downloaded and the queue drains on.
                    // This is the *sole* owner of a cancelled download's state reset —
                    // the `Cancel` operation's own ack (DownloadCanceled) does nothing.
                    finish_active(model, &episode_id);
                    reset_to_not_downloaded(model, &episode_id)
                        .and(maybe_start_next(model))
                        .and(render())
                }
                // Deleted never arrives here (it resolves DeleteDownload instead).
                DownloadResult::Deleted => render(),
            },
            Event::CancelDownload(episode_id) => {
                // A fresh user action supersedes any stale op-failure notice.
                model.download_notice = None;
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
                    // nothing is running and no file exists yet, so no shell cancel is
                    // needed and the reset happens synchronously here.
                    model.download_queue.remove(pos);
                    reset_to_not_downloaded(model, &episode_id).and(render())
                } else {
                    // Not downloading and not queued — nothing to cancel.
                    render()
                }
            }
            Event::DownloadCanceled(_result) => {
                // The `Cancel` request must be resolved (the shell resolves every
                // request), so this is its required sink — nothing more. The actual
                // state reset is owned entirely by the in-flight `Download` request
                // resolving `Cancelled` (see DownloadFinished above); if the cancel
                // somehow didn't take, that download simply resolves normally, so
                // there is nothing to surface here.
                render()
            }
            Event::DeleteDownload(episode_id) => {
                // A fresh user action supersedes any stale op-failure notice.
                model.download_notice = None;
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
                        reset_to_not_downloaded(model, &episode_id).and(render())
                    }
                }
            }
            Event::DownloadDeleted { episode_id, result } => match *result {
                DownloadResult::Deleted => {
                    reset_to_not_downloaded(model, &episode_id).and(render())
                }
                DownloadResult::Error(e) => {
                    // The file couldn't be removed; leave the row (still Downloaded) as
                    // it is and surface why through the non-blocking notice rather than
                    // the list-load error, which would hide every episode and control.
                    model.download_notice = Some(DownloadNotice {
                        episode_id,
                        message: format!("Couldn't remove the download: {e}"),
                    });
                    render()
                }
                // Only Deleted/Error arrive from a Delete request; the rest are
                // unreachable but keep the match total.
                DownloadResult::Completed { .. } | DownloadResult::Cancelled => render(),
            },
            Event::DownloadStatePersisted { episode_id, result } => {
                // A durability failure: the UI already shows the correct optimistic
                // state, so surface this as a non-blocking notice, not the list error.
                // Success is silent — notably it does NOT clear an existing notice, so
                // an unrelated episode's persist can't dismiss another episode's failure
                // message. The notice clears on the next user download action (see
                // DownloadEpisode/DeleteDownload/CancelDownload) or on a feed switch.
                if let StorageResult::Error(e) = *result {
                    model.download_notice = Some(DownloadNotice {
                        episode_id,
                        message: format!("Couldn't save the download's state: {e}"),
                    });
                }
                render()
            }
            Event::SetTheme { id, mode } => {
                // Records the theme choice and re-renders; resolution to platform
                // colors happens shell-side. No UI emits this yet — it's the seam
                // the Settings appearance section will use. Persistence hooks in
                // there too (via the storage capability).
                model.theme_id = id;
                model.theme_mode = mode;
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
                refreshing: is_refresh_pending(model, &s.id),
                refresh_error: s.last_refresh_error.clone(),
            })
            .collect();
        subscriptions.sort_by_cached_key(|s| s.title.to_lowercase());

        ViewModel {
            library: LibraryView {
                subscriptions,
                loading: model.loading,
                error: model.error.clone(),
                refreshing: model.refreshing.is_some() || !model.refresh_queue.is_empty(),
            },
            subscription_detail: build_subscription_detail(model),
            theme: theme_view(model.theme_id, model.theme_mode),
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

    // Overlay each failed episode's reason (also transient, not stored on the Episode).
    // Skipped entirely when nothing has failed, which is the common case.
    if !model.download_errors.is_empty() {
        for summary in episodes.iter_mut() {
            summary.download_error = model.download_errors.get(&summary.id).cloned();
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
        download_notice: model.download_notice.clone(),
        refreshing: model
            .selected_subscription
            .as_ref()
            .is_some_and(|s| is_refresh_pending(model, &s.id)),
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
        // Live progress and the failure reason are overlaid in
        // `build_subscription_detail`; every row ships without them here.
        download_received_bytes: None,
        download_total_bytes: None,
        download_error: None,
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
    // A recorded failure reason is only meaningful while the episode is Failed; every
    // other transition drops it. The `Error` handler re-inserts after calling this.
    if !matches!(status, DownloadStatus::Failed) {
        model.download_errors.remove(episode_id);
    }
    if let Some(episode) = model.episodes.iter_mut().find(|e| e.id == episode_id) {
        episode.download_status = status;
        episode.local_path = local_path;
        episode.file_size_bytes = size_bytes;
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
    // Capture the id so a persistence failure's notice can be tagged with its episode.
    let id = episode_id.to_string();
    Command::request_from_shell(StorageOperation::UpdateDownloadState {
        episode_id: episode_id.to_string(),
        status,
        local_path,
        size_bytes,
    })
    .then_send(move |r| Event::DownloadStatePersisted {
        episode_id: id.clone(),
        result: Box::new(r),
    })
}

/// Resets an episode to not-downloaded (in the model) and returns the command that
/// persists it. The single shape used everywhere a download ends without a file —
/// cancelling an active or queued download, and deleting a stored one.
fn reset_to_not_downloaded(model: &mut Model, episode_id: &str) -> Command<Effect, Event> {
    set_download_state(model, episode_id, DownloadStatus::NotDownloaded, None, None);
    persist_download_state(episode_id, DownloadStatus::NotDownloaded, None, None)
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

/// True while `id` is being fetched or waiting its turn in the refresh queue.
fn is_refresh_pending(model: &Model, id: &str) -> bool {
    model.refreshing.as_deref() == Some(id) || model.refresh_queue.iter().any(|q| q == id)
}

/// Queues a known subscription for refresh. Unknown ids and ones already queued or in
/// flight are ignored, so a repeated pull-to-refresh can't double-fetch a feed.
fn enqueue_refresh(model: &mut Model, id: &str) {
    if model.subscriptions.iter().any(|s| s.id == id) && !is_refresh_pending(model, id) {
        model.refresh_queue.push(id.to_string());
    }
}

/// Queues every subscription `include` accepts, in library order: case-insensitive by
/// title, as the library list is displayed (see `view`), with the id breaking ties so the
/// order is deterministic. The feeds the user sees first refresh first. Feeds already
/// queued or in flight are skipped by `enqueue_refresh`.
fn enqueue_in_library_order(model: &mut Model, include: impl Fn(&Subscription) -> bool) {
    let mut ids: Vec<(String, String)> = model
        .subscriptions
        .iter()
        .filter(|s| include(s))
        .map(|s| (s.title.to_lowercase(), s.id.clone()))
        .collect();
    ids.sort();
    for (_, id) in ids {
        enqueue_refresh(model, &id);
    }
}

/// Queues every feed that is due for an automatic refresh (see `is_due_for_auto_refresh`).
fn enqueue_due_refreshes(model: &mut Model) {
    let now = now_unix();
    enqueue_in_library_order(model, |s| is_due_for_auto_refresh(s, now));
}

/// Whether `sub` is due for an *automatic* refresh at `now`: never refreshed or last
/// refreshed at least `REFRESH_INTERVAL_HOURS` ago, and not inside a backoff.
///
/// Both timestamps are wall-clock values written earlier, so they can be wrong in a way
/// that would otherwise keep a feed out of auto-refresh for as long as the error is large:
///
/// - A `last_refreshed` in the future means the clock was ahead (or has since stepped
///   back) when it was written. A bare `now - last_refreshed >= interval` would stay
///   false until real time reached it, so a feed stamped a year ahead would never refresh.
///   It is treated as stale; a conditional GET makes the redundant fetch cheap, and the
///   result stamps a correct time.
/// - A `retry_after_until` is never written more than `MAX_RETRY_AFTER_SECS` ahead of the
///   clock at the time, and the clock only moves forward in normal use, so one further
///   out than that can only be skew. It is ignored rather than honoured for a year.
fn is_due_for_auto_refresh(sub: &Subscription, now: i64) -> bool {
    let interval = i64::from(REFRESH_INTERVAL_HOURS) * 3600;
    let stale = sub
        .last_refreshed
        .is_none_or(|t| t > now || now.saturating_sub(t) >= interval);
    let backing_off = sub
        .retry_after_until
        .is_some_and(|t| t > now && t.saturating_sub(now) <= MAX_RETRY_AFTER_SECS);
    stale && !backing_off
}

/// How long to leave a feed alone after a 429: the host's `Retry-After` when it sent one,
/// otherwise the default, but never more than `MAX_RETRY_AFTER_SECS`. A value too large
/// for an `i64` clamps to the cap rather than falling back to the (much shorter) default.
fn rate_limit_wait_secs(retry_after_secs: Option<u64>) -> i64 {
    retry_after_secs
        .map_or(RATE_LIMIT_BACKOFF_SECS, |s| {
            i64::try_from(s).unwrap_or(i64::MAX)
        })
        .min(MAX_RETRY_AFTER_SECS)
}

/// When auto-refresh may retry a feed whose refresh just failed.
fn failure_backoff_until() -> i64 {
    now_unix().saturating_add(FAILURE_BACKOFF_SECS)
}

/// Clears the in-flight marker when the active refresh reaches a terminal state.
fn finish_refresh(model: &mut Model, id: &str) {
    if model.refreshing.as_deref() == Some(id) {
        model.refreshing = None;
    }
}

/// Starts the next queued refresh if none is in flight (serial). Sends the stored
/// validators so an unchanged feed answers 304. Empty when busy or the queue is empty.
fn maybe_start_next_refresh(model: &mut Model) -> Command<Effect, Event> {
    while model.refreshing.is_none() && !model.refresh_queue.is_empty() {
        let id = model.refresh_queue.remove(0);
        // The feed may have been removed since it was queued.
        let Some(sub) = model.subscriptions.iter().find(|s| s.id == id) else {
            continue;
        };
        let (url, etag, last_modified) = (
            sub.feed_url.clone(),
            sub.etag.clone(),
            sub.last_modified.clone(),
        );
        model.refreshing = Some(id.clone());
        return Command::request_from_shell(HttpOperation::FetchFeed {
            url,
            etag,
            last_modified,
        })
        .then_send(move |r| Event::RefreshFetched {
            subscription_id: id.clone(),
            result: Box::new(r),
        });
    }
    Command::done()
}

/// Ends the active refresh without a new feed body: applies the outcome to the
/// in-memory subscription, persists it, and starts the next queued refresh.
/// `last_refreshed` is only moved when `Some` (a 304); `error` is the failure reason.
fn record_refresh_outcome(
    model: &mut Model,
    id: &str,
    last_refreshed: Option<i64>,
    error: Option<String>,
    retry_after_until: Option<i64>,
) -> Command<Effect, Event> {
    finish_refresh(model, id);
    let mut persist = Command::done();
    for sub in [
        model.subscriptions.iter_mut().find(|s| s.id == id),
        model.selected_subscription.as_mut().filter(|s| s.id == id),
    ]
    .into_iter()
    .flatten()
    {
        if last_refreshed.is_some() {
            sub.last_refreshed = last_refreshed;
        }
        sub.last_refresh_error = error.clone();
        sub.retry_after_until = retry_after_until;
    }
    if model.subscriptions.iter().any(|s| s.id == id) {
        persist = Command::request_from_shell(StorageOperation::UpdateRefreshState {
            subscription_id: id.to_string(),
            last_refreshed,
            last_refresh_error: error,
            retry_after_until,
        })
        .then_send(|r| Event::RefreshStatePersisted(Box::new(r)));
    }
    persist.and(maybe_start_next_refresh(model)).and(render())
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Started,
    SubscriptionsLoaded(Box<StorageResult>),
    /// Rebuild the download queue from episodes a previous session left
    /// `Downloading`/`Queued`, and resume them. Sent when the app first becomes
    /// active, never at launch, so a background launch (e.g. for feed refresh) can't
    /// start a download. Idempotent: only the first one per process does anything.
    ResumePendingDownloads,
    /// Episodes left `Downloading`/`Queued` by a previous session, so their downloads
    /// can be re-enqueued and resumed (see `ResumePendingDownloads`).
    PendingDownloadsLoaded(Box<StorageResult>),
    FetchFeed(String),
    FeedFetched {
        url: String,
        result: Box<HttpResult>,
    },
    FeedSaved(Box<StorageResult>),
    /// Re-fetch one subscription's feed (conditional GET) and merge new episodes.
    /// Metadata only — never triggers downloads.
    RefreshSubscription(String),
    /// Queue every subscription for refresh, one at a time, in library order.
    RefreshAll,
    /// Refresh the feeds that are due (older than the refresh interval and not backing
    /// off). Sent when the app becomes active; held until the library has loaded if it
    /// arrives first (i.e. at launch).
    RefreshStale,
    /// Stop starting new refreshes: clear the waiting queue and any held auto-refresh.
    /// The fetch already in flight is left to finish. Sent when the background task
    /// expires, so the app doesn't keep working through the queue after being told to
    /// stop.
    CancelRefresh,
    /// Result of a refresh fetch for `subscription_id`.
    RefreshFetched {
        subscription_id: String,
        result: Box<HttpResult>,
    },
    /// Result of saving a refreshed feed's new body.
    RefreshSaved {
        subscription_id: String,
        result: Box<StorageResult>,
    },
    /// Sink for the `UpdateRefreshState` request; carries no state change.
    RefreshStatePersisted(Box<StorageResult>),
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
    /// Required resolution sink for the `Cancel` request — carries no state change.
    /// The reset is owned by the in-flight `Download` request resolving `Cancelled`.
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
    DownloadStatePersisted {
        episode_id: String,
        result: Box<StorageResult>,
    },
    /// Change the active theme. Not yet emitted by any UI — the seam for the
    /// Settings appearance section (see `docs/features/theme.md`).
    SetTheme {
        id: ThemeId,
        mode: ThemeMode,
    },
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
            etag: None,
            last_modified: None,
            last_refresh_error: None,
            retry_after_until: None,
        }
    }

    fn response(status: u16, body: Vec<u8>) -> HttpResult {
        HttpResult::Response {
            status,
            body,
            etag: None,
            last_modified: None,
            retry_after_secs: None,
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
        // ListSubscriptions + render.
        assert_eq!(effects.len(), 2);

        let has_storage = effects
            .iter()
            .any(|e| matches!(e, Effect::Storage(r) if matches!(r.operation, StorageOperation::ListSubscriptions)));
        assert!(has_storage, "expected a ListSubscriptions storage effect");

        let has_render = effects.iter().any(|e| matches!(e, Effect::Render(_)));
        assert!(has_render, "expected a render effect");
    }

    #[test]
    fn started_does_not_touch_downloads_so_a_background_launch_stays_metadata_only() {
        // The core is also built when iOS launches the app in the background for a
        // refresh task, and that launch must never start a download.
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(Event::Started, &mut model);

        assert!(
            !storage_ops(&mut cmd)
                .iter()
                .any(|o| matches!(o, StorageOperation::ListPendingDownloads)),
            "Started must not load interrupted downloads"
        );
        assert!(model.download_queue.is_empty());
        assert!(model.downloading.is_none());
        assert!(!model.pending_downloads_requested);
    }

    #[test]
    fn resume_pending_downloads_requests_the_interrupted_downloads() {
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(Event::ResumePendingDownloads, &mut model);

        assert!(model.pending_downloads_requested);
        assert!(
            storage_ops(&mut cmd)
                .iter()
                .any(|o| matches!(o, StorageOperation::ListPendingDownloads)),
            "expected a ListPendingDownloads storage effect"
        );
    }

    #[test]
    fn resume_pending_downloads_only_acts_once_per_process() {
        let app = Pollux;
        let mut model = Model::default();
        let _ = app.update(Event::ResumePendingDownloads, &mut model);

        // The shell sends this on every activation; later ones must not re-list
        // episodes that are already in the in-memory queue.
        let mut again = app.update(Event::ResumePendingDownloads, &mut model);

        assert!(storage_ops(&mut again).is_empty());
    }

    #[test]
    fn resumed_downloads_start_after_the_pending_list_loads() {
        let app = Pollux;
        let mut model = Model::default();
        let _ = app.update(Event::ResumePendingDownloads, &mut model);
        let mut episode = make_episode("e1", "Interrupted", Some(1));
        episode.download_status = DownloadStatus::Downloading;

        let mut cmd = app.update(
            Event::PendingDownloadsLoaded(Box::new(StorageResult::Episodes(vec![episode]))),
            &mut model,
        );

        assert_eq!(model.downloading.as_deref(), Some("e1"));
        assert!(cmd.effects().any(|e| matches!(e, Effect::Download(_))));
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
            Effect::Http(r) if matches!(&r.operation, HttpOperation::FetchFeed { url, etag: None, last_modified: None } if url == "https://example.com/feed.rss")
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
                result: Box::new(response(404, vec![])),
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
                result: Box::new(response(200, MINIMAL_RSS.as_bytes().to_vec())),
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
                result: Box::new(response(200, b"this is not xml".to_vec())),
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
    fn default_view_projects_the_system_theme() {
        let app = Pollux;
        let model = Model::default();

        let theme = app.view(&model).theme;
        assert_eq!(theme.id, ThemeId::System);
        assert_eq!(theme.mode, ThemeMode::FollowSystem);
        assert!(theme.light.is_none() && theme.dark.is_none());
    }

    #[test]
    fn set_theme_updates_the_projected_theme() {
        let app = Pollux;
        let mut model = Model::default();

        let mut cmd = app.update(
            Event::SetTheme {
                id: ThemeId::Solarized,
                mode: ThemeMode::Dark,
            },
            &mut model,
        );
        cmd.expect_one_effect().expect_render();

        let theme = app.view(&model).theme;
        assert_eq!(theme.id, ThemeId::Solarized);
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert!(theme.light.is_some() && theme.dark.is_some());
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
    fn download_failure_reason_is_exposed_on_the_episode() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);

        let _ = app.update(
            Event::DownloadFinished {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Error("disk full".to_string())),
            },
            &mut model,
        );

        // The specific reason rides along on the failed episode's summary.
        let summary = &app.view(&model).subscription_detail.episodes[0];
        assert_eq!(summary.download_status, DownloadStatus::Failed);
        assert_eq!(summary.download_error.as_deref(), Some("disk full"));
    }

    #[test]
    fn retrying_a_failed_download_clears_the_reason() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        let _ = app.update(
            Event::DownloadFinished {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Error("boom".to_string())),
            },
            &mut model,
        );
        assert!(!model.download_errors.is_empty());

        // Retrying (a DownloadEpisode on the failed episode) clears the reason and
        // starts over.
        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);
        assert!(model.download_errors.is_empty());
        let summary = &app.view(&model).subscription_detail.episodes[0];
        assert!(summary.download_error.is_none());
        assert_eq!(summary.download_status, DownloadStatus::Downloading);
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
    fn download_persist_error_surfaces_as_a_notice_not_the_list_error() {
        let app = Pollux;
        let mut model = Model::default();
        // A pre-existing list-load error must survive a download op-failure.
        model.detail_error = Some("earlier list load failed".to_string());

        let _ = app.update(
            Event::DownloadStatePersisted {
                episode_id: "e1".to_string(),
                result: Box::new(StorageResult::Error("db locked".to_string())),
            },
            &mut model,
        );

        // The op failure lands in the non-blocking notice, tagged with its episode, and
        // the list error (which would blank the whole episode list) is untouched.
        let notice = model.download_notice.as_ref().expect("a notice");
        assert_eq!(notice.episode_id, "e1");
        assert!(notice.message.contains("db locked"));
        assert_eq!(
            model.detail_error.as_deref(),
            Some("earlier list load failed")
        );
    }

    #[test]
    fn an_unrelated_successful_persist_does_not_clear_the_notice() {
        // A different episode's persist succeeding must NOT dismiss a standing
        // op-failure notice — otherwise a background download completing could wipe
        // the message before the user sees it. The notice clears on a user action or
        // feed switch instead (covered by other tests).
        let app = Pollux;
        let mut model = Model::default();
        model.download_notice = Some(DownloadNotice {
            episode_id: "e1".to_string(),
            message: "Couldn't remove the download: disk busy".to_string(),
        });

        // A *different* episode's persist succeeds.
        let _ = app.update(
            Event::DownloadStatePersisted {
                episode_id: "e2".to_string(),
                result: Box::new(StorageResult::Success),
            },
            &mut model,
        );

        let notice = model.download_notice.as_ref().expect("notice retained");
        assert_eq!(notice.episode_id, "e1");
        assert_eq!(notice.message, "Couldn't remove the download: disk busy");
    }

    #[test]
    fn a_fresh_download_action_clears_a_stale_download_notice() {
        let app = Pollux;
        let mut model = Model::default();
        load_episodes(&app, &mut model, vec![make_episode("e1", "Ep", Some(1))]);
        model.download_notice = Some(DownloadNotice {
            episode_id: "e1".to_string(),
            message: "Couldn't remove the download: busy".to_string(),
        });

        let _ = app.update(Event::DownloadEpisode("e1".to_string()), &mut model);

        assert!(model.download_notice.is_none());
    }

    #[test]
    fn delete_failure_surfaces_as_a_notice_and_keeps_the_row() {
        let app = Pollux;
        let mut model = Model::default();
        let mut downloaded = make_episode("e1", "Ep", Some(1));
        downloaded.download_status = DownloadStatus::Downloaded;
        downloaded.local_path = Some("Downloads/e1.mp3".to_string());
        load_episodes(&app, &mut model, vec![downloaded]);
        let _ = app.update(Event::DeleteDownload("e1".to_string()), &mut model);

        // The shell couldn't remove the file.
        let _ = app.update(
            Event::DownloadDeleted {
                episode_id: "e1".to_string(),
                result: Box::new(DownloadResult::Error("disk busy".to_string())),
            },
            &mut model,
        );

        // The row stays Downloaded (the file is presumably still there) and the
        // failure lands in the non-blocking notice (tagged with e1), not the list error.
        assert_eq!(
            model_episode_status(&model, "e1"),
            DownloadStatus::Downloaded
        );
        let notice = model.download_notice.as_ref().expect("a notice");
        assert_eq!(notice.episode_id, "e1");
        assert!(notice.message.contains("disk busy"));
        assert!(model.detail_error.is_none());
    }

    #[test]
    fn selecting_a_subscription_clears_a_stale_download_notice() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions = vec![make_subscription("sub-1", "Feed")];
        model.download_notice = Some(DownloadNotice {
            episode_id: "e1".to_string(),
            message: "Couldn't save the download's state: db locked".to_string(),
        });

        let _ = app.update(Event::SelectSubscription("sub-1".to_string()), &mut model);

        assert!(
            model.download_notice.is_none(),
            "a notice from a previous feed shouldn't carry into another"
        );
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

        // The head item (e1) is started immediately, so it's persisted only as
        // Downloading — no redundant Queued write that Downloading would supersede.
        // The item waiting behind it (e2) is normalized to Queued.
        let persisted: Vec<(&str, &DownloadStatus)> = effects
            .iter()
            .filter_map(|e| match e {
                Effect::Storage(r) => match &r.operation {
                    StorageOperation::UpdateDownloadState {
                        episode_id, status, ..
                    } => Some((episode_id.as_str(), status)),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(
            persisted,
            vec![
                ("e2", &DownloadStatus::Queued),
                ("e1", &DownloadStatus::Downloading),
            ],
            "head persists Downloading only; the rest persist Queued"
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

    // --- feed refresh ---

    fn model_with_subs(ids: &[&str]) -> Model {
        let mut model = Model::default();
        for id in ids {
            model.subscriptions.push(make_subscription(id, id));
        }
        model
    }

    fn http_ops(cmd: &mut Command<Effect, Event>) -> Vec<HttpOperation> {
        cmd.effects()
            .filter_map(|e| match e {
                Effect::Http(r) => Some(r.operation.clone()),
                _ => None,
            })
            .collect()
    }

    fn storage_ops(cmd: &mut Command<Effect, Event>) -> Vec<StorageOperation> {
        cmd.effects()
            .filter_map(|e| match e {
                Effect::Storage(r) => Some(r.operation.clone()),
                _ => None,
            })
            .collect()
    }

    fn fetched(id: &str, result: HttpResult) -> Event {
        Event::RefreshFetched {
            subscription_id: id.to_string(),
            result: Box::new(result),
        }
    }

    #[test]
    fn refresh_sends_stored_validators_and_marks_in_flight() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        model.subscriptions[0].etag = Some("\"v1\"".to_string());
        model.subscriptions[0].last_modified = Some("Wed, 01 Oct 2026 00:00:00 GMT".to_string());

        let mut cmd = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        assert_eq!(model.refreshing.as_deref(), Some("a"));
        let ops = http_ops(&mut cmd);
        assert_eq!(ops.len(), 1);
        let HttpOperation::FetchFeed {
            url,
            etag,
            last_modified,
        } = &ops[0];
        assert_eq!(url, "https://example.com/a.rss");
        assert_eq!(etag.as_deref(), Some("\"v1\""));
        assert_eq!(
            last_modified.as_deref(),
            Some("Wed, 01 Oct 2026 00:00:00 GMT")
        );
        // Refresh must not borrow the subscribe flow's loading/error state.
        assert!(!model.loading);
        assert!(model.error.is_none());
    }

    #[test]
    fn refresh_ignores_unknown_and_duplicate_requests() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);

        let mut unknown = app.update(Event::RefreshSubscription("zzz".to_string()), &mut model);
        assert!(http_ops(&mut unknown).is_empty());
        assert!(model.refreshing.is_none());

        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);
        let mut again = app.update(Event::RefreshSubscription("a".to_string()), &mut model);
        assert!(http_ops(&mut again).is_empty());
        assert!(model.refresh_queue.is_empty());
    }

    #[test]
    fn refresh_runs_serially_in_request_order() {
        let app = Pollux;
        let mut model = model_with_subs(&["a", "b"]);

        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);
        let mut second = app.update(Event::RefreshSubscription("b".to_string()), &mut model);
        assert!(http_ops(&mut second).is_empty(), "b waits behind a");
        assert_eq!(model.refresh_queue, vec!["b".to_string()]);

        let mut next = app.update(fetched("a", response(304, vec![])), &mut model);
        let ops = http_ops(&mut next);
        assert_eq!(ops.len(), 1, "finishing a starts b");
        assert_eq!(model.refreshing.as_deref(), Some("b"));
    }

    #[test]
    fn refresh_304_only_touches_timestamp_and_clears_error() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        model.subscriptions[0].last_refresh_error = Some("old".to_string());
        model.subscriptions[0].retry_after_until = Some(5);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let mut cmd = app.update(fetched("a", response(304, vec![])), &mut model);

        let sub = &model.subscriptions[0];
        assert!(sub.last_refreshed.is_some());
        assert!(sub.last_refresh_error.is_none());
        assert!(sub.retry_after_until.is_none());
        assert!(model.refreshing.is_none());
        let ops = storage_ops(&mut cmd);
        assert!(ops.iter().any(|o| matches!(
            o,
            StorageOperation::UpdateRefreshState {
                last_refreshed: Some(_),
                last_refresh_error: None,
                retry_after_until: None,
                ..
            }
        )));
        assert!(
            !ops.iter()
                .any(|o| matches!(o, StorageOperation::UpsertFeedWithEpisodes { .. })),
            "304 must not rewrite episodes"
        );
    }

    #[test]
    fn refresh_429_persists_retry_after_and_error() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);
        let before = now_unix();

        let mut cmd = app.update(
            fetched(
                "a",
                HttpResult::Response {
                    status: 429,
                    body: vec![],
                    etag: None,
                    last_modified: None,
                    retry_after_secs: Some(600),
                },
            ),
            &mut model,
        );

        let until = model.subscriptions[0].retry_after_until.expect("retry set");
        assert!(until >= before + 600 && until <= now_unix() + 600);
        assert!(model.subscriptions[0].last_refresh_error.is_some());
        assert!(storage_ops(&mut cmd).iter().any(|o| matches!(
            o,
            StorageOperation::UpdateRefreshState {
                retry_after_until: Some(_),
                last_refresh_error: Some(_),
                ..
            }
        )));
    }

    #[test]
    fn refresh_429_without_retry_after_uses_default_backoff() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);
        let before = now_unix();

        let _ = app.update(fetched("a", response(429, vec![])), &mut model);

        let until = model.subscriptions[0].retry_after_until.expect("retry set");
        assert!(until >= before + RATE_LIMIT_BACKOFF_SECS);
    }

    #[test]
    fn rate_limit_wait_honours_the_hosts_value_up_to_the_cap() {
        assert_eq!(rate_limit_wait_secs(Some(600)), 600);
        assert_eq!(rate_limit_wait_secs(Some(0)), 0);
        assert_eq!(
            rate_limit_wait_secs(Some(MAX_RETRY_AFTER_SECS as u64)),
            MAX_RETRY_AFTER_SECS,
            "exactly the cap is allowed"
        );
        assert_eq!(rate_limit_wait_secs(None), RATE_LIMIT_BACKOFF_SECS);
    }

    #[test]
    fn rate_limit_wait_is_capped() {
        let year = 365 * 24 * 3600;
        assert_eq!(
            rate_limit_wait_secs(Some(MAX_RETRY_AFTER_SECS as u64 + 1)),
            MAX_RETRY_AFTER_SECS
        );
        assert_eq!(rate_limit_wait_secs(Some(year)), MAX_RETRY_AFTER_SECS);
        // Too big even for an i64: must clamp to the cap, not fall back to the default.
        assert_eq!(rate_limit_wait_secs(Some(u64::MAX)), MAX_RETRY_AFTER_SECS);
    }

    #[test]
    fn a_huge_retry_after_only_silences_auto_refresh_for_the_cap() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);
        let before = now_unix();

        let _ = app.update(
            fetched(
                "a",
                HttpResult::Response {
                    status: 429,
                    body: vec![],
                    etag: None,
                    last_modified: None,
                    retry_after_secs: Some(365 * 24 * 3600),
                },
            ),
            &mut model,
        );

        let until = model.subscriptions[0].retry_after_until.expect("retry set");
        assert!(until >= before + MAX_RETRY_AFTER_SECS);
        assert!(
            until <= now_unix() + MAX_RETRY_AFTER_SECS,
            "a year-long Retry-After must not mute the feed for a year"
        );
    }

    #[test]
    fn refresh_failure_records_error_without_touching_library_error() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let _ = app.update(
            fetched("a", HttpResult::Error("offline".to_string())),
            &mut model,
        );

        assert_eq!(
            model.subscriptions[0].last_refresh_error.as_deref(),
            Some("offline")
        );
        assert!(model.error.is_none());
        assert!(model.refreshing.is_none());
        assert_eq!(
            app.view(&model).library.subscriptions[0]
                .refresh_error
                .as_deref(),
            Some("offline")
        );
    }

    #[test]
    fn refresh_200_upserts_with_validators_from_response() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let mut cmd = app.update(
            fetched(
                "a",
                HttpResult::Response {
                    status: 200,
                    body: MINIMAL_RSS.as_bytes().to_vec(),
                    etag: Some("\"v2\"".to_string()),
                    last_modified: Some("Thu, 02 Oct 2026 00:00:00 GMT".to_string()),
                    retry_after_secs: None,
                },
            ),
            &mut model,
        );

        let ops = storage_ops(&mut cmd);
        assert!(ops.iter().any(|o| matches!(
            o,
            StorageOperation::UpsertFeedWithEpisodes { subscription, .. }
                if subscription.feed_url == "https://example.com/a.rss"
                    && subscription.etag.as_deref() == Some("\"v2\"")
                    && subscription.last_modified.as_deref()
                        == Some("Thu, 02 Oct 2026 00:00:00 GMT")
        )));
        // Still in flight until the save resolves.
        assert_eq!(model.refreshing.as_deref(), Some("a"));
    }

    /// A feed that parses fine but carries no channel title, description or image, like a
    /// degraded or partial response.
    const BARE_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
<channel>
  <link>https://example.com</link>
  <item>
    <title>Episode 1</title>
    <guid>episode-1-guid</guid>
    <enclosure url="https://example.com/ep1.mp3" type="audio/mpeg" length="1"/>
  </item>
</channel>
</rss>"#;

    fn known_subscription(id: &str) -> Subscription {
        let mut s = make_subscription(id, "Real Title");
        s.artwork_url = Some("https://example.com/art.png".to_string());
        s.description = Some("About the show".to_string());
        s
    }

    /// The subscription carried by the `UpsertFeedWithEpisodes` a command emitted.
    fn upserted_subscription(cmd: &mut Command<Effect, Event>) -> Subscription {
        storage_ops(cmd)
            .into_iter()
            .find_map(|o| match o {
                StorageOperation::UpsertFeedWithEpisodes { subscription, .. } => Some(subscription),
                _ => None,
            })
            .expect("expected an UpsertFeedWithEpisodes effect")
    }

    #[test]
    fn a_refresh_response_without_metadata_keeps_the_stored_title_artwork_and_description() {
        let app = Pollux;
        let mut model = model_with_subs(&[]);
        model.subscriptions.push(known_subscription("a"));
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let mut cmd = app.update(
            fetched("a", response(200, BARE_RSS.as_bytes().to_vec())),
            &mut model,
        );

        let saved = upserted_subscription(&mut cmd);
        assert_eq!(saved.title, "Real Title", "not renamed to the feed URL");
        assert_eq!(
            saved.artwork_url.as_deref(),
            Some("https://example.com/art.png")
        );
        assert_eq!(saved.description.as_deref(), Some("About the show"));
    }

    #[test]
    fn a_refresh_response_with_new_metadata_replaces_it_field_by_field() {
        let app = Pollux;
        let mut model = model_with_subs(&[]);
        model.subscriptions.push(known_subscription("a"));
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        // MINIMAL_RSS has a title and description but no image.
        let mut cmd = app.update(
            fetched("a", response(200, MINIMAL_RSS.as_bytes().to_vec())),
            &mut model,
        );

        let saved = upserted_subscription(&mut cmd);
        assert_eq!(
            saved.title, "Test Podcast",
            "the publisher's new title wins"
        );
        assert_eq!(saved.description.as_deref(), Some("A test feed"));
        assert_eq!(
            saved.artwork_url.as_deref(),
            Some("https://example.com/art.png"),
            "no image in the response, so the known one stays"
        );
    }

    #[test]
    fn re_adding_an_existing_feed_keeps_its_metadata_too() {
        let app = Pollux;
        let mut model = model_with_subs(&[]);
        model.subscriptions.push(known_subscription("a"));
        model.loading = true;

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/a.rss".to_string(),
                result: Box::new(response(200, BARE_RSS.as_bytes().to_vec())),
            },
            &mut model,
        );

        let saved = upserted_subscription(&mut cmd);
        assert_eq!(saved.title, "Real Title");
        assert_eq!(
            saved.artwork_url.as_deref(),
            Some("https://example.com/art.png")
        );
    }

    #[test]
    fn a_brand_new_feed_without_a_title_still_falls_back_to_its_url() {
        let app = Pollux;
        let mut model = Model {
            loading: true,
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/new.rss".to_string(),
                result: Box::new(response(200, BARE_RSS.as_bytes().to_vec())),
            },
            &mut model,
        );

        let saved = upserted_subscription(&mut cmd);
        assert_eq!(saved.title, "https://example.com/new.rss");
        assert!(saved.artwork_url.is_none());
    }

    #[test]
    fn refresh_saved_reloads_episodes_of_the_open_feed() {
        let app = Pollux;
        let mut model = model_with_subs(&["a"]);
        model.selected_subscription = Some(model.subscriptions[0].clone());
        model.episodes = vec![make_episode("e1", "Old", Some(1))];
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let mut saved = make_subscription("a", "a");
        saved.title = "Renamed".to_string();
        let mut cmd = app.update(
            Event::RefreshSaved {
                subscription_id: "a".to_string(),
                result: Box::new(StorageResult::Subscription(saved)),
            },
            &mut model,
        );

        assert!(model.refreshing.is_none());
        assert_eq!(model.subscriptions[0].title, "Renamed");
        assert_eq!(
            model
                .selected_subscription
                .as_ref()
                .map(|s| s.title.as_str()),
            Some("Renamed")
        );
        assert!(!model.detail_loading, "no spinner flash on refresh");
        assert!(storage_ops(&mut cmd).iter().any(|o| matches!(
            o,
            StorageOperation::ListEpisodesBySubscription { subscription_id } if subscription_id == "a"
        )));
    }

    #[test]
    fn refresh_saved_for_other_feed_does_not_reload_episodes() {
        let app = Pollux;
        let mut model = model_with_subs(&["a", "b"]);
        model.selected_subscription = Some(model.subscriptions[1].clone());
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let mut cmd = app.update(
            Event::RefreshSaved {
                subscription_id: "a".to_string(),
                result: Box::new(StorageResult::Subscription(make_subscription("a", "a"))),
            },
            &mut model,
        );

        assert!(storage_ops(&mut cmd).is_empty());
    }

    #[test]
    fn refresh_state_is_visible_in_the_view() {
        let app = Pollux;
        let mut model = model_with_subs(&["a", "b"]);
        model.selected_subscription = Some(model.subscriptions[1].clone());
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);
        let _ = app.update(Event::RefreshSubscription("b".to_string()), &mut model);

        let view = app.view(&model);
        assert!(view.library.subscriptions.iter().all(|s| s.refreshing));
        assert!(
            view.subscription_detail.refreshing,
            "queued counts as refreshing"
        );
    }

    #[test]
    fn refresh_all_queues_every_feed_in_library_order_and_starts_the_first() {
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions.push(make_subscription("z", "Zed"));
        model.subscriptions.push(make_subscription("a", "alpha"));
        model.subscriptions.push(make_subscription("m", "Middle"));

        let mut cmd = app.update(Event::RefreshAll, &mut model);

        assert_eq!(http_ops(&mut cmd).len(), 1, "serial: only one fetch starts");
        assert_eq!(model.refreshing.as_deref(), Some("a"));
        assert_eq!(model.refresh_queue, vec!["m".to_string(), "z".to_string()]);
        assert!(app.view(&model).library.refreshing);
    }

    #[test]
    fn feeds_with_the_same_title_refresh_in_a_stable_order() {
        // The id breaks title ties, so the order doesn't depend on storage order.
        let app = Pollux;
        let mut model = Model::default();
        model.subscriptions.push(make_subscription("b", "Same"));
        model.subscriptions.push(make_subscription("a", "same"));
        model.subscriptions.push(make_subscription("c", "SAME"));

        let _ = app.update(Event::RefreshAll, &mut model);

        // Same title ignoring case, so it falls to the ids: a, b, c.
        assert_eq!(model.refreshing.as_deref(), Some("a"));
        assert_eq!(model.refresh_queue, vec!["b".to_string(), "c".to_string()]);
    }

    #[test]
    fn auto_refresh_queues_due_feeds_in_the_same_order_as_refresh_all() {
        let app = Pollux;
        let mut model = loaded_model(vec![
            sub_refreshed("z", Some(24 * HOUR)),
            sub_refreshed("a", Some(24 * HOUR)),
            sub_refreshed("m", Some(HOUR)), // fresh: not due
        ]);
        for s in &mut model.subscriptions {
            s.title = s.id.clone();
        }

        let _ = app.update(Event::RefreshStale, &mut model);

        assert_eq!(model.refreshing.as_deref(), Some("a"));
        assert_eq!(model.refresh_queue, vec!["z".to_string()]);
    }

    #[test]
    fn refresh_all_does_not_double_queue_a_feed_already_refreshing() {
        let app = Pollux;
        let mut model = model_with_subs(&["a", "b"]);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let mut cmd = app.update(Event::RefreshAll, &mut model);

        assert!(http_ops(&mut cmd).is_empty());
        assert_eq!(model.refresh_queue, vec!["b".to_string()]);
    }

    #[test]
    fn refresh_all_continues_past_a_failing_feed_and_then_goes_idle() {
        let app = Pollux;
        let mut model = model_with_subs(&["a", "b"]);
        let _ = app.update(Event::RefreshAll, &mut model);

        let mut next = app.update(
            fetched("a", HttpResult::Error("offline".to_string())),
            &mut model,
        );
        assert_eq!(http_ops(&mut next).len(), 1, "b still runs after a fails");
        let _ = app.update(fetched("b", response(304, vec![])), &mut model);

        assert!(!app.view(&model).library.refreshing);
        assert_eq!(
            model.subscriptions[0].last_refresh_error.as_deref(),
            Some("offline")
        );
        assert!(model.subscriptions[1].last_refresh_error.is_none());
    }

    #[test]
    fn refresh_all_with_no_subscriptions_is_a_noop() {
        let app = Pollux;
        let mut model = Model::default();
        let mut cmd = app.update(Event::RefreshAll, &mut model);
        assert!(http_ops(&mut cmd).is_empty());
        assert!(!app.view(&model).library.refreshing);
    }

    // --- foreground auto-refresh ---

    fn loaded_model(subs: Vec<Subscription>) -> Model {
        Model {
            subscriptions: subs,
            subscriptions_loaded: true,
            ..Model::default()
        }
    }

    fn sub_refreshed(id: &str, age_secs: Option<i64>) -> Subscription {
        let mut s = make_subscription(id, id);
        s.last_refreshed = age_secs.map(|a| now_unix() - a);
        s
    }

    const HOUR: i64 = 3600;

    #[test]
    fn refresh_stale_queues_only_feeds_past_the_interval() {
        let app = Pollux;
        let mut model = loaded_model(vec![
            sub_refreshed("fresh", Some(HOUR)),
            sub_refreshed("old", Some(13 * HOUR)),
            sub_refreshed("never", None),
        ]);

        let mut cmd = app.update(Event::RefreshStale, &mut model);

        assert_eq!(http_ops(&mut cmd).len(), 1);
        let mut pending = model.refresh_queue.clone();
        pending.extend(model.refreshing.clone());
        pending.sort();
        assert_eq!(pending, vec!["never".to_string(), "old".to_string()]);
    }

    fn due_sub(last_refreshed: Option<i64>, retry_after_until: Option<i64>) -> Subscription {
        let mut s = make_subscription("a", "a");
        s.last_refreshed = last_refreshed;
        s.retry_after_until = retry_after_until;
        s
    }

    // Fixed "now" so these don't depend on the real clock.
    const NOW: i64 = 1_800_000_000;

    #[test]
    fn a_last_refreshed_in_the_future_counts_as_stale() {
        // Stamped while the clock was ahead, then the clock was corrected: without this
        // the feed would stay not-due until real time reached the stored value.
        assert!(is_due_for_auto_refresh(
            &due_sub(Some(NOW + 365 * 24 * 3600), None),
            NOW
        ));
        assert!(is_due_for_auto_refresh(&due_sub(Some(NOW + 1), None), NOW));
    }

    #[test]
    fn a_recent_last_refreshed_is_still_not_due() {
        assert!(!is_due_for_auto_refresh(&due_sub(Some(NOW), None), NOW));
        assert!(!is_due_for_auto_refresh(
            &due_sub(Some(NOW - HOUR), None),
            NOW
        ));
        assert!(!is_due_for_auto_refresh(
            &due_sub(Some(NOW - 12 * HOUR + 1), None),
            NOW
        ));
        assert!(is_due_for_auto_refresh(
            &due_sub(Some(NOW - 12 * HOUR), None),
            NOW
        ));
        assert!(is_due_for_auto_refresh(&due_sub(None, None), NOW));
    }

    #[test]
    fn a_retry_after_further_out_than_the_cap_is_ignored_as_skew() {
        let stale = Some(NOW - 24 * HOUR);
        // Wrote a year of backoff under a fast clock; now `now` is correct.
        assert!(is_due_for_auto_refresh(
            &due_sub(stale, Some(NOW + 365 * 24 * 3600)),
            NOW
        ));
        assert!(is_due_for_auto_refresh(
            &due_sub(stale, Some(NOW + MAX_RETRY_AFTER_SECS + 1)),
            NOW
        ));
    }

    #[test]
    fn a_legitimate_backoff_is_still_honoured_up_to_the_cap() {
        let stale = Some(NOW - 24 * HOUR);
        assert!(!is_due_for_auto_refresh(
            &due_sub(stale, Some(NOW + 600)),
            NOW
        ));
        assert!(
            !is_due_for_auto_refresh(&due_sub(stale, Some(NOW + MAX_RETRY_AFTER_SECS)), NOW),
            "exactly the cap is the longest value we ever write"
        );
        // Expired or exactly now: no longer backing off.
        assert!(is_due_for_auto_refresh(&due_sub(stale, Some(NOW)), NOW));
        assert!(is_due_for_auto_refresh(&due_sub(stale, Some(NOW - 1)), NOW));
    }

    #[test]
    fn extreme_timestamps_do_not_overflow() {
        assert!(is_due_for_auto_refresh(&due_sub(Some(i64::MIN), None), NOW));
        assert!(is_due_for_auto_refresh(&due_sub(Some(i64::MAX), None), NOW));
        assert!(is_due_for_auto_refresh(
            &due_sub(Some(NOW - 24 * HOUR), Some(i64::MAX)),
            NOW
        ));
    }

    #[test]
    fn auto_refresh_recovers_a_feed_stamped_in_the_future() {
        let app = Pollux;
        let mut skewed = sub_refreshed("a", None);
        skewed.last_refreshed = Some(now_unix() + 365 * 24 * 3600);
        skewed.retry_after_until = Some(now_unix() + 365 * 24 * 3600);
        let mut model = loaded_model(vec![skewed]);

        let mut cmd = app.update(Event::RefreshStale, &mut model);

        assert_eq!(http_ops(&mut cmd).len(), 1, "the skewed feed is refreshed");

        // A 304 stamps a correct time, so it isn't stuck again afterwards.
        let _ = app.update(fetched("a", response(304, vec![])), &mut model);
        let sub = &model.subscriptions[0];
        assert!(sub.last_refreshed <= Some(now_unix()));
        assert!(sub.retry_after_until.is_none());
        let mut again = app.update(Event::RefreshStale, &mut model);
        assert!(http_ops(&mut again).is_empty(), "now properly fresh");
    }

    #[test]
    fn refresh_stale_respects_the_twelve_hour_boundary() {
        let app = Pollux;
        let mut model = loaded_model(vec![
            sub_refreshed("just-under", Some(12 * HOUR - 60)),
            sub_refreshed("exactly", Some(12 * HOUR)),
        ]);

        let _ = app.update(Event::RefreshStale, &mut model);

        assert_eq!(model.refreshing.as_deref(), Some("exactly"));
        assert!(model.refresh_queue.is_empty());
    }

    #[test]
    fn refresh_stale_skips_feeds_still_backing_off() {
        let app = Pollux;
        let mut limited = sub_refreshed("limited", Some(24 * HOUR));
        limited.retry_after_until = Some(now_unix() + 600);
        let mut expired = sub_refreshed("expired", Some(24 * HOUR));
        expired.retry_after_until = Some(now_unix() - 600);
        let mut model = loaded_model(vec![limited, expired]);

        let _ = app.update(Event::RefreshStale, &mut model);

        assert_eq!(model.refreshing.as_deref(), Some("expired"));
        assert!(model.refresh_queue.is_empty());
    }

    #[test]
    fn manual_refresh_ignores_backoff() {
        let app = Pollux;
        let mut limited = sub_refreshed("limited", Some(HOUR));
        limited.retry_after_until = Some(now_unix() + 600);
        let mut model = loaded_model(vec![limited]);

        let mut cmd = app.update(
            Event::RefreshSubscription("limited".to_string()),
            &mut model,
        );

        assert_eq!(http_ops(&mut cmd).len(), 1, "an explicit request wins");
    }

    #[test]
    fn refresh_stale_before_the_library_loads_is_held_then_honoured() {
        let app = Pollux;
        let mut model = Model::default();
        let _ = app.update(Event::Started, &mut model);

        let mut early = app.update(Event::RefreshStale, &mut model);
        assert!(http_ops(&mut early).is_empty());
        assert!(
            !storage_ops(&mut early)
                .iter()
                .any(|o| matches!(o, StorageOperation::ListSubscriptions)),
            "the launch load is still in flight, so it must not be requested twice"
        );
        assert!(model.auto_refresh_pending);

        let mut loaded = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![
                sub_refreshed("old", Some(24 * HOUR)),
            ]))),
            &mut model,
        );

        assert!(!model.auto_refresh_pending);
        assert_eq!(http_ops(&mut loaded).len(), 1);
        assert_eq!(model.refreshing.as_deref(), Some("old"));
    }

    fn started_with_a_failed_library_load() -> Model {
        let app = Pollux;
        let mut model = Model::default();
        let _ = app.update(Event::Started, &mut model);
        let _ = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Error("db locked".to_string()))),
            &mut model,
        );
        assert!(!model.subscriptions_loaded);
        assert!(!model.loading);
        model
    }

    #[test]
    fn a_failed_library_load_is_retried_on_the_next_activation_and_then_auto_refreshes() {
        let app = Pollux;
        let mut model = started_with_a_failed_library_load();

        let mut activated = app.update(Event::RefreshStale, &mut model);

        assert!(model.loading, "retrying shows the library as loading again");
        assert!(
            model.auto_refresh_pending,
            "the request is held for the retry"
        );
        assert!(
            storage_ops(&mut activated)
                .iter()
                .any(|o| matches!(o, StorageOperation::ListSubscriptions)),
            "the failed load must be retried, or nothing ever recovers it"
        );

        let mut loaded = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![
                sub_refreshed("old", Some(24 * HOUR)),
            ]))),
            &mut model,
        );

        assert!(model.subscriptions_loaded);
        assert!(model.error.is_none(), "the earlier load error is cleared");
        assert!(!model.auto_refresh_pending);
        assert_eq!(
            http_ops(&mut loaded).len(),
            1,
            "the held auto-refresh now runs"
        );
    }

    #[test]
    fn a_library_load_that_keeps_failing_is_retried_every_activation_not_stuck() {
        let app = Pollux;
        let mut model = started_with_a_failed_library_load();

        for _ in 0..3 {
            let mut activated = app.update(Event::RefreshStale, &mut model);
            assert!(
                storage_ops(&mut activated)
                    .iter()
                    .any(|o| matches!(o, StorageOperation::ListSubscriptions)),
                "each activation retries the load"
            );
            let _ = app.update(
                Event::SubscriptionsLoaded(Box::new(StorageResult::Error(
                    "still locked".to_string(),
                ))),
                &mut model,
            );
            assert!(!model.loading);
            assert!(
                model.error.is_some(),
                "the error stays visible between retries"
            );
        }
    }

    #[test]
    fn a_retry_in_flight_is_not_requested_again_by_another_activation() {
        let app = Pollux;
        let mut model = started_with_a_failed_library_load();
        let _ = app.update(Event::RefreshStale, &mut model);

        let mut again = app.update(Event::RefreshStale, &mut model);

        assert!(storage_ops(&mut again).is_empty(), "one load at a time");
    }

    #[test]
    fn cancelling_before_a_retried_library_load_lands_drops_the_held_auto_refresh() {
        let app = Pollux;
        let mut model = started_with_a_failed_library_load();
        let _ = app.update(Event::RefreshStale, &mut model);
        let _ = app.update(Event::CancelRefresh, &mut model);

        let mut loaded = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![
                sub_refreshed("old", Some(24 * HOUR)),
            ]))),
            &mut model,
        );

        assert!(
            model.subscriptions_loaded,
            "the library itself still recovers"
        );
        assert!(
            http_ops(&mut loaded).is_empty(),
            "but the cancelled refresh doesn't run"
        );
    }

    #[test]
    fn a_later_library_reload_does_not_retrigger_auto_refresh() {
        let app = Pollux;
        let mut model = loaded_model(vec![]);
        let mut cmd = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![
                sub_refreshed("old", Some(24 * HOUR)),
            ]))),
            &mut model,
        );
        assert!(http_ops(&mut cmd).is_empty());
    }

    #[test]
    fn refresh_stale_does_not_double_queue_feeds_already_pending() {
        let app = Pollux;
        let mut model = loaded_model(vec![sub_refreshed("old", Some(24 * HOUR))]);
        let _ = app.update(Event::RefreshStale, &mut model);

        let mut again = app.update(Event::RefreshStale, &mut model);

        assert!(http_ops(&mut again).is_empty());
        assert!(model.refresh_queue.is_empty());
    }

    #[test]
    fn a_failed_refresh_backs_off_so_auto_refresh_skips_it() {
        let app = Pollux;
        let mut model = loaded_model(vec![sub_refreshed("old", Some(24 * HOUR))]);
        let _ = app.update(Event::RefreshStale, &mut model);
        let _ = app.update(
            fetched("old", HttpResult::Error("offline".to_string())),
            &mut model,
        );

        let until = model.subscriptions[0]
            .retry_after_until
            .expect("backoff set");
        assert!(until > now_unix());

        let mut next = app.update(Event::RefreshStale, &mut model);
        assert!(
            http_ops(&mut next).is_empty(),
            "a broken feed isn't retried on every foreground"
        );
    }

    #[test]
    fn every_kind_of_refresh_failure_backs_off_auto_refresh() {
        // Each failure path must set the backoff itself; carrying forward a missing or
        // expired `retry_after_until` would leave the feed due on every foreground.
        let failures: Vec<(&str, Event)> = vec![
            (
                "network error",
                fetched("a", HttpResult::Error("offline".to_string())),
            ),
            (
                "malformed 200",
                fetched("a", response(200, b"this is not xml".to_vec())),
            ),
            ("HTTP 404", fetched("a", response(404, vec![]))),
            ("HTTP 500", fetched("a", response(500, vec![]))),
            (
                "save error",
                Event::RefreshSaved {
                    subscription_id: "a".to_string(),
                    result: Box::new(StorageResult::Error("disk full".to_string())),
                },
            ),
            (
                "unexpected save result",
                Event::RefreshSaved {
                    subscription_id: "a".to_string(),
                    result: Box::new(StorageResult::Success),
                },
            ),
        ];

        for (name, failure) in failures {
            let app = Pollux;
            // A feed whose backoff already expired: the failure case that used to slip
            // through by preserving the old (expired) value.
            let mut stale = sub_refreshed("a", Some(24 * HOUR));
            stale.retry_after_until = Some(now_unix() - 600);
            let mut model = loaded_model(vec![stale]);
            let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

            let _ = app.update(failure, &mut model);

            let sub = &model.subscriptions[0];
            assert!(sub.last_refresh_error.is_some(), "{name}: error recorded");
            let until = sub
                .retry_after_until
                .unwrap_or_else(|| panic!("{name}: backoff not set"));
            assert!(
                until >= now_unix() + FAILURE_BACKOFF_SECS - 5,
                "{name}: backoff should be about {FAILURE_BACKOFF_SECS}s out"
            );
            assert!(model.refreshing.is_none(), "{name}: refresh finished");

            let mut next = app.update(Event::RefreshStale, &mut model);
            assert!(
                http_ops(&mut next).is_empty(),
                "{name}: auto-refresh must skip the feed while it backs off"
            );
        }
    }

    #[test]
    fn cancel_refresh_drops_the_queue_but_lets_the_active_fetch_finish() {
        let app = Pollux;
        let mut model = loaded_model(vec![
            sub_refreshed("a", Some(24 * HOUR)),
            sub_refreshed("b", Some(24 * HOUR)),
            sub_refreshed("c", Some(24 * HOUR)),
        ]);
        let _ = app.update(Event::RefreshStale, &mut model);
        assert_eq!(model.refreshing.as_deref(), Some("a"));
        assert_eq!(model.refresh_queue.len(), 2);

        let _ = app.update(Event::CancelRefresh, &mut model);

        assert!(model.refresh_queue.is_empty());
        assert_eq!(
            model.refreshing.as_deref(),
            Some("a"),
            "in-flight is untouched"
        );
        assert!(
            app.view(&model).library.refreshing,
            "still busy until it lands"
        );

        // The in-flight fetch resolves normally and records its outcome, but nothing
        // further starts.
        let mut done = app.update(fetched("a", response(304, vec![])), &mut model);
        assert!(http_ops(&mut done).is_empty(), "no next feed after cancel");
        assert!(model.refreshing.is_none());
        assert!(
            model.subscriptions[0].last_refreshed > Some(now_unix() - 5),
            "the in-flight fetch's 304 still records its timestamp"
        );
        assert!(!app.view(&model).library.refreshing);
        assert!(
            model.subscriptions[1].last_refreshed < Some(now_unix() - 23 * HOUR),
            "cancelled feeds are left for a later wake-up"
        );
    }

    #[test]
    fn cancel_refresh_drops_an_auto_refresh_held_for_the_library_load() {
        let app = Pollux;
        let mut model = Model::default();
        let _ = app.update(Event::RefreshStale, &mut model);
        assert!(model.auto_refresh_pending);

        let _ = app.update(Event::CancelRefresh, &mut model);
        let mut loaded = app.update(
            Event::SubscriptionsLoaded(Box::new(StorageResult::Subscriptions(vec![
                sub_refreshed("old", Some(24 * HOUR)),
            ]))),
            &mut model,
        );

        assert!(!model.auto_refresh_pending);
        assert!(
            http_ops(&mut loaded).is_empty(),
            "cancelled before it could run"
        );
    }

    #[test]
    fn cancel_refresh_when_idle_is_harmless_and_a_later_refresh_still_works() {
        let app = Pollux;
        let mut model = loaded_model(vec![sub_refreshed("a", Some(24 * HOUR))]);

        let mut cmd = app.update(Event::CancelRefresh, &mut model);
        assert!(http_ops(&mut cmd).is_empty());

        let mut next = app.update(Event::RefreshStale, &mut model);
        assert_eq!(http_ops(&mut next).len(), 1, "cancel isn't sticky");
    }

    #[test]
    fn an_unreachable_device_records_the_error_without_backing_the_feed_off() {
        let app = Pollux;
        let mut model = loaded_model(vec![sub_refreshed("a", Some(24 * HOUR))]);
        let _ = app.update(Event::RefreshStale, &mut model);

        let mut cmd = app.update(
            fetched("a", HttpResult::Unreachable("offline".to_string())),
            &mut model,
        );

        let sub = &model.subscriptions[0];
        assert_eq!(sub.last_refresh_error.as_deref(), Some("offline"));
        assert!(
            sub.retry_after_until.is_none(),
            "no backoff for a device-level failure"
        );
        assert!(storage_ops(&mut cmd).iter().any(|o| matches!(
            o,
            StorageOperation::UpdateRefreshState {
                last_refresh_error: Some(_),
                retry_after_until: None,
                ..
            }
        )));
    }

    #[test]
    fn feeds_refresh_again_as_soon_as_connectivity_returns() {
        // The scenario behind the finding: open the app offline, every feed fails,
        // reconnect and foreground again. Nothing should be sitting in a backoff.
        let app = Pollux;
        let mut model = loaded_model(vec![
            sub_refreshed("a", Some(24 * HOUR)),
            sub_refreshed("b", Some(24 * HOUR)),
        ]);
        let _ = app.update(Event::RefreshStale, &mut model);
        let _ = app.update(
            fetched("a", HttpResult::Unreachable("offline".to_string())),
            &mut model,
        );
        let _ = app.update(
            fetched("b", HttpResult::Unreachable("offline".to_string())),
            &mut model,
        );
        assert!(model.refreshing.is_none());

        let mut back_online = app.update(Event::RefreshStale, &mut model);

        assert_eq!(http_ops(&mut back_online).len(), 1, "feeds are due again");
        assert_eq!(model.refreshing.as_deref(), Some("a"));
        assert_eq!(model.refresh_queue, vec!["b".to_string()]);
    }

    #[test]
    fn an_unreachable_device_keeps_a_backoff_the_host_earned() {
        // A live backoff from a real host failure (e.g. a 429) is not the device's
        // doing, so an offline attempt must not clear or shorten it.
        let app = Pollux;
        let until = now_unix() + 600;
        let mut limited = sub_refreshed("a", Some(24 * HOUR));
        limited.retry_after_until = Some(until);
        let mut model = loaded_model(vec![limited]);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let _ = app.update(
            fetched("a", HttpResult::Unreachable("offline".to_string())),
            &mut model,
        );

        assert_eq!(model.subscriptions[0].retry_after_until, Some(until));
    }

    #[test]
    fn the_subscribe_flow_treats_an_unreachable_device_like_any_fetch_error() {
        let app = Pollux;
        let mut model = Model {
            loading: true,
            ..Model::default()
        };

        let mut cmd = app.update(
            Event::FeedFetched {
                url: "https://example.com/feed.rss".to_string(),
                result: Box::new(HttpResult::Unreachable("offline".to_string())),
            },
            &mut model,
        );

        assert!(!model.loading);
        assert_eq!(model.error.as_deref(), Some("offline"));
        cmd.expect_one_effect().expect_render();
    }

    #[test]
    fn a_successful_refresh_clears_the_backoff() {
        let app = Pollux;
        let mut model = loaded_model(vec![sub_refreshed("a", Some(24 * HOUR))]);
        model.subscriptions[0].retry_after_until = Some(now_unix() - 1);
        let _ = app.update(Event::RefreshSubscription("a".to_string()), &mut model);

        let _ = app.update(fetched("a", response(304, vec![])), &mut model);

        assert!(model.subscriptions[0].retry_after_until.is_none());
    }
}
