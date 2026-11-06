//! Shelf screens: the full page in Utilities, the floating always-on-top
//! window, the tile grid they share, and file drops on the main window.
//! The Shelf's data and file operations live in `services::shelf`.

mod actions;
mod floating;
mod page;
mod tiles;

pub(crate) use actions::*;
pub(crate) use floating::*;
pub(crate) use page::*;
pub(crate) use tiles::*;
