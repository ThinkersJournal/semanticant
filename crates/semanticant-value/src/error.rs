use std::fmt;

/// Why rule data could not be built from a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueError {
    /// The pack's `kind` is not the one this table needs.
    WrongKind {
        /// The kind found.
        found: String,
        /// The kind wanted.
        want: &'static str,
    },
    /// A required file is not listed in the pack.
    MissingFile(&'static str),
    /// A row is malformed (file, 1-based data row, what is wrong).
    BadRow(&'static str, usize, &'static str),
    /// Two rows define the same unit id differently.
    ConflictingUnit(String),
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongKind { found, want } => write!(f, "pack kind is {found:?}, want {want:?}"),
            Self::MissingFile(n) => write!(f, "pack does not list {n}"),
            Self::BadRow(file, row, why) => write!(f, "{file}: data row {row}: {why}"),
            Self::ConflictingUnit(id) => {
                write!(f, "unit {id:?} is defined twice with different values")
            }
        }
    }
}
impl std::error::Error for ValueError {}
