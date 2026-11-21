#![allow(missing_docs)]
use semanticant_core::pack::{FileEntry, Manifest, SCHEMA_VERSION, Sha256Hex};
use semanticant_core::{LangTag, Provenance};
use semanticant_lexicon::*;
use semanticant_pack::{Pack, sha256_hex};
use std::collections::{BTreeSet, HashMap};

fn en() -> LangTag {
    LangTag::parse("en").unwrap()
}

fn pack(kind: &str, senses: &str, rels: &str) -> Pack {
    let files = [("senses.tsv", senses), ("relations.tsv", rels)];
    let m = Manifest {
        name: "t".into(),
        version: "1".into(),
        kind: kind.into(),
        schema: SCHEMA_VERSION,
        lang: Some(en()),
        provenance: Provenance {
            source: "test".into(),
            licence: "MIT".into(),
            credit: "t".into(),
            url: None,
            retrieved: None,
        },
        files: files
            .iter()
            .map(|(n, b)| FileEntry {
                name: (*n).into(),
                sha256: Sha256Hex::parse(&sha256_hex(b.as_bytes())).unwrap(),
            })
            .collect(),
    };
    let data: HashMap<String, Vec<u8>> = files
        .iter()
        .map(|(n, b)| ((*n).to_string(), b.as_bytes().to_vec()))
        .collect();
    Pack::from_parts(m, data).unwrap()
}

// entity(root) <- animal, plant ; animal <- dog, cat ; plant <- tree ; rock (separate root). N = 7.
const SENSES: &str = "# key lang lemmas gloss
entity\ten\tentity|thing\ta thing
animal\ten\tanimal\ta living creature
plant\ten\tplant|flora\t
dog\ten\tdog|hound\ta canine
cat\ten\tcat\ta feline
tree\ten\ttree\ta woody plant
rock\ten\trock\t
";
const RELS: &str = "animal\thypernym\tentity
plant\thypernym\tentity
dog\thypernym\tanimal
cat\thypernym\tanimal
tree\thypernym\tplant
dog\tantonym\tcat
dog\tsynonym\thound_sense_is_not_here_but_fine_below
";

fn inv() -> InMemoryInventory {
    let rels = RELS.replace(
        "dog\tsynonym\thound_sense_is_not_here_but_fine_below\n",
        "dog\tfriend-of\ttree\n",
    );
    InMemoryInventory::from_pack(&pack("lexicon", SENSES, &rels)).unwrap()
}
fn id(i: &InMemoryInventory, k: &str) -> SenseId {
    i.by_key(k).unwrap()
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-9, "{a} != {b}");
}

#[test]
fn inventory_lookup_relations_and_inverses() {
    let i = inv();
    assert_eq!(i.senses("dog", &en()), vec![id(&i, "dog")]);
    assert_eq!(i.senses("hound", &en()), vec![id(&i, "dog")]);
    assert!(i.senses("Dog", &en()).is_empty(), "exact match");
    assert!(
        i.senses("dog", &LangTag::parse("fr").unwrap()).is_empty(),
        "language is part of the key"
    );
    assert_eq!(i.lemmas(id(&i, "plant")), vec!["plant", "flora"]);
    assert_eq!(i.gloss(id(&i, "dog")), Some("a canine"));
    assert_eq!(i.gloss(id(&i, "plant")), None);
    assert_eq!(
        i.relations(id(&i, "dog"), &Relation::Hypernym),
        vec![id(&i, "animal")]
    );
    let mut hypo = i.relations(id(&i, "animal"), &Relation::Hyponym);
    hypo.sort();
    assert_eq!(hypo, vec![id(&i, "dog"), id(&i, "cat")]);
    assert_eq!(
        i.relations(id(&i, "cat"), &Relation::Antonym),
        vec![id(&i, "dog")],
        "antonym is symmetric"
    );
    assert_eq!(
        i.relations(id(&i, "dog"), &Relation::Other("friend-of".into())),
        vec![id(&i, "tree")]
    );
    assert_eq!(
        i.relations(id(&i, "tree"), &Relation::Other("friend-of".into())),
        Vec::<SenseId>::new(),
        "no inverse invented for Other"
    );
    assert_eq!(i.all_senses().len(), 7);
    assert_eq!(i.key(id(&i, "rock")), Some("rock"));
}

#[test]
fn inventory_refuses_bad_packs() {
    assert_eq!(
        InMemoryInventory::from_pack(&pack("abbreviations", SENSES, RELS)).unwrap_err(),
        InventoryError::WrongKind("abbreviations".into())
    );
    assert!(matches!(
        InMemoryInventory::from_pack(&pack("lexicon", "a\ten\tx\n", "")),
        Err(InventoryError::BadRow("senses.tsv", 1))
    ));
    assert!(matches!(
        InMemoryInventory::from_pack(&pack("lexicon", "a\ten_x\tx\tg\n", "")),
        Err(InventoryError::BadLang(_))
    ));
    assert!(matches!(
        InMemoryInventory::from_pack(&pack("lexicon", "a\ten\tx\tg\na\ten\ty\tg\n", "")),
        Err(InventoryError::DuplicateKey(_))
    ));
    assert!(
        matches!(InMemoryInventory::from_pack(&pack("lexicon", "a\ten\tx\tg\n", "a\thypernym\tzzz\n")), Err(InventoryError::UnknownKey(k)) if k == "zzz")
    );
    assert!(matches!(
        InMemoryInventory::from_pack(&pack("lexicon", "a\ten\tx\tg\n", "a\thypernym\n")),
        Err(InventoryError::BadRow("relations.tsv", 1))
    ));
}

#[test]
fn string_measures_known_values() {
    close(
        Levenshtein.score("kitten", "sitting").value,
        1.0 - 3.0 / 7.0,
    );
    close(Levenshtein.score("", "").value, 1.0);
    close(Levenshtein.score("abc", "").value, 0.0);
    close(Levenshtein.score("héllo", "hello").value, 0.8);
    assert_eq!(Levenshtein::distance("flaw", "lawn"), 2);
    // classic Jaro-Winkler examples
    close(
        JaroWinkler.score("MARTHA", "MARHTA").value,
        0.9611111111111111,
    );
    close(
        JaroWinkler.score("DIXON", "DICKSONX").value,
        0.8133333333333332,
    );
    close(JaroWinkler.score("same", "same").value, 1.0);
    close(JaroWinkler.score("abc", "xyz").value, 0.0);
    close(JaroWinkler.score("", "").value, 1.0);
    close(JaroWinkler.score("", "a").value, 0.0);
    close(Dice.score("night", "nacht").value, 0.25);
    close(Dice.score("aaaa", "aa").value, 2.0 * 1.0 / 4.0);
    close(Dice.score("a", "a").value, 1.0);
    close(Dice.score("a", "b").value, 0.0);
    let v = Levenshtein.score("a", "b");
    assert_eq!(v.reasons[0].rule, "lexicon.string.levenshtein");
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn token_set_measures() {
    let (a, b) = (set(&["a", "b", "c"]), set(&["b", "c", "d", "e"]));
    close(Jaccard.score(&a, &b).value, 2.0 / 5.0);
    close(Overlap.score(&a, &b).value, 2.0 / 3.0);
    close(Containment.score(&a, &b).value, 2.0 / 3.0);
    close(Containment.score(&b, &a).value, 2.0 / 4.0);
    close(Jaccard.score(&set(&[]), &set(&[])).value, 1.0);
    close(Overlap.score(&set(&[]), &a).value, 0.0);
    close(Overlap.score(&set(&[]), &set(&[])).value, 1.0);
    close(Containment.score(&set(&[]), &a).value, 1.0);
    assert_eq!(
        Jaccard.score(&a, &b).reasons[0].rule,
        "lexicon.tokens.jaccard"
    );
}

#[test]
fn taxonomy_path_and_wu_palmer() {
    let i = inv();
    let (dog, cat, tree, animal, rock) = (
        id(&i, "dog"),
        id(&i, "cat"),
        id(&i, "tree"),
        id(&i, "animal"),
        id(&i, "rock"),
    );
    let p = PathSimilarity(&i);
    close(p.score(&dog, &cat).value, 1.0 / 3.0);
    close(p.score(&dog, &tree).value, 1.0 / 5.0);
    close(p.score(&dog, &dog).value, 1.0);
    close(p.score(&dog, &animal).value, 1.0 / 2.0);
    let w = WuPalmer(&i);
    close(w.score(&dog, &cat).value, 2.0 / 3.0);
    close(w.score(&dog, &tree).value, 1.0 / 3.0);
    close(w.score(&dog, &animal).value, 0.8);
    close(w.score(&dog, &dog).value, 1.0);
    for v in [p.score(&dog, &rock), w.score(&dog, &rock)] {
        close(v.value, 0.0);
        assert_eq!(v.reasons[0].rule, "lexicon.taxonomy.no_common_ancestor");
    }
    assert!(
        w.score(&dog, &cat).reasons[0].note.contains("animal"),
        "the reason names the common ancestor"
    );
}

#[test]
fn taxonomy_information_content() {
    let i = inv();
    let (dog, cat, tree, entity, rock) = (
        id(&i, "dog"),
        id(&i, "cat"),
        id(&i, "tree"),
        id(&i, "entity"),
        id(&i, "rock"),
    );
    let n = 7f64;
    let ic_animal = 1.0 - 3f64.ln() / n.ln(); // animal has 2 descendants
    let ic_entity = 1.0 - 6f64.ln() / n.ln(); // entity has 5
    close(Resnik(&i).score(&dog, &cat).value, ic_animal);
    close(Resnik(&i).score(&dog, &tree).value, ic_entity);
    close(Lin(&i).score(&dog, &cat).value, 2.0 * ic_animal / 2.0);
    close(Lin(&i).score(&dog, &dog).value, 1.0);
    close(Lin(&i).score(&dog, &tree).value, 2.0 * ic_entity / 2.0);
    close(Lin(&i).score(&rock, &rock).value, 1.0);
    close(Lin(&i).score(&entity, &entity).value, 1.0);
    close(Resnik(&i).score(&dog, &rock).value, 0.0);
    assert_eq!(
        Resnik(&i).score(&dog, &rock).reasons[0].rule,
        "lexicon.taxonomy.no_common_ancestor"
    );
    assert!(ic_animal > ic_entity, "more specific = more informative");
}

#[test]
fn jiang_conrath_distance_and_similarity() {
    let i = inv();
    let (dog, cat, tree, rock) = (id(&i, "dog"), id(&i, "cat"), id(&i, "tree"), id(&i, "rock"));
    let n = 7f64;
    let ic = |d: f64| 1.0 - (d + 1.0).ln() / n.ln();
    let (ic_dog, ic_cat, ic_tree, ic_animal, ic_entity) =
        (ic(0.0), ic(0.0), ic(0.0), ic(2.0), ic(5.0));
    let j = JiangConrath(&i);
    close(
        j.score(&dog, &cat).value,
        1.0 / (1.0 + ic_dog + ic_cat - 2.0 * ic_animal),
    );
    close(
        j.score(&dog, &tree).value,
        1.0 / (1.0 + ic_dog + ic_tree - 2.0 * ic_entity),
    );
    close(j.score(&dog, &dog).value, 1.0);
    assert!(j.score(&dog, &cat).value > j.score(&dog, &tree).value);
    let v = j.score(&dog, &rock);
    close(v.value, 0.0);
    assert_eq!(v.reasons[0].rule, "lexicon.taxonomy.no_common_ancestor");
}

#[test]
fn leacock_chodorow_normalised() {
    let i = inv();
    let (dog, cat, tree, rock) = (id(&i, "dog"), id(&i, "cat"), id(&i, "tree"), id(&i, "rock"));
    let l = LeacockChodorow(&i);
    let top = 6f64.ln(); // 2 * max depth (3)
    close(l.score(&dog, &dog).value, 1.0);
    close(l.score(&dog, &cat).value, 1.0 - 3f64.ln() / top);
    close(l.score(&dog, &tree).value, 1.0 - 5f64.ln() / top);
    assert!(l.score(&dog, &cat).value > l.score(&dog, &tree).value);
    let v = l.score(&dog, &rock);
    close(v.value, 0.0);
    assert_eq!(v.reasons[0].rule, "lexicon.taxonomy.no_common_ancestor");
}

#[test]
fn cyclic_hypernyms_terminate() {
    let i = InMemoryInventory::from_pack(&pack(
        "lexicon",
        "a\ten\ta\t\nb\ten\tb\t\n",
        "a\thypernym\tb\nb\thypernym\ta\n",
    ))
    .unwrap();
    let (a, b) = (id(&i, "a"), id(&i, "b"));
    close(PathSimilarity(&i).score(&a, &b).value, 1.0 / 2.0);
    let _ = WuPalmer(&i).score(&a, &b);
    let _ = Lin(&i).score(&a, &b);
}

struct Letters;
impl Embedder for Letters {
    fn embed(&self, t: &str) -> Vec<f32> {
        match t {
            "x" => vec![1.0, 0.0],
            "y" => vec![0.0, 1.0],
            "xy" => vec![1.0, 1.0],
            "neg" => vec![-1.0, 0.0],
            "zero" => vec![0.0, 0.0],
            _ => vec![1.0],
        }
    }
}

#[test]
fn cosine_over_a_caller_supplied_embedder() {
    let c = Cosine(Letters);
    close(c.score("x", "x").value, 1.0);
    close(c.score("x", "y").value, 0.0);
    close(c.score("x", "xy").value, std::f64::consts::FRAC_1_SQRT_2);
    close(c.score("x", "neg").value, 0.0);
    let z = c.score("x", "zero");
    close(z.value, 0.0);
    assert_eq!(z.reasons[0].rule, "lexicon.cosine.zero_vector");
    let m = c.score("x", "other");
    close(m.value, 0.0);
    assert_eq!(m.reasons[0].rule, "lexicon.cosine.dimension_mismatch");
}

#[test]
fn measures_are_usable_as_trait_objects() {
    let ms: Vec<Box<dyn Measure<str>>> =
        vec![Box::new(Levenshtein), Box::new(JaroWinkler), Box::new(Dice)];
    for m in &ms {
        close(m.score("same", "same").value, 1.0);
    }
}

#[test]
fn zero_information_senses() {
    // root with one child: IC(root) = 0, so Lin(root, root) hits the zero-denominator branch and must be 1 (equal); root vs child is 0.
    let i = InMemoryInventory::from_pack(&pack(
        "lexicon",
        "r	en	r	
c	en	c	
",
        "c	hypernym	r
",
    ))
    .unwrap();
    let (r, c) = (id(&i, "r"), id(&i, "c"));
    close(Resnik(&i).score(&r, &c).value, 0.0);
    close(Lin(&i).score(&r, &r).value, 1.0);
    close(Lin(&i).score(&r, &c).value, 0.0);
    let one = InMemoryInventory::from_pack(&pack(
        "lexicon", "a	en	a	
", "",
    ))
    .unwrap();
    let a = id(&one, "a");
    close(Lin(&one).score(&a, &a).value, 1.0);
    close(Resnik(&one).score(&a, &a).value, 0.0);
}

#[test]
fn jaro_winkler_gives_no_prefix_boost_below_jaro_0_7() {
    // jaro = (2/5 + 2/5 + 1) / 3 = 0.6 < 0.7, shared prefix "ab": the boost must NOT apply
    close(JaroWinkler.score("abxyz", "abqrs").value, 0.6);
}

#[test]
fn equally_informative_ancestors_resolve_deterministically() {
    // x and y both sit under p1 and p2 (equal IC): the reported ancestor must be the lower id (p1), every time.
    let senses = "p1\ten\tp1\t\np2\ten\tp2\t\nx\ten\tx\t\ny\ten\ty\t\n";
    let rels = "x\thypernym\tp1\nx\thypernym\tp2\ny\thypernym\tp1\ny\thypernym\tp2\n";
    for _ in 0..20 {
        let i = InMemoryInventory::from_pack(&pack("lexicon", senses, rels)).unwrap();
        let v = Resnik(&i).score(&id(&i, "x"), &id(&i, "y"));
        assert!(
            v.reasons[0].note.contains("ancestor p1"),
            "{}",
            v.reasons[0].note
        );
        let w = WuPalmer(&i).score(&id(&i, "x"), &id(&i, "y"));
        assert!(
            w.reasons[0].note.contains("lcs p1"),
            "{}",
            w.reasons[0].note
        );
    }
}
