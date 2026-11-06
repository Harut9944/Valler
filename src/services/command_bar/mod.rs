//! Command bar engine. Each kind of answer is its own small module: the
//! calculator, unit conversion, colours, dates and emoji, plus the fuzzy
//! matcher that ranks tools, apps and saved items. The screen in
//! `screens::tools::command_bar` turns these into rows.

mod calc;
mod colors;
mod dates;
mod emoji;
mod fuzzy;
mod units;

pub use calc::{calc, fmt_num};
pub use colors::color;
pub use dates::date_answer;
pub use emoji::emoji;
pub use fuzzy::score;
pub use units::convert;

#[cfg(test)]
mod tests;
