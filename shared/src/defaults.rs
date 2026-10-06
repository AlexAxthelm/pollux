pub const SKIP_FORWARD_SECS: u32 = 30;
pub const SKIP_BACKWARD_SECS: u32 = 15;
pub const RESUME_REWIND_SECS: u32 = 3;
pub const REFRESH_INTERVAL_HOURS: u32 = 12;
/// An episode counts as played once playback is within this many seconds of the end
/// (outros and credits shouldn't leave it in-progress forever). Will become a user
/// setting; see docs/user_stories/player/playback.md.
pub const PLAYED_TOLERANCE_SECS: u32 = 15;
/// While playing, position is persisted at least this often (in addition to pause,
/// seek, backgrounding and end), bounding what a crash can lose.
pub const POSITION_CHECKPOINT_SECS: u32 = 10;
