use crux_core::capability::Operation;
use facet::Facet;
use serde::{Deserialize, Serialize};

/// Where the shell should read an episode's audio from.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum MediaSource {
    /// Stream straight from the enclosure URL.
    Stream { url: String },
    /// A downloaded file; `local_path` is relative to the app's storage root and is
    /// resolved to an absolute URL by the shell (the same convention as downloads).
    Local { local_path: String },
}

/// Commands for the shell's audio engine. The engine (AVPlayer, audio session, lock
/// screen) lives in the shell; the core owns every policy decision — when to resume,
/// how far to skip, when an episode counts as played — and only tells the engine
/// what to do. Engine-initiated news (position ticks, duration, end of file, remote
/// commands, interruptions) comes back as ordinary `Event`s, not as results here.
#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum PlayerOperation {
    /// Replace the current item with `media`, positioned at `start_secs`. Starts
    /// playing immediately when `autoplay` is set. Also used to swap a streaming
    /// episode onto its freshly downloaded local copy mid-play.
    Load {
        /// Tags everything the engine reports about this item, so the core can drop
        /// news from an item it has since replaced.
        session: u32,
        media: MediaSource,
        start_secs: u32,
        autoplay: bool,
    },
    Play,
    Pause,
    Seek {
        secs: u32,
    },
    /// Unload the current item and release the audio session.
    Stop,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum PlayerResult {
    Ok,
    Error(String),
}

impl Operation for PlayerOperation {
    type Output = PlayerResult;
}
