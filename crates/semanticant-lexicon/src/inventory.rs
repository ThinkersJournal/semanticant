use semanticant_core::LangTag;
use semanticant_pack::Pack;
use std::collections::HashMap;
use std::fmt;

/// An opaque sense handle, meaningful only inside the inventory that issued it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SenseId(pub(crate) u32);

/// A typed link between senses. Any word not in the fixed list is kept as `Other`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relation {
    /// Same meaning.
    Synonym,
    /// Opposite meaning.
    Antonym,
    /// More general sense.
    Hypernym,
    /// More specific sense.
    Hyponym,
    /// Part of.
    Meronym,
    /// Has part.
    Holonym,
    /// Loosely related.
    Related,
    /// A relation this crate does not name.
    Other(String),
}

impl Relation {
    /// Parse a relation word (case-insensitive).
    pub fn parse(s: &str) -> Relation {
        match s.trim().to_lowercase().as_str() {
            "synonym" => Relation::Synonym,
            "antonym" => Relation::Antonym,
            "hypernym" => Relation::Hypernym,
            "hyponym" => Relation::Hyponym,
            "meronym" => Relation::Meronym,
            "holonym" => Relation::Holonym,
            "related" => Relation::Related,
            other => Relation::Other(other.to_string()),
        }
    }
    /// The automatically implied inverse, if this relation has one.
    pub fn inverse(&self) -> Option<Relation> {
        match self {
            Relation::Hypernym => Some(Relation::Hyponym),
            Relation::Hyponym => Some(Relation::Hypernym),
            Relation::Meronym => Some(Relation::Holonym),
            Relation::Holonym => Some(Relation::Meronym),
            Relation::Synonym => Some(Relation::Synonym),
            Relation::Antonym => Some(Relation::Antonym),
            _ => None,
        }
    }
}

/// Any source of word senses. Implement it over a database, a pack, a remote service.
pub trait SenseInventory {
    /// Senses of `lemma` in `lang` (exact, case-sensitive match on the stored lemma), in a stable order.
    fn senses(&self, lemma: &str, lang: &LangTag) -> Vec<SenseId>;
    /// Senses linked from `sense` by `rel`, in a stable order.
    fn relations(&self, sense: SenseId, rel: &Relation) -> Vec<SenseId>;
    /// The sense's gloss, if it has one.
    fn gloss(&self, sense: SenseId) -> Option<&str>;
    /// The lemmas that express the sense.
    fn lemmas(&self, sense: SenseId) -> Vec<&str>;
    /// Every sense the inventory holds, in a stable order.
    fn all_senses(&self) -> Vec<SenseId>;
}

/// Why an inventory could not be built from a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryError {
    /// The pack is not `kind = "lexicon"`.
    WrongKind(String),
    /// A required file is not listed in the pack.
    MissingFile(&'static str),
    /// A row has the wrong number of columns (file, 1-based row among data rows).
    BadRow(&'static str, usize),
    /// A language tag in a row is invalid.
    BadLang(String),
    /// Two senses share a key.
    DuplicateKey(String),
    /// A relation row names an unknown sense key.
    UnknownKey(String),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongKind(k) => write!(f, "pack kind is {k:?}, want \"lexicon\""),
            Self::MissingFile(n) => write!(f, "pack does not list {n}"),
            Self::BadRow(n, r) => write!(f, "{n}: data row {r} has the wrong number of columns"),
            Self::BadLang(l) => write!(f, "invalid language tag {l:?}"),
            Self::DuplicateKey(k) => write!(f, "duplicate sense key {k:?}"),
            Self::UnknownKey(k) => write!(f, "relation names unknown sense key {k:?}"),
        }
    }
}
impl std::error::Error for InventoryError {}

#[derive(Debug, Clone)]
struct Sense {
    key: String,
    lemmas: Vec<String>,
    gloss: String,
}

/// An inventory held in memory, built from a lexicon pack (format in `RULES.md`).
#[derive(Debug, Clone, Default)]
pub struct InMemoryInventory {
    senses: Vec<Sense>,
    by_lemma: HashMap<(String, String), Vec<SenseId>>,
    by_key: HashMap<String, SenseId>,
    rels: HashMap<(SenseId, Relation), Vec<SenseId>>,
}

impl InMemoryInventory {
    /// Build from a verified pack.
    pub fn from_pack(pack: &Pack) -> Result<Self, InventoryError> {
        let kind = &pack.manifest().kind;
        if kind != "lexicon" {
            return Err(InventoryError::WrongKind(kind.clone()));
        }
        let senses = pack
            .tsv("senses.tsv")
            .ok_or(InventoryError::MissingFile("senses.tsv"))?;
        let rels = pack
            .tsv("relations.tsv")
            .ok_or(InventoryError::MissingFile("relations.tsv"))?;
        let mut inv = Self::default();
        for (n, row) in senses.iter().enumerate() {
            if row.len() != 4 {
                return Err(InventoryError::BadRow("senses.tsv", n + 1));
            }
            let lang =
                LangTag::parse(row[1]).map_err(|_| InventoryError::BadLang(row[1].to_string()))?;
            let lemmas: Vec<String> = row[2]
                .split('|')
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect();
            let id = SenseId(u32::try_from(inv.senses.len()).expect("under 4 billion senses"));
            if inv.by_key.insert(row[0].to_string(), id).is_some() {
                return Err(InventoryError::DuplicateKey(row[0].to_string()));
            }
            for l in &lemmas {
                inv.by_lemma
                    .entry((lang.to_string(), l.clone()))
                    .or_default()
                    .push(id);
            }
            inv.senses.push(Sense {
                key: row[0].to_string(),
                lemmas,
                gloss: row[3].to_string(),
            });
        }
        for (n, row) in rels.iter().enumerate() {
            if row.len() != 3 {
                return Err(InventoryError::BadRow("relations.tsv", n + 1));
            }
            let from = *inv
                .by_key
                .get(row[0])
                .ok_or_else(|| InventoryError::UnknownKey(row[0].to_string()))?;
            let to = *inv
                .by_key
                .get(row[2])
                .ok_or_else(|| InventoryError::UnknownKey(row[2].to_string()))?;
            let rel = Relation::parse(row[1]);
            inv.link(from, rel.clone(), to);
            if let Some(inv_rel) = rel.inverse() {
                inv.link(to, inv_rel, from);
            }
        }
        Ok(inv)
    }

    fn link(&mut self, from: SenseId, rel: Relation, to: SenseId) {
        let v = self.rels.entry((from, rel)).or_default();
        if !v.contains(&to) {
            v.push(to);
        }
    }

    /// The pack-local key of a sense (what the data file calls it).
    pub fn key(&self, sense: SenseId) -> Option<&str> {
        self.senses.get(sense.0 as usize).map(|s| s.key.as_str())
    }
    /// Look a sense up by its pack-local key.
    pub fn by_key(&self, key: &str) -> Option<SenseId> {
        self.by_key.get(key).copied()
    }
}

impl SenseInventory for InMemoryInventory {
    fn senses(&self, lemma: &str, lang: &LangTag) -> Vec<SenseId> {
        self.by_lemma
            .get(&(lang.to_string(), lemma.to_string()))
            .cloned()
            .unwrap_or_default()
    }
    fn relations(&self, sense: SenseId, rel: &Relation) -> Vec<SenseId> {
        self.rels
            .get(&(sense, rel.clone()))
            .cloned()
            .unwrap_or_default()
    }
    fn gloss(&self, sense: SenseId) -> Option<&str> {
        self.senses
            .get(sense.0 as usize)
            .map(|s| s.gloss.as_str())
            .filter(|g| !g.is_empty())
    }
    fn lemmas(&self, sense: SenseId) -> Vec<&str> {
        self.senses
            .get(sense.0 as usize)
            .map(|s| s.lemmas.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }
    fn all_senses(&self) -> Vec<SenseId> {
        (0..self.senses.len()).map(|i| SenseId(i as u32)).collect()
    }
}
