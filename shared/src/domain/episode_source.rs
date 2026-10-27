use facet::Facet;
use serde::{Deserialize, Serialize};

/// Where a run of episodes comes from: the thing playback was started *from*, and what
/// "From: …" names and navigates back to. See `docs/DATA_MODEL.md` — EpisodeSource.
///
/// A subscription behaves as an implicit single-feed playlist, so today it is the only
/// variant. Playlists add a `Playlist { id }` variant; the playback engine, the saved
/// play context and the player's "From:" row already carry an `EpisodeSource` rather
/// than a bare subscription id, so adding one is a change here and in the places that
/// resolve a source (its title, its episode order), not a schema or event change.
///
/// An internal abstraction, never user-facing copy. The name is a placeholder, per the
/// data model.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum EpisodeSource {
    Subscription { id: String },
}
