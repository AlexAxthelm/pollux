mod app;
pub mod capabilities;
pub mod defaults;
pub mod domain;
pub mod ffi;

pub use app::*;
pub use capabilities::storage::{StorageOperation, StorageResult};
pub use crux_core::Core;
pub use domain::*;

#[cfg(feature = "uniffi")]
const _: () = assert!(
    uniffi::check_compatible_version("0.29.4"),
    "please use uniffi v0.29.4"
);
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
