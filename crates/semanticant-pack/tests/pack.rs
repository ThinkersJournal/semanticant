#![allow(missing_docs)]
use semanticant_core::pack::{FileEntry, Manifest, SCHEMA_VERSION, Sha256Hex};
use semanticant_core::{LangTag, Provenance};
use semanticant_pack::*;
use std::collections::HashMap;
use std::path::PathBuf;

const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "semanticant-pack-test-{}-{tag}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}
fn manifest_for(files: &[(&str, &[u8])]) -> Manifest {
    Manifest {
        name: "demo".into(),
        version: "1.0.0".into(),
        kind: "abbreviations".into(),
        schema: SCHEMA_VERSION,
        lang: Some(LangTag::parse("en").unwrap()),
        provenance: Provenance {
            source: "Hand-written".into(),
            licence: "MIT OR Apache-2.0".into(),
            credit: "Authors \"quoted\"".into(),
            url: Some("https://example.org".into()),
            retrieved: Some("2026-11-18".into()),
        },
        files: files
            .iter()
            .map(|(n, b)| FileEntry {
                name: (*n).into(),
                sha256: Sha256Hex::parse(&sha256_hex(b)).unwrap(),
            })
            .collect(),
    }
}
fn write_pack(dir: &std::path::Path, m: &Manifest, files: &[(&str, &[u8])]) {
    std::fs::write(dir.join("pack.toml"), manifest_to_toml(m)).unwrap();
    for (n, b) in files {
        let p = dir.join(n);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b).unwrap();
    }
}

#[test]
fn sha256_known_vector() {
    assert_eq!(sha256_hex(b"abc"), ABC);
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn manifest_roundtrips_including_awkward_strings() {
    let m = manifest_for(&[("a.tsv", b"x"), ("sub/b.tsv", b"y")]);
    assert_eq!(parse_manifest(&manifest_to_toml(&m)).unwrap(), m);
    let mut m2 = m.clone();
    m2.lang = None;
    m2.provenance.url = None;
    m2.provenance.retrieved = None;
    assert_eq!(parse_manifest(&manifest_to_toml(&m2)).unwrap(), m2);
}

#[test]
fn opens_a_good_pack_and_reads_tsv() {
    let data: &[u8] = b"# abbreviations\nDr\tdoctor\n\nMr\tmister\n";
    let files = [("abbr.tsv", data)];
    let m = manifest_for(&files);
    let d = tmp("good");
    write_pack(&d, &m, &files);
    std::fs::write(d.join("unlisted.txt"), "ignored").unwrap();
    let p = Pack::open(&d).unwrap();
    assert_eq!(p.manifest(), &m);
    assert_eq!(p.file("abbr.tsv"), Some(data));
    assert_eq!(
        p.tsv("abbr.tsv").unwrap(),
        vec![vec!["Dr", "doctor"], vec!["Mr", "mister"]]
    );
    assert_eq!(
        p.file("unlisted.txt"),
        None,
        "unlisted files are not served"
    );
    assert_eq!(p.file("nope"), None);
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn tampered_file_is_refused_naming_both_digests() {
    let files = [("abbr.tsv", b"Dr\n" as &[u8])];
    let m = manifest_for(&files);
    let d = tmp("tamper");
    write_pack(&d, &m, &files);
    std::fs::write(d.join("abbr.tsv"), "Dr\nEVIL\n").unwrap();
    match Pack::open(&d) {
        Err(PackError::ChecksumMismatch {
            file,
            expected,
            actual,
        }) => {
            assert_eq!(file, "abbr.tsv");
            assert_eq!(expected, sha256_hex(b"Dr\n"));
            assert_eq!(actual, sha256_hex(b"Dr\nEVIL\n"));
        }
        other => panic!("expected mismatch, got {other:?}"),
    }
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn missing_file_and_missing_manifest() {
    let files = [("abbr.tsv", b"x" as &[u8])];
    let m = manifest_for(&files);
    let d = tmp("missing");
    std::fs::write(d.join("pack.toml"), manifest_to_toml(&m)).unwrap();
    assert!(matches!(Pack::open(&d), Err(PackError::Missing(n)) if n == "abbr.tsv"));
    std::fs::remove_file(d.join("pack.toml")).unwrap();
    assert!(matches!(Pack::open(&d), Err(PackError::Io { .. })));
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn pack_without_licence_or_source_is_refused() {
    let files = [("a.tsv", b"x" as &[u8])];
    for (field, bad) in [("licence", true), ("source", true)] {
        let mut m = manifest_for(&files);
        if field == "licence" {
            m.provenance.licence = String::new();
        } else {
            m.provenance.source = String::new();
        }
        assert!(bad);
        let e = parse_manifest(&manifest_to_toml(&m)).unwrap_err();
        assert!(matches!(e, PackError::Invalid(_)), "{field}: {e}");
    }
    let text = manifest_to_toml(&manifest_for(&files)).replace("[provenance]", "[provenanc]");
    assert!(matches!(
        parse_manifest(&text),
        Err(PackError::Field("provenance"))
    ));
}

#[test]
fn malformed_manifests() {
    assert!(matches!(
        parse_manifest("this is = not [toml"),
        Err(PackError::Toml(_))
    ));
    assert!(matches!(
        parse_manifest(""),
        Err(PackError::Field("schema"))
    ));
    let files = [("a.tsv", b"x" as &[u8])];
    let good = manifest_to_toml(&manifest_for(&files));
    assert!(matches!(
        parse_manifest(&good.replace("schema = 1", "schema = 2")),
        Err(PackError::Invalid(_))
    ));
    assert!(matches!(
        parse_manifest(&good.replace("schema = 1", "schema = \"1\"")),
        Err(PackError::Field("schema"))
    ));
    assert!(matches!(
        parse_manifest(&good.replace("lang = \"en\"", "lang = \"e_n\"")),
        Err(PackError::Invalid(_))
    ));
    let upper = good.replace(&sha256_hex(b"x"), &sha256_hex(b"x").to_uppercase());
    assert!(matches!(parse_manifest(&upper), Err(PackError::Invalid(_))));
}

#[test]
fn escaping_file_names_are_refused_before_any_read() {
    let secret = tmp("secret");
    std::fs::write(secret.join("s.txt"), "top secret").unwrap();
    let d = tmp("escape");
    let mut m = manifest_for(&[("a.tsv", b"x")]);
    m.files[0].name = "../".to_string() + secret.file_name().unwrap().to_str().unwrap() + "/s.txt";
    m.files[0].sha256 = Sha256Hex::parse(&sha256_hex(b"top secret")).unwrap();
    std::fs::write(d.join("pack.toml"), manifest_to_toml(&m)).unwrap();
    assert!(matches!(Pack::open(&d), Err(PackError::Invalid(_))));
    std::fs::remove_dir_all(d).unwrap();
    std::fs::remove_dir_all(secret).unwrap();
}

#[test]
fn from_parts_verifies_and_keeps_only_listed_files() {
    let m = manifest_for(&[("a.tsv", b"x")]);
    let mut files: HashMap<String, Vec<u8>> = HashMap::new();
    files.insert("a.tsv".into(), b"x".to_vec());
    files.insert("extra".into(), b"y".to_vec());
    let p = Pack::from_parts(m.clone(), files.clone()).unwrap();
    assert_eq!(p.file("extra"), None);
    assert_eq!(p.text("a.tsv"), Some("x"));
    files.insert("a.tsv".into(), b"z".to_vec());
    assert!(matches!(
        Pack::from_parts(m.clone(), files),
        Err(PackError::ChecksumMismatch { .. })
    ));
    assert!(matches!(
        Pack::from_parts(m, HashMap::new()),
        Err(PackError::Missing(_))
    ));
}

#[test]
fn non_utf8_text_is_none_not_a_panic() {
    let m = manifest_for(&[("b.bin", &[0xff, 0xfe])]);
    let mut f = HashMap::new();
    f.insert("b.bin".to_string(), vec![0xff, 0xfe]);
    let p = Pack::from_parts(m, f).unwrap();
    assert_eq!(p.text("b.bin"), None);
    assert_eq!(p.file("b.bin"), Some(&[0xff, 0xfe][..]));
}

#[cfg(unix)]
#[test]
fn symlinked_file_is_refused() {
    let files = [("a.tsv", b"x" as &[u8])];
    let m = manifest_for(&files);
    let d = tmp("symlink");
    let outside = tmp("symlink-target");
    std::fs::write(outside.join("t"), "x").unwrap();
    std::fs::write(d.join("pack.toml"), manifest_to_toml(&m)).unwrap();
    std::os::unix::fs::symlink(outside.join("t"), d.join("a.tsv")).unwrap();
    assert!(matches!(Pack::open(&d), Err(PackError::Symlink(_))));
}

#[test]
fn from_parts_refuses_an_invalid_manifest() {
    let mut m = manifest_for(&[("a.tsv", b"x")]);
    m.provenance.licence = String::new();
    let mut f = HashMap::new();
    f.insert("a.tsv".to_string(), b"x".to_vec());
    assert!(matches!(Pack::from_parts(m, f), Err(PackError::Invalid(_))));
}
