//! Player policy. The shell owns the audio engine (see `capabilities::player`); this
//! module decides what it should do: which source to load, where to start, how far a
//! skip goes, when position is persisted, and when an episode counts as played.
//!
//! Engine-originated news arrives as ordinary events (`PlayerTick`, `PlayerEnded`, …)
//! rather than request results, mirroring how download progress is reported.
//!
//! # Sessions
//!
//! Every `Load` gets a fresh *session* id, and the shell tags everything the engine
//! reports with the session of the item that produced it. The core ignores news from
//! any session but the current one, so a late tick, end-of-file or failure from a
//! replaced item (a source swap, a retry, or a different episode) can't touch the
//! current one.
//!
//! # When an episode counts as played
//!
//! The decision is made at the moments the listener *leaves* the episode, never as a
//! side effect of saving progress:
//!
//! - the engine reaches the end of the file (natural completion);
//! - they pause (explicitly) within the played tolerance of the end;
//! - they start another episode while within the tolerance of the end.
//!
//! Periodic and background saves only ever record "in progress" and a position, so
//! listening on through the last seconds with the screen locked is never cut short or
//! reset. A system interruption (a call, unplugged headphones) is not leaving: it
//! pauses and keeps the place so playback can resume to the end.

use crux_core::{render::render, Command};

use crate::app::{enqueue_download, reset_to_not_downloaded, Event, DESCRIPTION_PREVIEW_CHARS};
use crate::capabilities::player::{MediaSource, PlayerOperation, PlayerResult};
use crate::capabilities::storage::{StorageOperation, StorageResult};
use crate::defaults::{
    PLAYED_TOLERANCE_SECS, POSITION_CHECKPOINT_SECS, RESUME_REWIND_SECS, SKIP_BACKWARD_SECS,
    SKIP_FORWARD_SECS,
};
use crate::domain::{DownloadStatus, Episode, EpisodeSource, PlaybackStatus};
use crate::effect::Effect;
use crate::html::strip_html_preview;
use crate::model::{ActivePlayback, Model};
use crate::view_model::PlayerView;

#[cfg(test)]
mod download_recovery_tests;
#[cfg(test)]
mod played_tests;
#[cfg(test)]
mod tests;

type Cmd = Command<Effect, Event>;

/// Where an episode should play from: its downloaded file when we have one, else the
/// network.
fn source_for(episode: &Episode) -> MediaSource {
    match (&episode.download_status, &episode.local_path) {
        (DownloadStatus::Downloaded, Some(path)) => MediaSource::Local {
            local_path: path.clone(),
        },
        _ => MediaSource::Stream {
            url: episode.enclosure_url.clone(),
        },
    }
}

/// Issues an engine operation. Its resolution comes back tagged with `session`, so an
/// error from a replaced item is dropped rather than blamed on the current one.
fn player_op(session: u32, op: PlayerOperation) -> Cmd {
    Command::request_from_shell(op).then_send(move |r| Event::PlayerResponded {
        session,
        result: Box::new(r),
    })
}

/// Hands out the next session id. Ids only ever increase, across episodes.
fn next_session(model: &mut Model) -> u32 {
    model.player_sessions += 1;
    model.player_sessions
}

/// Persists an episode's playback state. Best-effort: a failed write only costs
/// durability (the next checkpoint retries), so the result is dropped.
fn persist_playback(episode_id: &str, status: PlaybackStatus, position_secs: Option<u32>) -> Cmd {
    Command::request_from_shell(StorageOperation::UpdatePlaybackStatus {
        episode_id: episode_id.to_string(),
        status,
        position_secs,
    })
    .then_send(|r| Event::PlaybackPersisted(Box::new(r)))
}

fn save_play_context(episode_id: &str, source: &EpisodeSource) -> Cmd {
    Command::request_from_shell(StorageOperation::SavePlayContext {
        episode_id: episode_id.to_string(),
        source: source.clone(),
    })
    .then_send(|r| Event::PlaybackPersisted(Box::new(r)))
}

fn clear_play_context() -> Cmd {
    Command::request_from_shell(StorageOperation::ClearPlayContext)
        .then_send(|r| Event::PlaybackPersisted(Box::new(r)))
}

/// Mirrors a playback-state change onto the feed on screen, if that episode is loaded,
/// so its row updates without a reload (see the note at `SelectSubscription`).
fn sync_episode_row(
    model: &mut Model,
    episode_id: &str,
    status: &PlaybackStatus,
    position_secs: Option<u32>,
) {
    if let Some(row) = model.episodes.iter_mut().find(|e| e.id == episode_id) {
        row.playback_status = status.clone();
        row.playback_position_secs = position_secs;
    }
}

/// The duration to judge the end by: the engine's once known, else the feed's.
fn known_duration(active: &ActivePlayback) -> Option<u32> {
    active.duration_secs.or(active.episode.duration_secs)
}

/// Whether the playhead is close enough to the end to count as finished. The tolerance
/// is `PLAYED_TOLERANCE_SECS`, but never more than a quarter of the episode, so a short
/// clip isn't "finished" almost as soon as it starts. An unknown (or zero) duration is
/// never in the tail: only the engine reaching the end can finish such an episode.
fn in_tail(active: &ActivePlayback) -> bool {
    let Some(duration) = known_duration(active).filter(|&d| d > 0) else {
        return false;
    };
    let tolerance = PLAYED_TOLERANCE_SECS.min(duration / 4);
    active.position_secs.saturating_add(tolerance) >= duration
}

/// Persists the current position as in-progress. Called on seek, interruption,
/// backgrounding and every `POSITION_CHECKPOINT_SECS` of playback. It never decides
/// "played": see the module docs.
fn checkpoint(model: &mut Model) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    let position = Some(active.position_secs);
    active.last_checkpoint_secs = active.position_secs;
    active.episode.playback_status = PlaybackStatus::InProgress;
    active.episode.playback_position_secs = position;
    let id = active.episode.id.clone();
    sync_episode_row(model, &id, &PlaybackStatus::InProgress, position);
    persist_playback(&id, PlaybackStatus::InProgress, position)
}

/// Finishes the active episode: marked played with its position reset, the saved
/// context cleared, the engine stopped, and the player inactive. There is no
/// auto-advance until playlists exist.
fn complete(model: &mut Model) -> Cmd {
    let Some(active) = model.active_playback.take() else {
        return Command::done();
    };
    let id = active.episode.id;
    sync_episode_row(model, &id, &PlaybackStatus::Played, Some(0));
    persist_playback(&id, PlaybackStatus::Played, Some(0))
        .and(clear_play_context())
        .and(player_op(active.session, PlayerOperation::Stop))
        .and(render())
}

/// Settles the episode about to be replaced by another: played if it was left in the
/// tail, otherwise its place is saved.
fn leave_for_another(model: &mut Model) -> Cmd {
    let Some(active) = model.active_playback.as_ref() else {
        return Command::done();
    };
    if !in_tail(active) {
        return checkpoint(model);
    }
    let id = active.episode.id.clone();
    sync_episode_row(model, &id, &PlaybackStatus::Played, Some(0));
    persist_playback(&id, PlaybackStatus::Played, Some(0))
}

/// Whether `session` is the current load's, i.e. the news is not stale.
fn is_current(model: &Model, session: u32) -> bool {
    model
        .active_playback
        .as_ref()
        .is_some_and(|a| a.session == session)
}

/// Starts playing `episode_id` from the feed on screen. Resumes in place if it is
/// already the active episode.
pub(crate) fn play_episode(model: &mut Model, episode_id: &str) -> Cmd {
    if model
        .active_playback
        .as_ref()
        .is_some_and(|a| a.episode.id == episode_id)
    {
        return play(model);
    }
    let Some(episode) = model.episodes.iter().find(|e| e.id == episode_id).cloned() else {
        return render();
    };

    // Whatever was playing keeps its place (or counts as played) before it's replaced.
    let outgoing = leave_for_another(model);

    let media = source_for(&episode);
    // Started from this episode's feed. When playlists exist the view that initiated
    // playback decides (the event will carry the source); until then a feed's episode
    // list is the only place playback starts.
    let source = EpisodeSource::Subscription {
        id: episode.subscription_id.clone(),
    };
    // A played episode restarts from the top; otherwise resume with the usual rewind.
    let saved = match episode.playback_status {
        PlaybackStatus::Played => 0,
        _ => episode.playback_position_secs.unwrap_or(0),
    };
    let start = saved.saturating_sub(RESUME_REWIND_SECS);

    let mut playing_episode = episode.clone();
    playing_episode.playback_status = PlaybackStatus::InProgress;
    playing_episode.playback_position_secs = Some(start);
    sync_episode_row(model, episode_id, &PlaybackStatus::InProgress, Some(start));
    let streaming = matches!(media, MediaSource::Stream { .. });
    let session = next_session(model);
    model.active_playback = Some(ActivePlayback {
        episode: playing_episode,
        session,
        position_secs: start,
        duration_secs: None,
        is_playing: true,
        media: media.clone(),
        source: source.clone(),
        loaded: true,
        last_checkpoint_secs: start,
        resume_after_interruption: false,
        error: None,
    });

    let mut cmd = outgoing
        .and(persist_playback(
            episode_id,
            PlaybackStatus::InProgress,
            Some(start),
        ))
        .and(save_play_context(episode_id, &source))
        .and(player_op(
            session,
            PlayerOperation::Load {
                session,
                media,
                start_secs: start,
                autoplay: true,
            },
        ));
    // Streaming now, local later: kick off the download so the copy is there next
    // time (and for the mid-play swap in `on_download_completed`).
    if streaming {
        cmd = cmd.and(enqueue_download(model, episode_id));
    }
    cmd.and(render())
}

/// Resumes the active episode, rewinding slightly for context.
pub(crate) fn play(model: &mut Model) -> Cmd {
    if model.active_playback.as_ref().is_none_or(|a| a.is_playing) {
        return Command::done();
    }
    // A restored (or errored) episode has nothing loaded, so it needs a fresh session.
    let needs_load = model.active_playback.as_ref().is_some_and(|a| !a.loaded);
    let fresh = needs_load.then(|| next_session(model));
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    active.error = None;
    active.is_playing = true;
    // The listener took over: an interruption ending later must not second-guess them.
    active.resume_after_interruption = false;
    let rewound = active.position_secs.saturating_sub(RESUME_REWIND_SECS);
    active.position_secs = rewound;
    let cmd = if let Some(session) = fresh {
        active.session = session;
        active.loaded = true;
        player_op(
            session,
            PlayerOperation::Load {
                session,
                media: active.media.clone(),
                start_secs: rewound,
                autoplay: true,
            },
        )
    } else {
        let session = active.session;
        player_op(session, PlayerOperation::Seek { secs: rewound })
            .and(player_op(session, PlayerOperation::Play))
    };
    // Still streaming with no file coming (a restored episode whose download was never
    // started, or failed): playing is the moment to try again, as starting an episode
    // does. A download already queued or underway is left alone.
    let retry =
        matches!(active.media, MediaSource::Stream { .. }).then(|| active.episode.id.clone());
    match retry {
        Some(id) => cmd.and(enqueue_download(model, &id)).and(render()),
        None => cmd.and(render()),
    }
}

/// An explicit pause (the pause button, the lock screen, headphone controls). Pausing
/// within the played tolerance of the end finishes the episode; otherwise it saves the
/// place.
pub(crate) fn pause(model: &mut Model) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if !active.is_playing {
        return Command::done();
    }
    active.is_playing = false;
    if in_tail(active) {
        // `complete` stops the engine, which pauses it.
        return complete(model);
    }
    let session = active.session;
    player_op(session, PlayerOperation::Pause)
        .and(checkpoint(model))
        .and(render())
}

/// The system paused playback (a call, Siri, unplugged headphones). That is not the
/// listener leaving: the place is saved and the episode stays active, so playback can
/// resume and play out the final seconds. When `resumable`, the end of the interruption
/// may resume it (see `on_interruption_ended`); a route change never does.
pub(crate) fn interrupt(model: &mut Model, resumable: bool) -> Cmd {
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if !active.is_playing {
        return Command::done();
    }
    active.is_playing = false;
    active.resume_after_interruption = resumable;
    let session = active.session;
    player_op(session, PlayerOperation::Pause)
        .and(checkpoint(model))
        .and(render())
}

/// An audio-session interruption ended. Playback resumes only if the system says it may
/// *and* this interruption is what paused it: the system also reports "should resume"
/// after interruptions that found playback already paused, and resuming then would start
/// audio the listener had stopped.
pub(crate) fn on_interruption_ended(model: &mut Model, should_resume: bool) -> Cmd {
    let was_interrupted = model
        .active_playback
        .as_mut()
        .is_some_and(|a| std::mem::take(&mut a.resume_after_interruption));
    if should_resume && was_interrupted {
        play(model)
    } else {
        Command::done()
    }
}

pub(crate) fn toggle(model: &mut Model) -> Cmd {
    match model.active_playback.as_ref() {
        Some(a) if a.is_playing => pause(model),
        Some(_) => play(model),
        None => Command::done(),
    }
}

/// Moves the playhead, clamped to the episode: a skip never runs past the end (so it
/// can't spill into another episode) or before the start. Seeking to the very end
/// finishes the episode.
pub(crate) fn seek_to(model: &mut Model, secs: u32) -> Cmd {
    let needs_load = model.active_playback.as_ref().is_some_and(|a| !a.loaded);
    let fresh = needs_load.then(|| next_session(model));
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    active.resume_after_interruption = false;
    let duration = known_duration(active).filter(|&d| d > 0);
    // Skipping or scrubbing to the very end is finishing: decided here rather than left
    // to the engine, which only reports the end of the file while it is playing.
    if duration.is_some_and(|d| secs >= d) {
        return complete(model);
    }
    let target = duration.map_or(secs, |d| secs.min(d));
    active.position_secs = target;
    let (session, op) = if let Some(session) = fresh {
        // Seeking a restored-but-unloaded episode loads it paused at the new spot.
        active.session = session;
        active.loaded = true;
        (
            session,
            PlayerOperation::Load {
                session,
                media: active.media.clone(),
                start_secs: target,
                autoplay: false,
            },
        )
    } else {
        (active.session, PlayerOperation::Seek { secs: target })
    };
    player_op(session, op).and(checkpoint(model)).and(render())
}

pub(crate) fn skip_forward(model: &mut Model) -> Cmd {
    let Some(position) = model.active_playback.as_ref().map(|a| a.position_secs) else {
        return Command::done();
    };
    seek_to(model, position.saturating_add(SKIP_FORWARD_SECS))
}

pub(crate) fn skip_back(model: &mut Model) -> Cmd {
    let Some(position) = model.active_playback.as_ref().map(|a| a.position_secs) else {
        return Command::done();
    };
    seek_to(model, position.saturating_sub(SKIP_BACKWARD_SECS))
}

pub(crate) fn on_tick(model: &mut Model, session: u32, position_secs: u32) -> Cmd {
    if !is_current(model, session) {
        return Command::done();
    }
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    active.position_secs = position_secs;
    let due = active.is_playing
        && position_secs.abs_diff(active.last_checkpoint_secs) >= POSITION_CHECKPOINT_SECS;
    if due {
        checkpoint(model).and(render())
    } else {
        render()
    }
}

pub(crate) fn on_duration(model: &mut Model, session: u32, duration_secs: u32) -> Cmd {
    if !is_current(model, session) {
        return Command::done();
    }
    if let Some(active) = model.active_playback.as_mut() {
        active.duration_secs = Some(duration_secs);
    }
    render()
}

/// The engine reached the end of the file: natural completion.
pub(crate) fn on_ended(model: &mut Model, session: u32) -> Cmd {
    if !is_current(model, session) {
        return Command::done();
    }
    complete(model)
}

/// Why the engine failed, as far as the shell can tell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Failure {
    /// The media itself can't be used: a missing, unreadable or undecodable file.
    MediaUnusable,
    /// Anything else (the audio session was refused, playback stopped mid-play). Says
    /// nothing about the media.
    Other,
}

/// The engine couldn't play (or lost) the item.
///
/// A downloaded file the engine reports as *unusable* (missing, unreadable, undecodable)
/// is given up on: the episode streams instead, from the same spot, and is re-downloaded.
/// Any other failure leaves playback paused with a notice and the download untouched, so
/// a transient problem (another app holding the audio session, say) can't make the app
/// discard a good multi-megabyte file and fetch it again.
pub(crate) fn on_failure(
    model: &mut Model,
    session: u32,
    message: String,
    failure: Failure,
) -> Cmd {
    if !is_current(model, session) {
        return Command::done();
    }
    let is_local = model
        .active_playback
        .as_ref()
        .is_some_and(|a| matches!(a.media, MediaSource::Local { .. }));
    if !(is_local && failure == Failure::MediaUnusable) {
        if let Some(active) = model.active_playback.as_mut() {
            active.is_playing = false;
            active.loaded = false;
            active.error = Some(message);
        }
        return checkpoint(model).and(render());
    }
    // The downloaded file is unreadable (deleted behind our back, say): forget it and
    // stream instead, carrying on from the same spot under a new session.
    let fresh = next_session(model);
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    let media = MediaSource::Stream {
        url: active.episode.enclosure_url.clone(),
    };
    let id = active.episode.id.clone();
    active.media = media.clone();
    active.session = fresh;
    active.episode.download_status = DownloadStatus::NotDownloaded;
    active.episode.local_path = None;
    active.error = None;
    active.loaded = true;
    let (position, was_playing) = (active.position_secs, active.is_playing);
    reset_to_not_downloaded(model, &id)
        .and(enqueue_download(model, &id))
        .and(player_op(
            fresh,
            PlayerOperation::Load {
                session: fresh,
                media,
                start_secs: position,
                autoplay: was_playing,
            },
        ))
        .and(render())
}

pub(crate) fn on_response(model: &mut Model, session: u32, result: PlayerResult) -> Cmd {
    match result {
        PlayerResult::Ok => Command::done(),
        PlayerResult::Error(message) => on_failure(model, session, message, Failure::Other),
        PlayerResult::MediaUnusable(message) => {
            on_failure(model, session, message, Failure::MediaUnusable)
        }
    }
}

/// A download finished: if it's the episode on the player, move playback onto the
/// local file at the current position, so streaming stops using the network.
pub(crate) fn on_download_completed(
    model: &mut Model,
    episode_id: &str,
    local_path: &str,
    size_bytes: u64,
) -> Cmd {
    let needs_swap = model.active_playback.as_ref().is_some_and(|a| {
        a.episode.id == episode_id && a.loaded && matches!(a.media, MediaSource::Stream { .. })
    });
    let swap_session = needs_swap.then(|| next_session(model));
    let Some(active) = model.active_playback.as_mut() else {
        return Command::done();
    };
    if active.episode.id != episode_id {
        return Command::done();
    }
    active.episode.download_status = DownloadStatus::Downloaded;
    active.episode.local_path = Some(local_path.to_string());
    active.episode.file_size_bytes = Some(size_bytes);
    if matches!(active.media, MediaSource::Stream { .. }) {
        active.media = MediaSource::Local {
            local_path: local_path.to_string(),
        };
    }
    let Some(session) = swap_session else {
        // Not loaded into the engine (restored, or errored): the new source is picked up
        // by the next Load. Already local: nothing to swap.
        return render();
    };
    active.session = session;
    player_op(
        session,
        PlayerOperation::Load {
            session,
            media: active.media.clone(),
            start_secs: active.position_secs,
            autoplay: active.is_playing,
        },
    )
}

/// Restores the active episode at launch, paused, so the mini-player is there
/// immediately. The engine isn't loaded until the first Play. An episode that is already
/// played is not restored (there's nothing left to resume), and its stale context is
/// cleared.
pub(crate) fn on_context_loaded(model: &mut Model, result: StorageResult) -> Cmd {
    if model.active_playback.is_some() {
        return Command::done();
    }
    let StorageResult::PlayContext { episode, source } = result else {
        return Command::done();
    };
    if episode.playback_status == PlaybackStatus::Played {
        return clear_play_context();
    }
    let position = episode.playback_position_secs.unwrap_or(0);
    model.active_playback = Some(ActivePlayback {
        media: source_for(&episode),
        source,
        episode,
        session: 0,
        position_secs: position,
        duration_secs: None,
        is_playing: false,
        loaded: false,
        last_checkpoint_secs: position,
        resume_after_interruption: false,
        error: None,
    });
    render()
}

/// Flushes position when the app leaves the foreground. Never decides "played": the
/// listener may well be still listening with the screen locked.
pub(crate) fn on_backgrounded(model: &mut Model) -> Cmd {
    checkpoint(model)
}

pub(crate) fn player_view(model: &Model) -> Option<PlayerView> {
    let active = model.active_playback.as_ref()?;
    let subscription = model
        .subscriptions
        .iter()
        .find(|s| s.id == active.episode.subscription_id);
    // The source's display name. A subscription is named by its feed.
    let source_title = match &active.source {
        EpisodeSource::Subscription { id } => model
            .subscriptions
            .iter()
            .find(|s| &s.id == id)
            .map(|s| s.title.clone())
            .unwrap_or_default(),
    };
    // A blank description is no show notes at all, so the page can be left out.
    let description = active
        .episode
        .description
        .clone()
        .filter(|d| !d.trim().is_empty());
    let description_text = description
        .as_deref()
        .map(|d| strip_html_preview(d, DESCRIPTION_PREVIEW_CHARS))
        .filter(|t| !t.is_empty());
    Some(PlayerView {
        episode_id: active.episode.id.clone(),
        episode_title: active.episode.title.clone(),
        feed_title: subscription.map(|s| s.title.clone()).unwrap_or_default(),
        source: active.source.clone(),
        source_title,
        artwork_url: active
            .episode
            .artwork_url
            .clone()
            .or_else(|| subscription.and_then(|s| s.artwork_url.clone())),
        position_secs: active.position_secs,
        duration_secs: known_duration(active),
        is_playing: active.is_playing,
        is_streaming: matches!(active.media, MediaSource::Stream { .. }),
        skip_forward_secs: SKIP_FORWARD_SECS,
        skip_back_secs: SKIP_BACKWARD_SECS,
        description,
        description_text,
        error: active.error.clone(),
    })
}
