//! Normalisation, folding, tokenisation and sentence segmentation. See the crate README.
#![forbid(unsafe_code)]

mod fold;
mod normalise;
mod segment;

pub use fold::{FoldRules, fold};
pub use normalise::{NormForm, Normalised, normalise};
pub use segment::{AbbreviationSet, Segmenter, Token, TokenKind, UaxSegmenter};
