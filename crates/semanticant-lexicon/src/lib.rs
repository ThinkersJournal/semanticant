//! Sense inventories and similarity measures. See the crate README.
#![forbid(unsafe_code)]

mod inventory;
mod measure;
mod strings;
mod taxonomy;
mod tokens;
mod vectors;

pub use inventory::{InMemoryInventory, InventoryError, Relation, SenseId, SenseInventory};
pub use measure::Measure;
pub use strings::{Dice, JaroWinkler, Levenshtein};
pub use taxonomy::{Lin, PathSimilarity, Resnik, WuPalmer};
pub use tokens::{Containment, Jaccard, Overlap};
pub use vectors::{Cosine, Embedder};
