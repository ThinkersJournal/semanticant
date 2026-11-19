//! Cue detection with scope. See the crate README.
#![forbid(unsafe_code)]

mod find;
mod rules;

pub use find::{Cue, find_cues};
pub use rules::{CueError, CueKind, CueRules, Direction};
