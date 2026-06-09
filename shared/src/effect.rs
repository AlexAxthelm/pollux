#![allow(clippy::large_enum_variant)]
// RenderOperation is permanently 0 bytes; the size gap with Storage is expected.

use crux_core::{macros::effect, render::RenderOperation};

use crate::capabilities::storage::StorageOperation;

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Storage(StorageOperation),
}
