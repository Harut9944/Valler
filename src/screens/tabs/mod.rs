//! The panel tabs, one file each. Every page is a function taking the app
//! state and a `Ui`; `app::panel` picks the one for the current tab.

mod common;
mod controls;
mod disk;
mod displays;
mod keep_awake;
mod mixer;
mod network;
mod power;
mod system;

pub(crate) use common::*;
pub(crate) use controls::*;
pub(crate) use disk::*;
pub(crate) use displays::*;
pub(crate) use keep_awake::*;
pub(crate) use mixer::*;
pub(crate) use network::*;
pub(crate) use power::*;
pub(crate) use system::*;
