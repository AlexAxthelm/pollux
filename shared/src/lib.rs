mod app;
pub mod capabilities;
pub mod defaults;
pub mod domain;
pub mod effect;
mod feed_parser;
pub mod ffi;
mod html;
pub mod model;
pub mod theme;
pub mod view_model;

pub use app::*;
pub use capabilities::http::{HttpOperation, HttpResult};
pub use capabilities::storage::{StorageOperation, StorageResult};
pub use crux_core::Core;
pub use domain::*;
pub use theme::{Base16Palette, ThemeId, ThemeMode, ThemeView};
pub use view_model::{LibraryView, SubscriptionSummary, ViewModel};

#[cfg(feature = "uniffi")]
const _: () = assert!(
    uniffi::check_compatible_version("0.29.4"),
    "please use uniffi v0.29.4"
);
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
