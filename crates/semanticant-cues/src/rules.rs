use semanticant_pack::Pack;
use std::collections::HashSet;
use std::fmt;

/// What a cue expresses. Any kind word not listed is kept as `Other`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CueKind {
    /// Denial ("not", "kein").
    Negation,
    /// Possibility, obligation, ability ("may", "must").
    Modality,
    /// Reduced commitment ("suggests", "probably").
    Hedge,
    /// A kind this crate does not name.
    Other(String),
}

impl CueKind {
    fn parse(s: &str) -> CueKind {
        match s.trim().to_lowercase().as_str() {
            "negation" => CueKind::Negation,
            "modality" => CueKind::Modality,
            "hedge" => CueKind::Hedge,
            o => CueKind::Other(o.to_string()),
        }
    }
}

/// Which side of the cue its scope lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// After the cue ("not good").
    Forward,
    /// Before the cue ("good it is not": "is ruled out").
    Backward,
    /// The cue has no scope of its own.
    None,
}

#[derive(Debug, Clone)]
pub(crate) struct CueDef {
    pub words: Vec<String>,
    pub kind: CueKind,
    pub direction: Direction,
    pub window: usize,
    pub strength: f32,
}

/// Cue phrases and scope rules of a language or genre. DATA from a pack or built by the caller.
#[derive(Debug, Clone)]
pub struct CueRules {
    pub(crate) cues: Vec<CueDef>,
    pub(crate) pseudo: Vec<Vec<String>>,
    pub(crate) terminators: HashSet<String>,
    pub(crate) ends: HashSet<String>,
}

impl Default for CueRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Why rules could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CueError {
    /// The pack's `kind` is not `cues`.
    WrongKind(String),
    /// A required file is not listed.
    MissingFile(&'static str),
    /// A malformed row (file, 1-based data row, problem).
    BadRow(&'static str, usize, &'static str),
}

impl fmt::Display for CueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongKind(k) => write!(f, "pack kind is {k:?}, want \"cues\""),
            Self::MissingFile(n) => write!(f, "pack does not list {n}"),
            Self::BadRow(file, row, why) => write!(f, "{file}: data row {row}: {why}"),
        }
    }
}
impl std::error::Error for CueError {}

fn words(phrase: &str) -> Vec<String> {
    phrase.split_whitespace().map(str::to_lowercase).collect()
}

impl CueRules {
    /// Empty rules with the default sentence-end tokens.
    pub fn new() -> Self {
        Self {
            cues: Vec::new(),
            pseudo: Vec::new(),
            terminators: HashSet::new(),
            ends: [".", "!", "?", ";", "\u{3002}", "\u{ff01}", "\u{ff1f}"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        }
    }
    /// Add a cue. `window` is clamped to at least 1 and `strength` to `0.0..=1.0`. An empty phrase is ignored.
    pub fn with_cue(
        mut self,
        phrase: &str,
        kind: CueKind,
        direction: Direction,
        window: usize,
        strength: f32,
    ) -> Self {
        let w = words(phrase);
        if !w.is_empty() {
            self.cues.push(CueDef {
                words: w,
                kind,
                direction,
                window: window.max(1),
                strength: strength.clamp(0.0, 1.0),
            });
        }
        self
    }
    /// Add a scope terminator token (scope stops before it).
    pub fn with_terminator(mut self, word: &str) -> Self {
        self.terminators.insert(word.to_lowercase());
        self
    }
    /// Add a pseudo-cue phrase that suppresses any cue wholly inside it.
    pub fn with_pseudo(mut self, phrase: &str) -> Self {
        let w = words(phrase);
        if !w.is_empty() {
            self.pseudo.push(w);
        }
        self
    }
    /// Replace the sentence-end tokens.
    pub fn with_ends(mut self, ends: &[&str]) -> Self {
        self.ends = ends.iter().map(|s| (*s).to_string()).collect();
        self
    }
    /// Build from a `kind = "cues"` pack (format in `RULES.md`).
    pub fn from_pack(pack: &Pack) -> Result<Self, CueError> {
        let kind = &pack.manifest().kind;
        if kind != "cues" {
            return Err(CueError::WrongKind(kind.clone()));
        }
        let mut r = Self::new();
        for (n, row) in pack
            .tsv("cues.tsv")
            .ok_or(CueError::MissingFile("cues.tsv"))?
            .iter()
            .enumerate()
        {
            let n = n + 1;
            if row.len() != 5 {
                return Err(CueError::BadRow("cues.tsv", n, "want 5 columns"));
            }
            let direction = match row[2].trim() {
                "forward" => Direction::Forward,
                "backward" => Direction::Backward,
                "none" => Direction::None,
                _ => {
                    return Err(CueError::BadRow(
                        "cues.tsv",
                        n,
                        "direction must be forward, backward or none",
                    ));
                }
            };
            let window: usize =
                row[3]
                    .trim()
                    .parse()
                    .ok()
                    .filter(|w| *w >= 1)
                    .ok_or(CueError::BadRow(
                        "cues.tsv",
                        n,
                        "window must be a whole number >= 1",
                    ))?;
            let strength: f32 = row[4]
                .trim()
                .parse()
                .ok()
                .filter(|s| (0.0..=1.0).contains(s))
                .ok_or(CueError::BadRow("cues.tsv", n, "strength must be 0..=1"))?;
            if words(row[0]).is_empty() {
                return Err(CueError::BadRow("cues.tsv", n, "empty phrase"));
            }
            r = r.with_cue(row[0], CueKind::parse(row[1]), direction, window, strength);
        }
        for row in pack.tsv("terminators.tsv").unwrap_or_default() {
            r = r.with_terminator(row[0]);
        }
        for row in pack.tsv("pseudo.tsv").unwrap_or_default() {
            r = r.with_pseudo(row[0]);
        }
        if let Some(rows) = pack.tsv("ends.tsv") {
            r.ends = rows.iter().map(|row| row[0].to_string()).collect();
        }
        Ok(r)
    }
}
