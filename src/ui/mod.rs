//! Visual building blocks shared by every screen. Each concern lives in its
//! own file and is re-exported here, so screens simply `use crate::ui`.

mod chrome;
pub mod fonts;
mod format;
mod icon;
mod markdown;
mod theme;
mod widgets;

pub use chrome::*;
pub use format::*;
pub use icon::*;
pub use markdown::markdown;
pub use theme::*;
pub use widgets::*;

#[cfg(test)]
mod tests;
