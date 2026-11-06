//! Everything the person sees: the panel tabs, the Utilities tools and the
//! Settings window. Screens read and change `ValleApp` state and call
//! `services` for real work; they hold no system logic of their own.

pub mod settings;
pub mod tabs;
pub mod tools;
